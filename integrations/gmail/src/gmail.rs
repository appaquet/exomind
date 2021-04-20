use crate::config::Config;
use exomind_protos::base::{Account, AccountScope, AccountType};
use google_gmail1::api::ModifyThreadRequest;
use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const CLIENT_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);
const FULL_ACCESS_SCOPE: &str = "https://mail.google.com/";

pub type HistoryId = u64;

pub struct GmailClient {
    account: GmailAccount,
    config: Config,
    client: google_gmail1::Gmail,
    last_refresh: Instant,
}

impl GmailClient {
    pub async fn new(config: &Config, account: GmailAccount) -> anyhow::Result<GmailClient> {
        let client = Self::create_client(config, &account).await?;

        Ok(GmailClient {
            account,
            config: config.clone(),
            client,
            last_refresh: Instant::now(),
        })
    }

    async fn create_client(
        config: &Config,
        account: &GmailAccount,
    ) -> anyhow::Result<google_gmail1::Gmail> {
        info!("Creating gmail client for account {}", account.email());

        let token_file = account_token_file(config, account.email())?;
        let app_secret = yup_oauth2::read_application_secret(&config.client_secret).await?;
        let auth =
            InstalledFlowAuthenticator::builder(app_secret, InstalledFlowReturnMethod::Interactive)
                .persist_tokens_to_disk(token_file)
                .build()
                .await?;

        auth.token(&[FULL_ACCESS_SCOPE]).await?;

        let client = google_gmail1::Gmail::new(
            hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
            auth,
        );

        Ok(client)
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

    pub async fn get_profile(&self) -> anyhow::Result<google_gmail1::api::Profile> {
        let (_resp, profile) = self
            .client
            .users()
            .get_profile("me")
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        Ok(profile)
    }

    pub async fn list_inbox_threads(
        &self,
        full: bool,
    ) -> anyhow::Result<Vec<google_gmail1::api::Thread>> {
        let (_resp, list) = self
            .client
            .users()
            .threads_list("me")
            .add_label_ids("INBOX")
            .max_results(100) // TODO: Should be done via paging instead
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        let partial_threads = list.threads.unwrap_or_default();

        let mut threads = Vec::new();
        for partial_thread in partial_threads {
            let thread_id = if let Some(id) = partial_thread.id.as_deref() {
                id
            } else {
                continue;
            };

            match self.fetch_thread(thread_id, full).await {
                Ok(thread) => {
                    threads.push(thread);
                }
                Err(err) => {
                    error!(
                        "Error fetching thread {} for account {}: {}",
                        thread_id,
                        self.account.email(),
                        err
                    );
                }
            }
        }

        Ok(threads)
    }

    pub async fn fetch_thread(
        &self,
        thread_id: &str,
        full: bool,
    ) -> anyhow::Result<google_gmail1::api::Thread> {
        let format = if full { "full" } else { "metadata" };

        let (_resp, thread) = self
            .client
            .users()
            .threads_get("me", thread_id)
            .format(format)
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        Ok(thread)
    }

    pub async fn add_label(
        &self,
        thread_id: &str,
        label: String,
    ) -> anyhow::Result<google_gmail1::api::Thread> {
        info!(
            "Adding label {} to {} in account {}",
            label,
            thread_id,
            self.account.email()
        );

        let req = ModifyThreadRequest {
            add_label_ids: Some(vec![label]),
            remove_label_ids: None,
        };
        let (_resp, thread) = self
            .client
            .users()
            .threads_modify(req, "me", thread_id)
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        Ok(thread)
    }

    pub async fn remove_label(
        &self,
        thread_id: &str,
        label: String,
    ) -> anyhow::Result<google_gmail1::api::Thread> {
        info!(
            "Removing label {} to {} in account {}",
            label,
            thread_id,
            self.account.email()
        );
        let req = ModifyThreadRequest {
            add_label_ids: None,
            remove_label_ids: Some(vec![label]),
        };

        let (_resp, thread) = self
            .client
            .users()
            .threads_modify(req, "me", thread_id)
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        Ok(thread)
    }

    pub async fn list_history(
        &self,
        history: HistoryId,
    ) -> anyhow::Result<Vec<GmailHistoryAction>> {
        let (_resp, history_resp) = self
            .client
            .users()
            .history_list("me")
            .label_id("INBOX")
            .start_history_id(&format!("{}", history))
            .add_scope(FULL_ACCESS_SCOPE)
            .doit()
            .await?;

        if history_resp.next_page_token.is_some() {
            // TODO: Implement history paging
            error!("History had next page, but paging wasn't implemented");
        }

        let mut actions = Vec::new();
        let mut imported_threads = HashSet::<String>::new();
        let mut removed_threads = HashSet::<String>::new();

        let history_list = history_resp.history.unwrap_or_default();
        for history in history_list {
            let history_id: HistoryId = history.id.as_ref().unwrap().parse().unwrap();
            let messages_added = history.messages_added.unwrap_or_default();
            for added in messages_added {
                let msg = added.message.as_ref().unwrap();
                let thread_id = msg.thread_id.as_deref().unwrap();

                if !imported_threads.contains(thread_id) {
                    imported_threads.insert(thread_id.to_string());

                    match self.fetch_thread(thread_id, true).await {
                        Ok(thread) => {
                            actions.push(GmailHistoryAction::AddToInbox(history_id, thread));
                        }
                        Err(err) => {
                            error!(
                                "Error fetching thread {} for account {}: {}",
                                thread_id,
                                self.account.email(),
                                err
                            );
                        }
                    }
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

                    match self.fetch_thread(thread_id, true).await {
                        Ok(thread) => {
                            actions.push(GmailHistoryAction::AddToInbox(history_id, thread));
                        }
                        Err(err) => {
                            error!(
                                "Error fetching thread {} for account {}: {}",
                                thread_id,
                                self.account.email(),
                                err
                            );
                        }
                    }
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
                        history_id,
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
    AddToInbox(HistoryId, google_gmail1::api::Thread),
    RemoveFromInbox(HistoryId, String),
}

pub fn account_token_file(config: &Config, email: &str) -> anyhow::Result<PathBuf> {
    let token_dir = PathBuf::from(&config.tokens_directory);
    if !token_dir.exists() {
        std::fs::create_dir(&token_dir)?;
    }

    Ok(token_dir.join(format!("token_{}.json", email)))
}
