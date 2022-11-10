use std::collections::HashMap;

use crate::{gmail::GmailAccount, parsing::FlaggedEmail, sync::SynchronizedThread};
use exocore::{
    core::time::{ConsistentTimestamp, DateTime, Utc},
    protos::{
        prost::{Message, ProstAnyPackMessageExt, ProstTimestampExt, Timestamp},
        store::{Entity, EntityResult, Reference, Trait},
        NamedMessage,
    },
    store::{
        entity::{EntityExt, TraitIdRef},
        mutation::{MutationBuilder, OperationId},
        query::{ProjectionBuilder, QueryBuilder, TraitQueryBuilder},
        remote::ClientHandle,
        store::Store,
    },
};
use exomind_protos::base::{Account, AccountType, CollectionChild, Email, EmailThread, Unread};

#[derive(Clone)]
pub struct ExomindClient {
    pub client: exocore::client::Client,
    pub store: ClientHandle,
}

impl ExomindClient {
    pub async fn new(client: exocore::client::Client) -> anyhow::Result<ExomindClient> {
        let store = client.store.clone();

        store.on_start().await;

        Ok(ExomindClient { client, store })
    }

    pub async fn get_accounts(&self, only_gmail: bool) -> anyhow::Result<Vec<GmailAccount>> {
        let query = QueryBuilder::with_trait::<Account>()
            .count(1000)
            .programmatic()
            .build();
        let results = self.store.query(query).await?;

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
            .project(ProjectionBuilder::for_trait::<Unread>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_all().skip())
            .count(100)
            .programmatic()
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
        let query = QueryBuilder::with_id(entity_id).programmatic().build();
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
            .project(ProjectionBuilder::for_trait::<Unread>().return_field_groups(vec![1]))
            .project(ProjectionBuilder::for_all().skip())
            .count(100)
            .order_by_operations(false)
            .programmatic()
            .build();

        let after_timestamp = ConsistentTimestamp::from(after_operation_id);
        let after_date = after_timestamp.to_datetime();

        let results = self.store.query(query).await?;
        let actions = results
            .entities
            .into_iter()
            .flat_map(|res| {
                if let Some(actions) = Self::history_result_to_action(res, account, after_date) {
                    actions
                } else {
                    vec![]
                }
            })
            .collect();

        Ok(actions)
    }

    fn history_result_to_action(
        entity_result: EntityResult,
        account: &GmailAccount,
        after_date: DateTime<Utc>,
    ) -> Option<Vec<ExomindHistoryAction>> {
        let mut ret = Vec::new();
        let entity = entity_result.entity?;

        let thread = entity.trait_of_type::<EmailThread>()?;
        let thread_account_entity_id = thread
            .instance
            .account
            .as_ref()
            .map(|f| f.entity_id.as_str());
        if thread_account_entity_id != Some(&account.entity_id) {
            return None;
        }

        let is_after_date = |ts: &Option<Timestamp>| -> bool {
            if let Some(delete_timestamp) = ts {
                let deleted_date = delete_timestamp.to_chrono_datetime();
                deleted_date > after_date
            } else {
                false
            }
        };

        {
            // check if email still in inbox
            let inbox_child_trait = get_inbox_child_trait(&entity)?;
            if is_after_date(&inbox_child_trait.trt.deletion_date) {
                let thread_entity = SynchronizedThread::from_exomind(&entity)?;
                ret.push(ExomindHistoryAction::RemovedFromInbox(
                    inbox_child_trait.trt.last_operation_id,
                    thread_entity,
                ));
            } else if is_after_date(&inbox_child_trait.trt.creation_date) {
                let thread_entity = SynchronizedThread::from_exomind(&entity)?;
                ret.push(ExomindHistoryAction::AddToInbox(
                    inbox_child_trait.trt.last_operation_id,
                    thread_entity,
                ));
            }
        }

        {
            // check unread flag
            for unread_trait in entity.traits_of_type::<Unread>() {
                let unread_ref = unread_trait.instance.entity.as_ref();
                let Some(unread_ref_trait) = unread_ref.map(|f| &f.trait_id) else {
                    continue;
                };

                if !is_email_trait_id(unread_ref_trait) {
                    continue;
                }

                let message_id = email_msg_id_from_trait_id(unread_ref_trait).to_string();

                if is_after_date(&unread_trait.trt.deletion_date) {
                    ret.push(ExomindHistoryAction::MarkRead(
                        unread_trait.trt.last_operation_id,
                        message_id.clone(),
                    ));
                } else if is_after_date(&unread_trait.trt.creation_date) {
                    ret.push(ExomindHistoryAction::MarkUnread(
                        unread_trait.trt.last_operation_id,
                        message_id.clone(),
                    ));
                }
            }
        }

        Some(ret)
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
            .and_then(|email| email.proto.received_date.clone());
        let modification_date = thread
            .emails
            .last()
            .and_then(|email| email.proto.received_date.clone());
        let thread_last_date = modification_date
            .as_ref()
            .or(creation_date.as_ref())
            .map(|t| t.to_chrono_datetime())
            .unwrap_or_else(Utc::now);

        // create thread if none exist
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

        let previous_emails: HashMap<String, &FlaggedEmail> = if let Some(prev) = previous_thread {
            prev.emails
                .iter()
                .map(|email| (email.proto.source_id.clone(), email))
                .collect()
        } else {
            HashMap::new()
        };

        // create emails & unread flags
        for email in &thread.emails {
            if let Some(prev_email) = previous_emails.get(&email.proto.source_id) {
                if prev_email.unread == email.unread {
                    // the email already exist in exomind & has the same unread flag, we skip it
                    continue;
                }
            }

            let email_trait_id = email_trait_id(&email.proto.source_id);
            let creation_date = email.proto.received_date.clone();

            let mut mutations = MutationBuilder::new().put_trait(
                &thread_entity_id,
                Trait {
                    id: email_trait_id.clone(),
                    message: Some(email.proto.pack_to_any()?),
                    creation_date: creation_date.clone(),
                    ..Default::default()
                },
            );

            let unread_trait_id = email_unread_trait_id(&email.proto.source_id);
            if email.unread {
                mutations = mutations.put_trait(
                    &thread_entity_id,
                    Trait {
                        id: unread_trait_id,
                        message: Some(email_unread_trait(&email.proto.source_id).pack_to_any()?),
                        creation_date,
                        ..Default::default()
                    },
                );
            } else {
                mutations = mutations.delete_trait(&thread_entity_id, unread_trait_id);
            }

            let mut res = self.store.mutate(mutations).await?;
            operation_ids.append(&mut res.operation_ids);
        }

        {
            // create or update inbox child
            let had_inbox_child = previous_thread.map_or(false, |c| c.inbox_child.is_some());
            let mut inbox_child = previous_thread
                .and_then(|c| c.inbox_child.clone())
                .unwrap_or_else(|| CollectionChild {
                    collection: Some(Reference {
                        entity_id: "inbox".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                });

            let weight_now = thread_last_date.timestamp_millis() as u64;

            if !had_inbox_child || inbox_child.weight < weight_now {
                inbox_child.weight = weight_now;

                let child_trait = Trait {
                    id: "child_inbox".to_string(),
                    message: Some(inbox_child.pack_to_any()?),
                    ..Default::default()
                };

                let mutation =
                    MutationBuilder::new().put_trait(thread_entity_id.clone(), child_trait);
                let mut res = self.store.mutate(mutation).await?;
                operation_ids.append(&mut res.operation_ids);
            }
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

    pub async fn mark_email_unread(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> anyhow::Result<Vec<OperationId>> {
        info!(
            "Marking thread {} email {} as unread from exomind",
            thread_id, message_id,
        );

        let unread_trait = Trait {
            id: email_unread_trait_id(message_id),
            message: Some(email_unread_trait(message_id).pack_to_any()?),
            ..Default::default()
        };

        let mutation =
            MutationBuilder::new().put_trait(thread_id_entity_id(thread_id), unread_trait);
        let res = self.store.mutate(mutation).await?;
        Ok(res.operation_ids)
    }

    pub async fn mark_email_read(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> anyhow::Result<Vec<OperationId>> {
        info!(
            "Marking thread {} email {} as read in exomind",
            thread_id, message_id,
        );

        let mutation = MutationBuilder::new().delete_trait(
            thread_id_entity_id(thread_id),
            email_unread_trait_id(thread_id),
        );
        let res = self.store.mutate(mutation).await?;
        Ok(res.operation_ids)
    }
}

fn get_inbox_child_trait(
    entity: &Entity,
) -> Option<exocore::store::entity::TraitInstance<CollectionChild>> {
    let inbox_child_trait = entity
        .traits_of_type::<CollectionChild>()
        .into_iter()
        .find(|c| {
            c.instance
                .collection
                .as_ref()
                .map(|r| r.entity_id == "inbox")
                .unwrap_or(false)
        })?;
    Some(inbox_child_trait)
}

pub enum ExomindHistoryAction {
    AddToInbox(OperationId, SynchronizedThread),
    RemovedFromInbox(OperationId, SynchronizedThread),
    MarkRead(OperationId, String),
    MarkUnread(OperationId, String),
}

impl ExomindHistoryAction {
    pub fn operation_id(&self) -> OperationId {
        match self {
            ExomindHistoryAction::AddToInbox(op_id, _) => *op_id,
            ExomindHistoryAction::RemovedFromInbox(op_id, _) => *op_id,
            ExomindHistoryAction::MarkRead(op_id, _) => *op_id,
            ExomindHistoryAction::MarkUnread(op_id, _) => *op_id,
        }
    }
}

pub fn thread_entity_id(thread: &EmailThread) -> String {
    thread_id_entity_id(&thread.source_id)
}

fn thread_id_entity_id(thread_id: &str) -> String {
    format!("bgt{}", thread_id)
}

pub fn email_trait_id(message_id: &str) -> String {
    format!("bge{}", message_id)
}

fn email_msg_id_from_trait_id(id: TraitIdRef) -> &str {
    id.trim_start_matches("bge")
}

fn is_email_trait_id(id: TraitIdRef) -> bool {
    id.starts_with("bge")
}

pub fn email_unread_trait_id(message_id: &str) -> String {
    format!("bge{}_unread", message_id)
}

fn email_unread_trait(message_id: &str) -> Unread {
    Unread {
        entity: Some(Reference {
            trait_id: email_trait_id(message_id),
            ..Default::default()
        }),
    }
}
