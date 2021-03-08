use crate::{
    config::Config,
    exomind::ExomindClient,
    gmail::{self, GmailAccount, GmailClient},
};
use exocore::{
    protos::{prost::ProstAnyPackMessageExt, store::Trait},
    store::{mutation::MutationBuilder, remote::ClientHandle, store::Store},
};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    ListAccounts,
    Login(LoginOptions),
    Logout(LogoutOptions),
}

#[derive(StructOpt)]
pub struct LoginOptions {
    pub email: String,
}

#[derive(StructOpt)]
pub struct LogoutOptions {
    pub email: String,
}

pub async fn exec(store_handle: ClientHandle, config: Config, opt: Options) -> anyhow::Result<()> {
    let exm = ExomindClient::new(store_handle).await?;

    match opt.subcommand {
        SubCommand::ListAccounts => list_accounts(exm).await,
        SubCommand::Login(login_opt) => login(config, login_opt, exm).await,
        SubCommand::Logout(logout_opt) => logout(config, logout_opt, exm).await,
    }
}

async fn login(config: Config, opt: LoginOptions, exm: ExomindClient) -> anyhow::Result<()> {
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

async fn logout(config: Config, opt: LogoutOptions, exm: ExomindClient) -> anyhow::Result<()> {
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
