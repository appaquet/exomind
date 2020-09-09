use crate::{cli, gmail::GmailAccount, SynchronizedThread};
use exocore::{
    chain::operation::OperationId, core::protos::prost::ProstAnyPackMessageExt,
    core::time::ConsistentTimestamp, core::time::DateTime, core::time::Utc,
    index::entity::EntityExt, protos::index::EntityResult, protos::prost::ProstTimestampExt,
};
use exocore::{
    core::{cell::Cell, futures::spawn_future, time::Clock},
    index::{
        mutation::MutationBuilder,
        query::{QueryBuilder, TraitQueryBuilder},
        remote::{Client, ClientHandle},
    },
    protos::NamedMessage,
    protos::{
        index::{Entity, Reference, Trait},
        prost::Message,
    },
    transport::{Libp2pTransport, TransportLayer},
};
use exomind_core::protos::base::{
    Account, AccountType, Collection, CollectionChild, Email, EmailThread,
};

pub struct ExomindClient {
    pub store: ClientHandle,
}

impl ExomindClient {
    pub async fn new(config: &cli::Config) -> anyhow::Result<ExomindClient> {
        let config = exocore::core::cell::node_config_from_yaml_file(&config.node_config)?;
        let (cells, local_node) = Cell::new_from_local_node_config(config)?;
        let either_cell = cells
            .first()
            .ok_or_else(|| anyhow!("Node doesn't have any cell configured"))?;
        let cell = either_cell.cell();

        let clock = Clock::new();

        let mut transport = Libp2pTransport::new(local_node.clone(), Default::default());
        let store_transport = transport.get_handle(cell.clone(), TransportLayer::Index)?;

        spawn_future(async move {
            let res = transport.run().await;
            info!("Transport done: {:?}", res);
        });

        let store_client = Client::new(Default::default(), cell.clone(), clock, store_transport)?;
        let store_handle = store_client.get_handle();

        spawn_future(async move {
            let res = store_client.run().await;
            info!("Remote client done: {:?}", res);
        });

        store_handle.on_start().await;

        Ok(ExomindClient {
            store: store_handle,
        })
    }

    pub async fn create_base_objects(&self) -> anyhow::Result<()> {
        // TODO: move to exomind server

        let inbox_trait = Trait {
            id: "inbox".to_string(),
            message: Some(
                Collection {
                    name: "Inbox".to_string(),
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
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        };

        let mutations = MutationBuilder::new()
            .put_trait("inbox", inbox_trait)
            .put_trait("favorites", fav_trait);
        let _ = self.store.mutate(mutations).await?;

        Ok(())
    }

    pub async fn get_accounts(&self, only_gmail: bool) -> anyhow::Result<Vec<GmailAccount>> {
        let results = self
            .store
            .query(QueryBuilder::with_trait::<Account>().count(1000).build())
            .await?;

        let account_any_url = Account::protobuf_any_url();
        let gmail_account_type: i32 = AccountType::Gmail.into();

        let mut accounts = Vec::new();
        for entity in results.entities {
            if let Some(entity) = entity.entity {
                for trt in entity.traits {
                    if let Some(msg) = trt.message {
                        if msg.type_url == account_any_url {
                            let account = Account::decode(msg.value.as_slice())?;

                            if only_gmail && account.r#type != gmail_account_type {
                                continue;
                            }

                            accounts.push(GmailAccount {
                                entity_id: entity.id.clone(),
                                account,
                            });
                        }
                    }
                }
            }
        }

        Ok(accounts)
    }

    pub async fn list_inbox_entities(&self) -> anyhow::Result<Vec<Entity>> {
        let tq = TraitQueryBuilder::field_references("collection", "inbox").build();
        let query = QueryBuilder::with_trait_query::<CollectionChild>(tq).build();

        // TODO: Count
        // TODO: Projection

        let results = self.store.query(query).await?;
        let entities = results
            .entities
            .into_iter()
            .flat_map(|e| e.entity)
            .collect::<Vec<_>>();

        Ok(entities)
    }

    pub async fn list_inbox_history(
        &self,
        account: &GmailAccount,
        after_operation_id: OperationId,
    ) -> anyhow::Result<Vec<ExomindHistoryAction>> {
        // TODO: Projection

        let tq = TraitQueryBuilder::field_references("collection", "inbox").build();
        let query = QueryBuilder::with_trait_query::<CollectionChild>(tq)
            .include_deleted()
            .order_by_operations(false)
            .count(100)
            .build();

        let after_timestamp = ConsistentTimestamp::from(after_operation_id);
        let after_date = after_timestamp.to_datetime();

        let mut actions = Vec::new();
        let results = self.store.query(query).await?;
        for entity_result in results.entities {
            if let Some(action) = Self::history_result_to_action(entity_result, account, after_date)
            {
                actions.push(action);
            }
        }

        Ok(actions)
    }

    fn history_result_to_action(
        entity_result: EntityResult,
        account: &GmailAccount,
        after_date: DateTime<Utc>,
    ) -> Option<ExomindHistoryAction> {
        let entity = entity_result.entity?;

        let thread = entity.trait_of_type::<EmailThread>()?;
        if thread.instance.account.as_ref()?.entity_id != account.entity_id {
            return None;
        }

        let child_trait = entity
            .traits_of_type::<CollectionChild>()
            .into_iter()
            .find(|c| {
                c.instance
                    .collection
                    .as_ref()
                    .map(|r| r.entity_id == "inbox")
                    .unwrap_or(false)
            })?;

        if let Some(delete_timestamp) = &child_trait.trt.deletion_date {
            let deleted_date = delete_timestamp.to_chrono_datetime();
            if deleted_date > after_date {
                let thread_entity = SynchronizedThread::from_exomind(entity)?;
                return Some(ExomindHistoryAction::RemovedFromInbox(thread_entity));
            }
        }

        if let Some(create_timestamp) = &child_trait.trt.creation_date {
            let create_date = create_timestamp.to_chrono_datetime();
            if create_date > after_date {
                let thread_entity = SynchronizedThread::from_exomind(entity)?;
                return Some(ExomindHistoryAction::AddToInbox(thread_entity));
            }
        }

        None
    }

    pub async fn import_thread(&self, thread: &SynchronizedThread) -> anyhow::Result<()> {
        info!(
            "Importing thread {} from {} to exomind",
            thread.thread_id(),
            thread.account_entity_id
        );

        let thread_entity_id = thread_entity_id(&thread.thread);

        let thread_create_date = thread
            .emails
            .first()
            .and_then(|email| email.received_date.clone());
        let thread_modification_date = thread
            .emails
            .last()
            .and_then(|email| email.received_date.clone());
        let thread_last_date = thread_modification_date
            .as_ref()
            .or(thread_create_date.as_ref())
            .map(|t| t.to_chrono_datetime())
            .unwrap_or_else(|| Utc::now());

        {
            let thread_trait = Trait {
                id: thread_entity_id.clone(),
                message: Some(thread.thread.pack_to_any().unwrap()),
                creation_date: thread_create_date,
                modification_date: thread_modification_date,
                ..Default::default()
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), thread_trait);
            let _ = self.store.mutate(mutation).await.unwrap();
        }

        for email in &thread.emails {
            let creation_date = email.received_date.clone();
            let email_trait = Trait {
                id: email_trait_id(&email),
                message: Some(email.pack_to_any().unwrap()),
                creation_date,
                ..Default::default()
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), email_trait);
            let _ = self.store.mutate(mutation).await.unwrap();
        }

        {
            let child_trait = Trait {
                id: "child_inbox".to_string(),
                message: Some(
                    CollectionChild {
                        collection: Some(Reference {
                            entity_id: "inbox".to_string(),
                            ..Default::default()
                        }),
                        weight: thread_last_date.timestamp_millis() as u64,
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                ..Default::default()
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), child_trait);
            let _ = self.store.mutate(mutation).await.unwrap();
        }

        Ok(())
    }

    pub async fn remove_from_inbox(&self, thread_id: &str) -> anyhow::Result<()> {
        let thread_entity_id = thread_id_entity_id(thread_id);

        info!("Removing thread {} from exomind", thread_entity_id,);

        // TODO: This isn't right. It may have a different trait id
        let mutation = MutationBuilder::new().delete_trait(thread_entity_id, "child_inbox");
        let _ = self.store.mutate(mutation).await.unwrap();

        Ok(())
    }
}

pub enum ExomindHistoryAction {
    AddToInbox(SynchronizedThread),
    RemovedFromInbox(SynchronizedThread),
}

pub fn thread_entity_id(thread: &EmailThread) -> String {
    thread_id_entity_id(&thread.source_id)
}

pub fn thread_id_entity_id(thread_id: &str) -> String {
    format!("bgt{}", thread_id)
}

pub fn email_trait_id(email: &Email) -> String {
    format!("bge{}", email.source_id)
}
