use std::collections::HashMap;

use crate::{cli::Options, gmail::GmailAccount, sync::SynchronizedThread};
use exocore::{
    core::time::{ConsistentTimestamp, DateTime, Utc},
    protos::{
        prost::{Message, ProstAnyPackMessageExt, ProstTimestampExt},
        store::{Entity, EntityResult, Reference, Trait},
        NamedMessage,
    },
    store::{
        entity::EntityExt,
        mutation::{MutationBuilder, OperationId},
        query::{ProjectionBuilder, QueryBuilder, TraitQueryBuilder},
        remote::ClientHandle,
        store::Store,
    },
};
use exomind_protos::base::{Account, AccountType, CollectionChild, Email, EmailThread};

#[derive(Clone)]
pub struct ExomindClient {
    pub client: exocore::client::Client,
    pub store: ClientHandle,
}

impl ExomindClient {
    pub async fn new(opts: &Options) -> anyhow::Result<ExomindClient> {
        let client = exocore::client::Client::from_node_config_file(&opts.node_config).await?;
        let store = client.store.clone();

        Ok(ExomindClient { client, store })
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
        // TODO: Paging

        let tq = TraitQueryBuilder::field_references("collection", "inbox").build();
        let query = QueryBuilder::with_trait_query::<CollectionChild>(tq)
            .project(ProjectionBuilder::for_trait::<CollectionChild>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_trait::<Email>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_trait::<EmailThread>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_all().skip())
            .count(100)
            .build();

        let results = self.store.query(query).await?;
        let entities = results
            .entities
            .into_iter()
            .flat_map(|e| e.entity)
            .collect::<Vec<_>>();

        Ok(entities)
    }

    pub async fn get_entity(&self, entity_id: &str) -> anyhow::Result<Option<Entity>> {
        let query = QueryBuilder::with_id(entity_id).build();
        let results = self.store.query(query).await?;
        let entity = results.entities.into_iter().flat_map(|e| e.entity).next();

        Ok(entity)
    }

    pub async fn list_inbox_history(
        &self,
        account: &GmailAccount,
        after_operation_id: OperationId,
    ) -> anyhow::Result<Vec<ExomindHistoryAction>> {
        // TODO: Paging

        let tq = TraitQueryBuilder::field_references("collection", "inbox").build();
        let query = QueryBuilder::with_trait_query::<CollectionChild>(tq)
            .include_deleted()
            .project(ProjectionBuilder::for_trait::<CollectionChild>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_trait::<Email>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_trait::<EmailThread>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_all().skip())
            .count(100)
            .order_by_operations(false)
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
                let thread_entity = SynchronizedThread::from_exomind(&entity)?;
                return Some(ExomindHistoryAction::RemovedFromInbox(
                    entity.last_operation_id,
                    thread_entity,
                ));
            }
        }

        if let Some(create_timestamp) = &child_trait.trt.creation_date {
            let create_date = create_timestamp.to_chrono_datetime();
            if create_date > after_date {
                let thread_entity = SynchronizedThread::from_exomind(&entity)?;
                return Some(ExomindHistoryAction::AddToInbox(
                    entity.last_operation_id,
                    thread_entity,
                ));
            }
        }

        None
    }

    pub async fn import_thread(
        &self,
        thread: &SynchronizedThread,
        previous_thread: Option<&SynchronizedThread>,
    ) -> anyhow::Result<Vec<OperationId>> {
        let mut operation_ids = Vec::new();

        let thread_entity_id = thread_entity_id(&thread.thread);

        let creation_date = thread
            .emails
            .first()
            .and_then(|email| email.received_date.clone());
        let modification_date = thread
            .emails
            .last()
            .and_then(|email| email.received_date.clone());
        let thread_last_date = modification_date
            .as_ref()
            .or_else(|| creation_date.as_ref())
            .map(|t| t.to_chrono_datetime())
            .unwrap_or_else(Utc::now);

        if previous_thread.is_none() {
            let thread_trait = Trait {
                id: thread_entity_id.clone(),
                message: Some(thread.thread.pack_to_any()?),
                creation_date,
                modification_date,
                ..Default::default()
            };

            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), thread_trait);
            let mut res = self.store.mutate(mutation).await?;
            operation_ids.append(&mut res.operation_ids);
        }

        let previous_emails: HashMap<String, &Email> = if let Some(prev) = previous_thread {
            prev.emails
                .iter()
                .map(|email| (email.source_id.clone(), email))
                .collect()
        } else {
            HashMap::new()
        };

        for email in &thread.emails {
            if previous_emails.contains_key(&email.source_id) {
                continue;
            }

            let creation_date = email.received_date.clone();
            let email_trait = Trait {
                id: email_trait_id(&email),
                message: Some(email.pack_to_any()?),
                creation_date,
                ..Default::default()
            };

            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), email_trait);
            let mut res = self.store.mutate(mutation).await?;
            operation_ids.append(&mut res.operation_ids);
        }

        if previous_thread.map_or(true, |c| c._inbox_child.is_none()) {
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
                    .pack_to_any()?,
                ),
                ..Default::default()
            };

            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), child_trait);
            let mut res = self.store.mutate(mutation).await?;
            operation_ids.append(&mut res.operation_ids);
        }

        if !operation_ids.is_empty() {
            info!(
                "Imported {} objects for thread {} from {} to exomind",
                operation_ids.len(),
                thread.thread_id(),
                thread.account_entity_id
            );
        }

        Ok(operation_ids)
    }

    pub async fn remove_from_inbox(&self, thread_id: &str) -> anyhow::Result<Vec<OperationId>> {
        let thread_entity_id = thread_id_entity_id(thread_id);

        info!("Removing thread {} from exomind", thread_entity_id,);

        // TODO: This isn't right. It may have a different trait id
        let mutation = MutationBuilder::new().delete_trait(thread_entity_id, "child_inbox");
        let res = self.store.mutate(mutation).await?;

        Ok(res.operation_ids)
    }
}

pub enum ExomindHistoryAction {
    AddToInbox(OperationId, SynchronizedThread),
    RemovedFromInbox(OperationId, SynchronizedThread),
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
