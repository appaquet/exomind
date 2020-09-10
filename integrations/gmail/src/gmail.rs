use crate::cli;
use exomind_core::protos::base::{Account, AccountScope, AccountType};
use google_gmail1::schemas::ModifyThreadRequest;
use std::{collections::HashSet, path::PathBuf, time::Duration, time::Instant};
use tokio::task::block_in_place;
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const CLIENT_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub type HistoryId = u64;

pub struct GmailClient {
    account: GmailAccount,
    config: cli::Config,
    client: google_gmail1::Client,
    last_refresh: Instant,
}

impl GmailClient {
    pub async fn new(config: &cli::Config, account: GmailAccount) -> anyhow::Result<GmailClient> {
        let client = Self::create_client(config, &account).await?;

        Ok(GmailClient {
            account,
            config: config.clone(),
            client,
            last_refresh: Instant::now(),
        })
    }

    async fn create_client(
        config: &cli::Config,
        account: &GmailAccount,
    ) -> anyhow::Result<google_gmail1::Client> {
        let token_file = account_token_file(config, account.email())?;

        let secret = yup_oauth2::read_application_secret(&config.client_secret).await?;
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
                .persist_tokens_to_disk(token_file)
                .build()
                .await?;

        let scopes = &["https://mail.google.com/"];
        let token = auth.token(scopes).await?;

        Ok(google_gmail1::Client::new(YupAuth { token }))
    }

    pub async fn maybe_refresh(&mut self) -> anyhow::Result<()> {
        let elapsed_refresh = self.last_refresh.elapsed();

        if elapsed_refresh > CLIENT_REFRESH_INTERVAL {
            info!(
                "Refreshing gmail client for account {}",
                self.account.email()
            );
            self.client = Self::create_client(&self.config, &self.account).await?;
            self.last_refresh = Instant::now();
        }

        Ok(())
    }

    pub async fn get_profile(&self) -> anyhow::Result<google_gmail1::schemas::Profile> {
        let profile = block_in_place(|| self.client.users().get_profile("me").execute())?;

        Ok(profile)
    }

    pub async fn list_inbox_threads(
        &self,
        full: bool,
    ) -> anyhow::Result<Vec<google_gmail1::schemas::Thread>> {
        let list: google_gmail1::schemas::ListThreadsResponse = block_in_place(|| {
            self.client
                .users()
                .threads()
                .list("me")
                .label_ids("INBOX".to_string())
                .max_results(1000) // TODO: Should be done via paging instead
                .execute()
        })?;

        let partial_threads = list.threads.unwrap_or_default();

        let mut threads = Vec::new();
        for partial_thread in partial_threads {
            let thread_id = if let Some(id) = partial_thread.id.as_deref() {
                id
            } else {
                continue;
            };

            let thread = self.fetch_thread(thread_id, full).await?;
            threads.push(thread);
        }

        Ok(threads)
    }

    pub async fn fetch_thread(
        &self,
        thread_id: &str,
        full: bool,
    ) -> anyhow::Result<google_gmail1::schemas::Thread> {
        use google_gmail1::resources::users::threads::params::GetFormat;
        let format = if full {
            GetFormat::Full
        } else {
            GetFormat::Metadata
        };

        let thread: google_gmail1::schemas::Thread = block_in_place(|| {
            self.client
                .users()
                .threads()
                .get("me", thread_id)
                .format(format)
                .execute()
        })?;

        Ok(thread)
    }

    pub async fn add_label(
        &self,
        thread_id: &str,
        label: String,
    ) -> anyhow::Result<google_gmail1::schemas::Thread> {
        info!(
            "Adding label {} to {} in account {}",
            label,
            thread_id,
            self.account.email()
        );

        let thread: google_gmail1::schemas::Thread = block_in_place(|| {
            let req = ModifyThreadRequest {
                add_label_ids: Some(vec![label]),
                remove_label_ids: None,
            };

            self.client
                .users()
                .threads()
                .modify(req, "me", thread_id)
                .execute()
        })?;

        Ok(thread)
    }

    pub async fn remove_label(
        &self,
        thread_id: &str,
        label: String,
    ) -> anyhow::Result<google_gmail1::schemas::Thread> {
        info!(
            "Removing label {} to {} in account {}",
            label,
            thread_id,
            self.account.email()
        );

        let thread: google_gmail1::schemas::Thread = block_in_place(|| {
            let req = ModifyThreadRequest {
                add_label_ids: None,
                remove_label_ids: Some(vec![label]),
            };

            self.client
                .users()
                .threads()
                .modify(req, "me", thread_id)
                .execute()
        })?;

        Ok(thread)
    }

    pub async fn list_history(
        &self,
        history: HistoryId,
    ) -> anyhow::Result<Vec<GmailHistoryAction>> {
        let history_resp: google_gmail1::schemas::ListHistoryResponse = block_in_place(|| {
            self.client
                .users()
                .history()
                .list("me")
                .label_id("INBOX")
                .start_history_id(history)
                .execute()
        })?;

        if history_resp.next_page_token.is_some() {
            // TODO:
            error!("History had next page...");
        }

        let mut actions = Vec::new();
        let mut imported_threads = HashSet::<String>::new();
        let mut removed_threads = HashSet::<String>::new();

        let history_list = history_resp.history.unwrap_or_default();
        for history in history_list {
            let messages_added = history.messages_added.unwrap_or_default();
            for added in messages_added {
                let msg = added.message.as_ref().unwrap();
                let thread_id = msg.thread_id.as_deref().unwrap();

                if !imported_threads.contains(thread_id) {
                    imported_threads.insert(thread_id.to_string());

                    let thread = self.fetch_thread(thread_id, true).await?;
                    actions.push(GmailHistoryAction::AddToInbox(
                        history.id.unwrap().clone(),
                        thread,
                    ));
                }
            }

            let labels_added = history.labels_added.unwrap_or_default();
            for added in labels_added {
                let labels = added.label_ids.unwrap_or_default();
                if !labels.contains(&"INBOX".to_string()) {
                    continue;
                }

                let msg = added.message.as_ref().unwrap();
                let thread_id = msg.thread_id.as_deref().unwrap();

                if !imported_threads.contains(thread_id) {
                    imported_threads.insert(thread_id.to_string());

                    let thread = self.fetch_thread(thread_id, true).await?;
                    actions.push(GmailHistoryAction::AddToInbox(
                        history.id.unwrap().clone(),
                        thread,
                    ));
                }
            }

            let labels_removed = history.labels_removed.unwrap_or_default();
            for removed in labels_removed {
                let labels = removed.label_ids.unwrap_or_default();
                if !labels.contains(&"INBOX".to_string()) {
                    continue;
                }

                let msg = removed.message.as_ref().unwrap();
                let thread_id = msg.thread_id.as_deref().unwrap();

                if !removed_threads.contains(thread_id) {
                    removed_threads.insert(thread_id.to_string());

                    actions.push(GmailHistoryAction::RemoveFromInbox(
                        history.id.unwrap().clone(),
                        thread_id.to_string(),
                    ))
                }
            }
        }

        Ok(actions)
    }
}

#[derive(Debug, Clone)]
pub struct GmailAccount {
    pub entity_id: String,
    pub account: Account,
}

impl GmailAccount {
    pub fn from_email(email: &str) -> GmailAccount {
        let data = vec![("email".to_string(), email.to_string())]
            .into_iter()
            .collect();

        GmailAccount {
            entity_id: format!("exomind_{}", email),
            account: Account {
                key: email.to_string(),
                name: format!("Gmail - {}", email),
                r#type: AccountType::Gmail.into(),
                scopes: vec![AccountScope::Email.into()],
                data,
            },
        }
    }

    pub fn email(&self) -> &str {
        &self.account.key
    }
}

pub enum GmailHistoryAction {
    AddToInbox(HistoryId, google_gmail1::schemas::Thread),
    RemoveFromInbox(HistoryId, String),
}

pub fn account_token_file(config: &cli::Config, email: &str) -> anyhow::Result<PathBuf> {
    let token_dir = PathBuf::from(&config.tokens_directory);
    if !token_dir.exists() {
        std::fs::create_dir(&token_dir)?;
    }

    Ok(token_dir.join(format!("token_{}.json", email)))
}

#[derive(Debug)]
struct YupAuth {
    token: AccessToken,
}

impl google_api_auth::GetAccessToken for YupAuth {
    fn access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.token.as_str().to_string())
    }
}
