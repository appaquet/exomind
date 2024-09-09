mod capped_hashset;
pub mod cli;
mod config;
mod exomind;
mod gmail;
mod parsing;
mod sync;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_derive;

use std::{path::Path, time::Duration};

use cli::{LoginOptions, LogoutOptions};
use config::Config;
use exocore::{
    core::futures::{sleep, tokio::time::Instant},
    protos::{prost::ProstAnyPackMessageExt, store::Trait},
    store::{mutation::MutationBuilder, store::Store},
};
use exomind::ExomindClient;
use gmail::{GmailAccount, GmailClient};
use sync::AccountSynchronizer;

pub async fn handle<C: AsRef<Path>>(
    client: exocore::client::Client,
    node_dir: C,
    opt: &cli::Options,
) {
    let conf_path = node_dir.as_ref().join(&opt.conf);
    let config = Config::from_file(conf_path).expect("Failed to parse config");
    let exm = ExomindClient::new(client)
        .await
        .expect("Couldn't create exomind client");

    match &opt.subcommand {
        cli::Command::Daemon => daemon(config, exm).await.unwrap(),
        cli::Command::ListAccounts => list_accounts(exm).await.unwrap(),
        cli::Command::Login(login_opt) => login(config, login_opt, exm).await.unwrap(),
        cli::Command::Logout(logout_opt) => logout(config, logout_opt, exm).await.unwrap(),
    };
}

async fn daemon(config: Config, exm: ExomindClient) -> anyhow::Result<()> {
    info!("Starting a gmail synchronizer");

    let accounts = exm.get_accounts(true).await?;

    let mut account_synchronizers = Vec::new();
    for account in accounts {
        let gmail_client = GmailClient::new(&config, account.clone()).await?;
        let mut synchronizer = AccountSynchronizer::new(account, exm.clone(), gmail_client);

        if config.save_fixtures {
            synchronizer.save_fixtures = true;
        }

        account_synchronizers.push(synchronizer);
    }

    let full_sync_interval: Duration = config.full_sync_interval.into();
    let mut last_full_sync: Option<Instant> = None;

    loop {
        let sync_res = synchronize_accounts(
            &mut last_full_sync,
            full_sync_interval,
            &mut account_synchronizers,
        )
        .await;
        if let Err(e) = sync_res {
            error!("Error executing synchronization loop: {}", e);
        }

        // TODO: Watch query on exomind
        sleep(Duration::from_secs(10)).await;
    }
}

async fn synchronize_accounts(
    last_full_sync: &mut Option<Instant>,
    full_sync_interval: Duration,
    account_synchronizers: &mut [AccountSynchronizer],
) -> Result<(), anyhow::Error> {
    let should_full_sync = last_full_sync.map_or(true, |i| i.elapsed() > full_sync_interval);
    if should_full_sync {
        for sync in account_synchronizers.iter_mut() {
            sync.synchronize_inbox().await.map_err(|err| {
                anyhow!(
                    "failed to fully sync inbox for account {:?}: {}",
                    sync.account,
                    err
                )
            })?;
        }
        *last_full_sync = Some(Instant::now());
    }

    for sync in account_synchronizers.iter_mut() {
        sync.maybe_refresh_client().await.map_err(|err| {
            anyhow!(
                "failed to refresh client for account {:?}: {}",
                sync.account,
                err
            )
        })?;
        sync.synchronize_history().await.map_err(|err| {
            anyhow!(
                "failed to sync history for account {:?}: {}",
                sync.account,
                err
            )
        })?;
    }

    Ok(())
}

async fn login(config: Config, opt: &LoginOptions, exm: ExomindClient) -> anyhow::Result<()> {
    let account = GmailAccount::from_email(&opt.email);
    let gmc = GmailClient::new(&config, account.clone()).await?;

    let profile = gmc.get_profile().await?;

    if profile.email_address.as_deref() != Some(&opt.email) {
        panic!(
            "Token is logged in to a different email. Expected {}, got {:?}",
            opt.email, profile.email_address
        );
    }

    let mutations = MutationBuilder::new().put_trait(
        format!("exomind_{}", opt.email),
        Trait {
            id: opt.email.clone(),
            message: Some(account.account.pack_to_any()?),
            ..Default::default()
        },
    );

    let _ = exm.store.mutate(mutations).await?;

    Ok(())
}

async fn logout(config: Config, opt: &LogoutOptions, exm: ExomindClient) -> anyhow::Result<()> {
    if let Ok(token_file) = gmail::account_token_file(&config, &opt.email) {
        let _ = std::fs::remove_file(token_file);
    }

    let mutations = MutationBuilder::new().delete_entity(format!("exomind_{}", opt.email));
    let _ = exm.store.mutate(mutations).await?;

    Ok(())
}

async fn list_accounts(exm: ExomindClient) -> anyhow::Result<()> {
    let accounts = exm.get_accounts(true).await?;

    for account in accounts {
        println!("{:?}", account);
    }

    Ok(())
}
