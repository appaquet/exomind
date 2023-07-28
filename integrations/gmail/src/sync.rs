use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use crate::{
    capped_hashset::CappedHashSet,
    exomind::{email_trait_id, thread_entity_id, ExomindClient, ExomindHistoryAction},
    gmail::{GmailAccount, GmailClient, GmailHistoryAction, HistoryId},
    parsing::{self, FlaggedEmail},
};
use exocore::{
    core::time::{ConsistentTimestamp, Utc},
    protos::{
        prost::ProstTimestampExt,
        store::{Entity, Reference},
    },
    store::{
        entity::{EntityExt, EntityId},
        mutation::OperationId,
    },
};
use exomind_protos::base::{CollectionChild, Email, EmailThread, Unread};

pub struct AccountSynchronizer {
    pub account: GmailAccount,
    pub exomind: ExomindClient,
    pub gmail: GmailClient,
    pub last_gmail_history: Option<HistoryId>,
    pub generated_gmail_history: CappedHashSet<HistoryId>,
    pub last_exomind_operation: Option<OperationId>,
    pub generated_exomind_operations: CappedHashSet<OperationId>,
    pub save_fixtures: bool,
}

impl AccountSynchronizer {
    pub fn new(
        account: GmailAccount,
        exomind: ExomindClient,
        gmail: GmailClient,
    ) -> AccountSynchronizer {
        AccountSynchronizer {
            account,
            gmail,
            exomind,
            last_gmail_history: None,
            generated_gmail_history: CappedHashSet::new(100),
            last_exomind_operation: None,
            generated_exomind_operations: CappedHashSet::new(100),
            save_fixtures: false,
        }
    }

    pub async fn maybe_refresh_client(&mut self) -> anyhow::Result<()> {
        self.gmail.maybe_refresh().await?;
        Ok(())
    }

    pub async fn synchronize_inbox(&mut self) -> anyhow::Result<()> {
        info!("Doing full inbox sync for account {}", self.account.email());

        let gmail_threads = self.gmail.list_inbox_threads(true).await?;
        if self.save_fixtures {
            for thread in &gmail_threads {
                let path = format!("{}.new.json", thread.id.as_ref().unwrap());
                let mut f = std::fs::File::create(path)?;
                let json = serde_json::to_string_pretty(&thread)?;
                f.write_all(json.as_bytes())?;
            }
        }
        let gmail_threads = gmail_threads
            .into_iter()
            .flat_map(|th| SynchronizedThread::from_gmail(self.account.clone(), th))
            .collect::<Vec<_>>();

        let exomind_entities = self.exomind.list_inbox_entities().await?;
        let exomind_threads = exomind_entities
            .iter()
            .flat_map(SynchronizedThread::from_exomind)
            .map(|th| (th.thread_id().to_string(), th))
            .collect::<HashMap<String, SynchronizedThread>>();

        // import threads from gmail to exomind inbox
        let mut in_gmail = HashSet::new();
        for new_thread in gmail_threads {
            let thread_id = new_thread.thread_id();
            self.update_last_gmail_history(new_thread.history_id);
            in_gmail.insert(thread_id.to_string());

            let current_thread = exomind_threads.get(thread_id);
            let operation_ids = self
                .exomind
                .import_thread(&new_thread, current_thread)
                .await?;
            self.generated_exomind_operations.insert_all(&operation_ids);
        }

        // move to inbox emails that are in exomind's inbox
        for (thread_id, thread) in exomind_threads {
            if thread.account_entity_id == self.account.entity_id && !in_gmail.contains(&thread_id)
            {
                self.add_to_gmail_inbox(thread.thread_id()).await?;
            }
        }

        Ok(())
    }

    pub async fn synchronize_history(&mut self) -> anyhow::Result<()> {
        if let Some(last_gmail_history) = self.last_gmail_history {
            self.synchronize_gmail_history(Some(last_gmail_history))
                .await?;
        } else {
            self.synchronize_inbox().await?;
        }

        self.synchronize_exomind_history().await?;

        Ok(())
    }

    pub async fn synchronize_gmail_history(
        &mut self,
        last_history_id: Option<HistoryId>,
    ) -> anyhow::Result<()> {
        let last_history_id =
            if let Some(last_history_id) = last_history_id.or(self.last_gmail_history) {
                last_history_id
            } else {
                return Ok(());
            };

        let history_list = self.gmail.list_history(last_history_id).await?;
        if !history_list.is_empty() {
            info!(
                "Fetched {} history from gmail from history id {}",
                history_list.len(),
                last_history_id
            );
        }

        for history in history_list {
            let history_id = history.history_id();
            self.update_last_gmail_history(Some(history_id));
            if self.generated_gmail_history.contains(&history_id) {
                debug!(
                    "Dropping history {} because generated by ourself",
                    history_id
                );
                continue;
            }

            match history {
                GmailHistoryAction::AddToInbox(_history_id, thread) => {
                    let sync_thread = SynchronizedThread::from_gmail(self.account.clone(), thread);
                    if let Some(sync_thread) = sync_thread {
                        let current_entity = self
                            .exomind
                            .get_entity(&sync_thread.entity_id())
                            .await?
                            .and_then(|t| SynchronizedThread::from_exomind(&t));

                        let operation_ids = self
                            .exomind
                            .import_thread(&sync_thread, current_entity.as_ref())
                            .await?;
                        self.generated_exomind_operations.insert_all(&operation_ids);
                    }
                }
                GmailHistoryAction::RemoveFromInbox(_history_id, thread_id) => {
                    let operation_ids = self.exomind.remove_from_inbox(&thread_id).await?;
                    self.generated_exomind_operations.insert_all(&operation_ids);
                }
                GmailHistoryAction::MarkUnread(_history_id, thread_id, msg_id) => {
                    let operation_ids = self.exomind.mark_email_unread(&thread_id, &msg_id).await?;
                    self.generated_exomind_operations.insert_all(&operation_ids);
                }
                GmailHistoryAction::MarkRead(_history_id, thread_id, msg_id) => {
                    let operation_ids = self.exomind.mark_email_read(&thread_id, &msg_id).await?;
                    self.generated_exomind_operations.insert_all(&operation_ids);
                }
            }
        }

        Ok(())
    }

    pub async fn synchronize_exomind_history(&mut self) -> anyhow::Result<()> {
        let Some(last_operation_id) = self.last_exomind_operation else {
            // never fetched through history, we fetch last exomind history action for next sync via history
            if let Some(last_action) = self.latest_exomind_history_action().await? {
                self.last_exomind_operation = Some(last_action.operation_id());
            }
            return Ok(());
        };

        let history_actions = self
            .exomind
            .list_inbox_history(&self.account, last_operation_id)
            .await?;

        if !history_actions.is_empty() {
            info!(
                "Fetched {} history from exomind after operation {} ({:?})",
                history_actions.len(),
                last_operation_id,
                ConsistentTimestamp::from(last_operation_id).to_datetime(),
            );
        }

        for action in history_actions {
            let operation_id = action.operation_id();
            self.update_last_exomind_operation(Some(operation_id));
            if self.generated_exomind_operations.contains(&operation_id) {
                debug!(
                    "Dropping operation {} because generated by ourself",
                    operation_id
                );
                continue;
            }

            match action {
                ExomindHistoryAction::AddToInbox(_operation_id, thread) => {
                    self.add_to_gmail_inbox(thread.thread_id()).await?;
                }
                ExomindHistoryAction::RemovedFromInbox(_operation_id, thread) => {
                    self.remove_from_gmail_inbox(thread.thread_id()).await?;
                }
                ExomindHistoryAction::MarkRead(_operation_id, message_id) => {
                    self.mark_gmail_read(&message_id).await?;
                }
                ExomindHistoryAction::MarkUnread(_, message_id) => {
                    self.mark_gmail_unread(&message_id).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn latest_exomind_history_action(
        &mut self,
    ) -> anyhow::Result<Option<ExomindHistoryAction>> {
        let history_list = self.exomind.list_inbox_history(&self.account, 0).await?;
        Ok(history_list.into_iter().next())
    }

    pub fn update_last_gmail_history(&mut self, history_id: Option<HistoryId>) {
        let Some(history_id) = history_id else {
            return;
        };

        match self.last_gmail_history {
            Some(last_history_id) if last_history_id < history_id => {
                self.last_gmail_history = Some(history_id);
            }
            None => {
                self.last_gmail_history = Some(history_id);
            }
            _ => {}
        }
    }

    pub fn update_last_exomind_operation(&mut self, operation_id: Option<OperationId>) {
        let Some(operation_id) = operation_id else {
            return;
        };

        match self.last_exomind_operation {
            Some(last_operation_id) if last_operation_id < operation_id => {
                self.last_exomind_operation = Some(operation_id);
            }
            None => {
                self.last_exomind_operation = Some(operation_id);
            }
            _ => {}
        }
    }

    async fn add_to_gmail_inbox(&mut self, thread_id: &str) -> anyhow::Result<()> {
        let thread = self
            .gmail
            .add_thread_label(thread_id, "INBOX".to_string())
            .await?;

        if let Some(history_id) = thread.history_id {
            self.generated_gmail_history.insert(history_id);
        }

        Ok(())
    }

    async fn remove_from_gmail_inbox(&mut self, thread_id: &str) -> anyhow::Result<()> {
        let thread = self
            .gmail
            .remove_thread_label(thread_id, "INBOX".to_string())
            .await?;

        if let Some(history_id) = thread.history_id {
            self.generated_gmail_history.insert(history_id);
        }

        Ok(())
    }

    async fn mark_gmail_unread(&mut self, message_id: &str) -> anyhow::Result<()> {
        let message = self
            .gmail
            .add_message_label(message_id, "UNREAD".to_string())
            .await?;

        if let Some(history_id) = message.history_id {
            self.generated_gmail_history.insert(history_id);
        }

        Ok(())
    }

    async fn mark_gmail_read(&mut self, message_id: &str) -> anyhow::Result<()> {
        let message = self
            .gmail
            .remove_message_label(message_id, "UNREAD".to_string())
            .await?;

        if let Some(history_id) = message.history_id {
            self.generated_gmail_history.insert(history_id);
        }

        Ok(())
    }
}

pub struct SynchronizedThread {
    pub account_entity_id: String,
    pub thread: EmailThread,
    pub emails: Vec<FlaggedEmail>,
    pub inbox_child: Option<CollectionChild>,
    pub history_id: Option<HistoryId>,
    pub operation_id: Option<OperationId>,
}

impl SynchronizedThread {
    pub fn from_exomind(entity: &Entity) -> Option<SynchronizedThread> {
        let thread: EmailThread = entity.trait_of_type::<EmailThread>()?.instance;
        let account_entity_id = thread.account.as_ref()?.entity_id.clone();

        let inbox_child = entity
            .traits_of_type::<CollectionChild>()
            .into_iter()
            .find(|c| c.instance.collection.as_ref().map(|c| c.entity_id.as_ref()) == Some("inbox"))
            .map(|t| t.instance);

        let email_unread_flags = entity
            .traits_of_type::<Unread>()
            .into_iter()
            .flat_map(|flag| flag.instance.entity.map(|refer| refer.trait_id))
            .collect::<HashSet<_>>();

        let emails: Vec<FlaggedEmail> = entity
            .traits_of_type::<Email>()
            .into_iter()
            .map(|t| {
                let unread = email_unread_flags.contains(&email_trait_id(&t.instance.source_id));
                FlaggedEmail {
                    proto: t.instance,
                    unread,
                }
            })
            .collect();

        Some(SynchronizedThread {
            account_entity_id,
            thread,
            emails,
            inbox_child,
            history_id: None,
            operation_id: Some(entity.last_operation_id),
        })
    }

    pub fn from_gmail(
        account: GmailAccount,
        thread: google_gmail1::api::Thread,
    ) -> Option<SynchronizedThread> {
        let history_id = thread.history_id;
        let thread_id = thread.id.clone()?;
        let parsed = parsing::parse_thread(thread);
        if let Err(err) = &parsed {
            error!("Error parsing thread {:?}: {}", thread_id, err);
            return None;
        }

        let parsing::ParsedThread {
            mut thread,
            mut emails,
            labels,
        } = parsed.ok()?;

        let thread_entity_id = thread_entity_id(&thread);

        let account_ref = Reference {
            entity_id: account.entity_id.clone(),
            trait_id: account.email().to_string(),
        };

        thread.account = Some(account_ref.clone());
        for email in &mut emails {
            email.proto.account = Some(account_ref.clone());
        }

        if let Some(email) = emails.last() {
            thread.snippet = email.proto.snippet.clone();
            thread.subject = email.proto.subject.clone();
            thread.from = email.proto.from.clone();
            thread.last_email = Some(Reference {
                entity_id: thread_entity_id,
                trait_id: email_trait_id(&email.proto.source_id),
            })
        }

        let thread_create_date = emails
            .first()
            .and_then(|email| email.proto.received_date.clone());
        let thread_modification_date = emails
            .last()
            .and_then(|email| email.proto.received_date.clone());
        let thread_last_date = thread_modification_date
            .as_ref()
            .or(thread_create_date.as_ref())
            .map(|t| t.to_chrono_datetime())
            .unwrap_or_else(Utc::now);

        let inbox_child = if labels.contains(&"INBOX".to_string()) {
            Some(CollectionChild {
                collection: Some(Reference {
                    entity_id: "inbox".to_string(),
                    ..Default::default()
                }),
                weight: thread_last_date.timestamp_millis() as u64,
            })
        } else {
            None
        };

        Some(SynchronizedThread {
            account_entity_id: account.entity_id,
            thread,
            emails,
            inbox_child,
            history_id,
            operation_id: None,
        })
    }

    pub fn thread_id(&self) -> &str {
        &self.thread.source_id
    }

    pub fn entity_id(&self) -> EntityId {
        thread_entity_id(&self.thread)
    }
}
