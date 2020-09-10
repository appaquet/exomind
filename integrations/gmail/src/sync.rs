use std::{
    borrow::Borrow, collections::HashMap, collections::HashSet, collections::LinkedList,
    hash::Hash, io::Write,
};

use crate::{
    capped_hashset::CappedHashSet,
    exomind::ExomindHistoryAction,
    gmail::{GmailAccount, GmailClient, GmailHistoryAction, HistoryId},
};
use crate::{
    exomind::{email_trait_id, thread_entity_id, ExomindClient},
    parsing,
};
use exocore::core::{protos::prost::ProstTimestampExt, time::ConsistentTimestamp};
use exocore::{chain::operation::OperationId, core::protos::index::Reference};
use exocore::{core::time::Utc, index::entity::EntityId};
use exocore::{index::entity::EntityExt, protos::index::Entity};
use exomind_core::protos::base::{CollectionChild, Email, EmailThread};

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
            self.update_last_exomind_operation(thread.operation_id);

            if thread.account_entity_id == self.account.entity_id {
                if !in_gmail.contains(&thread_id) {
                    let history_id = thread.add_to_gmail_inbox(&self.gmail).await?;
                    self.update_last_gmail_history(history_id);

                    if let Some(history_id) = history_id {
                        self.generated_gmail_history.insert(history_id);
                    }
                }
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
        info!(
            "Fetch {} history from gmail from history id {}",
            history_list.len(),
            last_history_id
        );

        for history in history_list {
            match history {
                GmailHistoryAction::AddToInbox(history_id, thread) => {
                    self.update_last_gmail_history(Some(history_id));

                    if self.generated_gmail_history.contains(&history_id) {
                        info!("Dropping history {} because generated by ourself", history_id);
                        continue;
                    }

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
                GmailHistoryAction::RemoveFromInbox(history_id, thread_id) => {
                    self.update_last_gmail_history(Some(history_id));

                    if self.generated_gmail_history.contains(&history_id) {
                        info!("Dropping history {} because generated by ourself", history_id);
                        continue;
                    }

                    let operation_ids = self.exomind.remove_from_inbox(&thread_id).await?;
                    self.generated_exomind_operations.insert_all(&operation_ids);
                }
            }
        }

        Ok(())
    }

    pub async fn synchronize_exomind_history(&mut self) -> anyhow::Result<()> {
        let last_operation_id = self.last_exomind_operation.unwrap_or_default();

        let history_list = self
            .exomind
            .list_inbox_history(&self.account, last_operation_id)
            .await?;

        info!(
            "Fetch {} history from exomind after operation {} ({:?})",
            history_list.len(),
            last_operation_id,
            ConsistentTimestamp::from(last_operation_id).to_datetime(),
        );
        for history in history_list {
            match history {
                ExomindHistoryAction::AddToInbox(operation_id, thread) => {
                    self.update_last_exomind_operation(thread.operation_id);

                    if self.generated_exomind_operations.contains(&operation_id) {
                        info!("Dropping operation {} because generated by ourself", operation_id);
                        continue;
                    }

                    let history_id = thread.add_to_gmail_inbox(&self.gmail).await?;
                    if let Some(history_id) = history_id {
                        self.generated_gmail_history.insert(history_id);
                    }
                }
                ExomindHistoryAction::RemovedFromInbox(operation_id, thread) => {
                    self.update_last_exomind_operation(thread.operation_id);

                    if self.generated_exomind_operations.contains(&operation_id) {
                        info!("Dropping operation {} because generated by ourself", operation_id);
                        continue;
                    }

                    let history_id = thread.remove_from_gmail_inbox(&self.gmail).await?;
                    if let Some(history_id) = history_id {
                        self.generated_gmail_history.insert(history_id);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn update_last_gmail_history(&mut self, history_id: Option<HistoryId>) {
        let history_id = if let Some(history_id) = history_id {
            history_id
        } else {
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
        let operation_id = if let Some(operation_id) = operation_id {
            operation_id
        } else {
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
}

pub struct SynchronizedThread {
    pub account_entity_id: String,
    pub thread: EmailThread,
    pub emails: Vec<Email>,
    pub _inbox_child: Option<CollectionChild>,
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

        let emails: Vec<Email> = entity
            .traits_of_type::<Email>()
            .into_iter()
            .map(|t| t.instance)
            .collect();

        Some(SynchronizedThread {
            account_entity_id,
            thread,
            emails,
            _inbox_child: inbox_child,
            history_id: None,
            operation_id: Some(entity.last_operation_id),
        })
    }

    pub fn from_gmail(
        account: GmailAccount,
        thread: google_gmail1::schemas::Thread,
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
            email.account = Some(account_ref.clone());
        }

        if let Some(email) = emails.last() {
            thread.snippet = email.snippet.clone();
            thread.subject = email.subject.clone();
            thread.from = email.from.clone();
            thread.last_email = Some(Reference {
                entity_id: thread_entity_id.clone(),
                trait_id: email_trait_id(email),
            })
        }

        let thread_create_date = emails.first().and_then(|email| email.received_date.clone());
        let thread_modification_date = emails.last().and_then(|email| email.received_date.clone());
        let thread_last_date = thread_modification_date
            .as_ref()
            .or(thread_create_date.as_ref())
            .map(|t| t.to_chrono_datetime())
            .unwrap_or_else(|| Utc::now());

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
            account_entity_id: account.entity_id.clone(),
            thread,
            emails,
            _inbox_child: inbox_child,
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

    pub async fn add_to_gmail_inbox(&self, gmc: &GmailClient) -> anyhow::Result<Option<HistoryId>> {
        let thread = gmc.add_label(self.thread_id(), "INBOX".to_string()).await?;
        Ok(thread.history_id)
    }

    pub async fn remove_from_gmail_inbox(
        &self,
        gmc: &GmailClient,
    ) -> anyhow::Result<Option<HistoryId>> {
        let thread = gmc
            .remove_label(self.thread_id(), "INBOX".to_string())
            .await?;
        Ok(thread.history_id)
    }
}
