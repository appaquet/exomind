#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use std::{sync::Arc, time::Duration};

use exocore::{
    apps::sdk::prelude::*,
    protos::{
        prost::{ProstAnyPackMessageExt, ProstTimestampExt},
        store::{Entity, Reference, Trait},
    },
    store::{entity::EntityExt, mutation::MutationBuilder, query::QueryBuilder},
};
use exomind_protos::base::{Collection, CollectionChild, Snoozed};

#[exocore_app]
pub struct ExomindApp {}

impl ExomindApp {
    fn new() -> Self {
        ExomindApp {}
    }
}

impl App for ExomindApp {
    fn start(&self, exocore: &Exocore) -> Result<(), AppError> {
        info!("Application initialized");

        let store = exocore.store.clone();
        spawn(async move {
            create_base_entities(&store)
                .await
                .expect("error in check_base_entities");

            spawn(check_snoozed_loop(store.clone()));
        });

        Ok(())
    }
}

async fn create_base_entities(store: &Arc<Store>) -> anyhow::Result<()> {
    let inbox_trait = Trait {
        id: "inbox".to_string(),
        message: Some(
            Collection {
                name: "Inbox".to_string(),
                ..Default::default()
            }
            .pack_to_any()?,
        ),
        ..Default::default()
    };

    let fav_trait = Trait {
        id: "favorites".to_string(),
        message: Some(
            Collection {
                name: "Favorites".to_string(),
                ..Default::default()
            }
            .pack_to_any()?,
        ),
        ..Default::default()
    };

    let mutations = MutationBuilder::new()
        .put_trait("inbox", inbox_trait)
        .put_trait("favorites", fav_trait);
    let _ = store.mutate(mutations).await;

    info!("Application base entities created");

    Ok(())
}

async fn check_snoozed_loop(store: Arc<Store>) {
    loop {
        if let Err(err) = check_snoozed(&store).await {
            error!("Error checking for snoozed entity: {}", err);
        }

        sleep(Duration::from_secs(60)).await;
    }
}

async fn check_snoozed(store: &Arc<Store>) -> anyhow::Result<()> {
    let snoozed_list = get_snoozed(store).await?;
    debug!("Found {} entities to moved to inbox", snoozed_list.len());

    for snoozed_entity in snoozed_list {
        if let Err(err) = move_snoozed_inbox(store, &snoozed_entity).await {
            error!(
                "Error moving snoozed entity {} to inbox: {}",
                snoozed_entity.id, err
            );
        }
    }

    Ok(())
}

async fn move_snoozed_inbox(store: &Arc<Store>, snoozed_entity: &Entity) -> anyhow::Result<()> {
    let snoozed_trait = snoozed_entity
        .trait_of_type::<Snoozed>()
        .ok_or_else(|| anyhow!("no snoozed trait on entity"))?;

    let until_date = snoozed_trait
        .instance
        .until_date
        .map(|d| d.to_chrono_datetime())
        .ok_or_else(|| anyhow!("snoozed trait didn't have an until_date"))?;

    let now = now().to_chrono_datetime();
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

        let _ = store.mutate(mb.build()).await?;
    }

    Ok(())
}

async fn get_snoozed(store: &Arc<Store>) -> anyhow::Result<Vec<Entity>> {
    let query = QueryBuilder::with_trait::<Snoozed>()
        .count(100)
        .order_by_field("until_date", true)
        .programmatic()
        .build();

    let results = store.query(query).await?;
    let entities = results
        .entities
        .into_iter()
        .flat_map(|res| res.entity)
        .collect();

    Ok(entities)
}
