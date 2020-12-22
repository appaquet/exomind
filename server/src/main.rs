use std::{str::FromStr, time::Duration};

use exocore::{
    core::{
        protos::prost::{ProstAnyPackMessageExt, ProstTimestampExt},
        time::Utc,
    },
    protos::store::{Entity, Reference, Trait},
    store::{entity::EntityExt, mutation::MutationBuilder},
};
use exomind::ExomindClient;
use exomind_core::protos::base::{CollectionChild, Snoozed};
use futures::FutureExt;
use log::LevelFilter;
use structopt::StructOpt;
use tokio::time::sleep;

mod cli;
mod exomind;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_derive;

#[tokio::main]
async fn main() {
    let opt: cli::Options = cli::Options::from_args();
    exocore::core::logging::setup(Some(LevelFilter::from_str(&opt.logging_level).unwrap()));

    let config = cli::Config::from_file(&opt.config)
        .unwrap_or_else(|err| panic!("Couldn't parse config {:?}: {}", &opt.config, err));

    match opt.subcommand {
        cli::SubCommand::start => {
            start(config).await.unwrap();
        }
        cli::SubCommand::gmail(gmail_opt) => {
            let gmail_config = config
                .gmail
                .clone()
                .expect("Config didn't contain a gmail section");

            let exm = ExomindClient::new(&config)
                .await
                .expect("Couldn't create exomind client");
            exomind_gmail::cli::exec(exm.store.clone(), gmail_config, gmail_opt)
                .await
                .unwrap();
        }
    }
}

async fn start(config: cli::Config) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config)
        .await
        .expect("Couldn't create exomind client");

    let gmail_store_handle = exm.store.clone();
    let gmail_server = async move {
        if let Some(gmail_config) = config.gmail {
            exomind_gmail::server::run(gmail_config, gmail_store_handle).await?;
        } else {
            futures::future::pending::<()>().await;
        }

        Ok::<(), anyhow::Error>(())
    };

    let snooze_loop = async move {
        exm.create_base_entities().await?;

        loop {
            if let Err(err) = check_snoozed(&exm).await {
                error!("Error checking for snoozed entity: {}", err);
            }

            sleep(Duration::from_secs(60)).await;
        }

        // types the async block
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    };

    futures::select! {
        _ = snooze_loop.fuse() => {},
        _ = gmail_server.fuse() => {},
    }

    Ok(())
}

async fn check_snoozed(exm: &ExomindClient) -> anyhow::Result<()> {
    let snoozed_list = exm.get_snoozed().await?;
    for snoozed_entity in snoozed_list {
        if let Err(err) = move_snoozed_inbox(&exm, &snoozed_entity).await {
            error!(
                "Error moving snoozed entity {} to inbox: {}",
                snoozed_entity.id, err
            );
        }
    }

    Ok(())
}

async fn move_snoozed_inbox(exm: &ExomindClient, snoozed_entity: &Entity) -> anyhow::Result<()> {
    let snoozed_trait = snoozed_entity
        .trait_of_type::<Snoozed>()
        .ok_or_else(|| anyhow!("no snoozed trait on entity"))?;

    let until_date = snoozed_trait
        .instance
        .until_date
        .map(|d| d.to_chrono_datetime())
        .ok_or_else(|| anyhow!("snoozed trait didn't have an until_date"))?;

    let now = Utc::now();
    if until_date < now {
        info!("Moving snoozed entity {} to inbox", snoozed_entity.id);

        let mb = MutationBuilder::new()
            .delete_trait(&snoozed_entity.id, &snoozed_trait.trt.id)
            .put_trait(
                &snoozed_entity.id,
                Trait {
                    id: "child_inbox".to_string(),
                    message: Some(
                        CollectionChild {
                            collection: Some(Reference {
                                entity_id: "inbox".to_string(),
                                ..Default::default()
                            }),
                            weight: now.timestamp_millis() as u64,
                        }
                        .pack_to_any()?,
                    ),
                    ..Default::default()
                },
            );

        let _ = exm.store.mutate(mb.build()).await?;
    }

    Ok(())
}
