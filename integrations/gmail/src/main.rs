use exocore::core::protos::index::Trait;
use exocore::core::protos::prost::ProstAnyPackMessageExt;
use exocore::index::mutation::MutationBuilder;
use exomind::ExomindClient;
use gmail::{GmailAccount, GmailClient};
use log::LevelFilter;
use std::{str::FromStr, time::Duration};
use structopt::StructOpt;
use sync::{AccountSynchronizer, SynchronizedThread};
use tokio::time::delay_for;

mod cli;
mod exomind;
mod gmail;
mod parsing;
mod sync;
mod capped_hashset;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

#[tokio::main]
async fn main() {
    let opt: cli::Options = cli::Options::from_args();
    exocore::core::logging::setup(Some(LevelFilter::from_str(&opt.logging_level).unwrap()));

    let config = cli::Config::from_file(&opt.config)
        .unwrap_or_else(|err| panic!("Couldn't parse config {:?}: {}", &opt.config, err));

    match opt.subcommand {
        cli::SubCommand::start(start_opt) => {
            start(config, start_opt).await.unwrap();
        }
        cli::SubCommand::list_accounts => {
            list_accounts(config).await.unwrap();
        }
        cli::SubCommand::login(login_opt) => {
            login(config, login_opt).await.unwrap();
        }
        cli::SubCommand::logout(logout_opt) => {
            logout(config, logout_opt).await.unwrap();
        }
    }
}

async fn login(config: cli::Config, opt: cli::LoginOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;

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

async fn logout(config: cli::Config, opt: cli::LogoutOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;

    if let Ok(token_file) = gmail::account_token_file(&config, &opt.email) {
        let _ = std::fs::remove_file(token_file);
    }

    let mutations = MutationBuilder::new().delete_entity(format!("exomind_{}", opt.email));
    let _ = exm.store.mutate(mutations).await?;

    Ok(())
}

async fn list_accounts(config: cli::Config) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;
    let accounts = exm.get_accounts(true).await?;

    for account in accounts {
        println!("{:?}", account);
    }

    Ok(())
}

async fn start(config: cli::Config, opt: cli::StartOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;

    exm.create_base_objects().await?; // TODO: This shouldn't be here

    let accounts = exm.get_accounts(true).await?;
    let mut account_synchronizers = Vec::new();
    for account in accounts {
        let gmail_client = GmailClient::new(&config, account.clone()).await?;
        let mut synchronizer = AccountSynchronizer::new(account, exm.clone(), gmail_client);

        if opt.save_fixtures {
            synchronizer.save_fixtures = true;
        }

        account_synchronizers.push(synchronizer);
    }

    for sync in &mut account_synchronizers {
        sync.synchronize_inbox().await?;
    }

    loop {
        for sync in &mut account_synchronizers {
            if let Err(err)  =sync.maybe_refresh_client().await {
                error!(
                    "Error refreshing client for account {}: {}",
                    sync.account.email(),
                    err
                );
                continue;
            }

            if let Err(err) = sync.synchronize_history().await {
                error!(
                    "Error synchronizing via history for account {}: {}",
                    sync.account.email(),
                    err
                );
            }
        }

        // TODO: Watch query on exomind
        delay_for(Duration::from_secs(10)).await;
    }
}
