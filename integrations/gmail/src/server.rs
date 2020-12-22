use std::time::Duration;

use exocore::store::remote::ClientHandle;
use tokio::time::sleep;

use crate::{
    config::Config, exomind::ExomindClient, gmail::GmailClient, sync::AccountSynchronizer,
};

pub async fn run(config: Config, store_handle: ClientHandle) -> anyhow::Result<()> {
    info!("Starting a gmail synchronizer");

    let exm = ExomindClient::new(store_handle).await?;
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

    for sync in &mut account_synchronizers {
        sync.synchronize_inbox().await?;
    }

    loop {
        for sync in &mut account_synchronizers {
            if let Err(err) = sync.maybe_refresh_client().await {
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
        sleep(Duration::from_secs(10)).await;
    }
}
