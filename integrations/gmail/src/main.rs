use exocore::core::time::Utc;
use exocore::core::{
    protos::prost::{ProstAnyPackMessageExt, ProstTimestampExt},
    time::ConsistentTimestamp,
};
use exocore::{
    chain::operation::OperationId,
    core::protos::index::{Reference, Trait},
};
use exocore::{
    index::{entity::EntityExt, mutation::MutationBuilder},
    protos::index::Entity,
};
use exomind::{email_trait_id, thread_entity_id, ExomindClient, ExomindHistoryAction};
use exomind_core::protos::base::{CollectionChild, Email, EmailThread};
use gmail::{GmailAccount, GmailClient, GmailHistoryAction, HistoryId};
use log::LevelFilter;
use std::{collections::HashSet, str::FromStr, time::Duration, io::Write};
use structopt::StructOpt;
use tokio::time::delay_for;

mod cli;
mod exomind;
mod gmail;
mod parsing;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

// TODO: Account entity key vs trait key fix up
// TODO: Make sure account key is right in converter

#[tokio::main]
async fn main() {
    let opt: cli::Options = cli::Options::from_args();
    exocore::core::logging::setup(Some(LevelFilter::from_str(&opt.logging_level).unwrap()));

    let config = cli::Config::from_file(&opt.config)
        .unwrap_or_else(|err| panic!("Couldn't parse config {:?}: {}", &opt.config, err));

    match opt.subcommand {
        cli::SubCommand::start(start_opt) => {
            start(config, start_opt).await.unwrap();
        }
        cli::SubCommand::list_accounts => {
            list_accounts(config).await.unwrap();
        }
        cli::SubCommand::login(login_opt) => {
            login(config, login_opt).await.unwrap();
        }
        cli::SubCommand::logout(logout_opt) => {
            logout(config, logout_opt).await.unwrap();
        }
    }
}

async fn login(config: cli::Config, opt: cli::LoginOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;

    let account = GmailAccount::from_email(&opt.email);
    let gmc = GmailClient::new(&config, account.clone()).await?;

    let profile = gmc.get_profile().await?;

    if profile.email_address.as_deref() != Some(&opt.email) {
        panic!(
            "Token is logged in to a different email. Expected {}, got {:?}",
            opt.email, profile.email_address
        );
    }

    let mutations = MutationBuilder::new().put_trait(
        format!("exomind_{}", opt.email),
        Trait {
            id: opt.email.clone(),
            message: Some(account.account.pack_to_any()?),
            ..Default::default()
        },
    );

    let _ = exm.store.mutate(mutations).await?;

    Ok(())
}

async fn logout(config: cli::Config, opt: cli::LogoutOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;

    if let Ok(token_file) = gmail::account_token_file(&config, &opt.email) {
        let _ = std::fs::remove_file(token_file);
    }

    let mutations = MutationBuilder::new().delete_entity(format!("exomind_{}", opt.email));
    let _ = exm.store.mutate(mutations).await?;

    Ok(())
}

async fn list_accounts(config: cli::Config) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;
    let accounts = exm.get_accounts(true).await?;

    for account in accounts {
        println!("{:?}", account);
    }

    Ok(())
}

async fn start(config: cli::Config, opt: cli::StartOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;
    exm.create_base_objects().await?; // TODO: This shouldn't be here

    let accounts = exm.get_accounts(true).await?;
    let mut account_synchronizers = Vec::new();
    for account in accounts {
        account_synchronizers.push(AccountSynchronizer {
            account: account.clone(),
            gmail: GmailClient::new(&config, account).await?,
            last_gmail_history: None,
            last_exomind_operation: None,
        });
    }

    let exomind_inbox = exm
        .list_inbox_entities()
        .await?
        .into_iter()
        .flat_map(SynchronizedThread::from_exomind)
        .collect::<Vec<_>>();
    for sync in &mut account_synchronizers {
        info!("Initial inbox sync for account {}", sync.account.email());

        let threads = sync.gmail.list_inbox_threads(true).await?;
        if opt.save_fixtures {
            for thread in &threads {
                let path = format!("{}.new.json", thread.id.as_ref().unwrap());
                let mut f = std::fs::File::create(path)?;
                let json = serde_json::to_string_pretty(&thread)?;
                f.write_all(json.as_bytes())?;
            }
        }

        // import threads from gmail to exomind inbox
        let thread_entities = threads
            .into_iter()
            .flat_map(|th| SynchronizedThread::from_gmail(sync.account.clone(), th))
            .collect::<Vec<_>>();
        let mut gmail_threads = HashSet::new();
        for thread_entity in thread_entities {
            sync.update_last_gmail_history(thread_entity.history_id);
            gmail_threads.insert(thread_entity.thread_id().to_string());

            // TODO: Check if we need to import it really
            thread_entity.import_to_exomind(&exm).await?;
        }

        // move to inbox emails that are in exomind's inbox
        for thread in &exomind_inbox {
            if thread.account_entity_id == sync.account.entity_id {
                sync.update_last_exomind_operation(thread.operation_id);

                if !gmail_threads.contains(thread.thread_id()) {
                    let history_id = thread.add_to_gmail_inbox(&sync.gmail).await?;
                    sync.update_last_gmail_history(history_id);
                }
            }
        }
    }

    // TODO: Error handling. Shouldn't fail because one sync failed.
    // TODO: Try to parallelize accounts fetching. May have to duplicate handles.
    loop {
        for sync in &mut account_synchronizers {
            match sync.last_gmail_history {
                Some(last_history_id) => {
                    let history_list = sync.gmail.list_history(last_history_id).await?;
                    info!(
                        "Fetch {} history from gmail from history id {}",
                        history_list.len(),
                        last_history_id
                    );
                    for history in history_list {
                        match history {
                            GmailHistoryAction::AddToInbox(history_id, thread) => {
                                let thread_entity =
                                    SynchronizedThread::from_gmail(sync.account.clone(), thread);
                                if let Some(thread_entity) = thread_entity {
                                    // TODO: Fetch to make sure we aren't adding traits that already exist
                                    thread_entity.import_to_exomind(&exm).await?;
                                }

                                sync.update_last_gmail_history(Some(history_id));
                            }
                            GmailHistoryAction::RemoveFromInbox(history_id, thread_id) => {
                                exm.remove_from_inbox(&thread_id).await?;
                                sync.update_last_gmail_history(Some(history_id));
                            }
                        }
                    }

                    // TODO: Note "just added" threads
                    // TODO: Note "just removed" threads
                }
                None => {
                    error!("Require full inbox fetch");
                    // TODO: Aggregate full
                }
            }

            let last_operation_id = sync.last_exomind_operation.unwrap_or_default();
            // TODO: Should be fetched once, and filtering done externally so that we can do in a watched at some point
            let history_list = exm
                .list_inbox_history(&sync.account, last_operation_id)
                .await?;
            info!(
                "Fetch {} history from exomind after {:?}",
                history_list.len(),
                ConsistentTimestamp::from(last_operation_id).to_datetime(),
            );
            for history in history_list {
                match history {
                    ExomindHistoryAction::AddToInbox(thread) => {
                        thread.add_to_gmail_inbox(&sync.gmail).await?;
                        sync.update_last_exomind_operation(thread.operation_id);

                        // TODO: Note history ID to prevent adding in exomind
                    }
                    ExomindHistoryAction::RemovedFromInbox(thread) => {
                        thread.remove_from_gmail_inbox(&sync.gmail).await?;
                        sync.update_last_exomind_operation(thread.operation_id);

                        // TODO: Note history ID to prevent removing in exomind
                    }
                }
            }
        }

        delay_for(Duration::from_secs(10)).await;
    }
}

struct AccountSynchronizer {
    account: GmailAccount,
    gmail: GmailClient,
    last_gmail_history: Option<HistoryId>,
    last_exomind_operation: Option<OperationId>,
}

impl AccountSynchronizer {
    fn update_last_gmail_history(&mut self, history_id: Option<HistoryId>) {
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

    fn update_last_exomind_operation(&mut self, operation_id: Option<OperationId>) {
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
    account_entity_id: String,
    thread: EmailThread,
    emails: Vec<Email>,
    _inbox_child: Option<CollectionChild>,
    history_id: Option<HistoryId>,
    operation_id: Option<OperationId>,
}

impl SynchronizedThread {
    fn from_exomind(entity: Entity) -> Option<SynchronizedThread> {
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

        // TODO: Fix this, should be operation id
        let operation_id = if let Some(d) = entity.deletion_date {
            Some(d.to_timestamp_nanos())
        } else if let Some(d) = entity.modification_date {
            Some(d.to_timestamp_nanos())
        } else if let Some(d) = entity.creation_date {
            Some(d.to_timestamp_nanos())
        } else {
            None
        };

        Some(SynchronizedThread {
            account_entity_id,
            thread,
            emails,
            _inbox_child: inbox_child,
            history_id: None,
            operation_id,
        })
    }

    fn from_gmail(
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

    fn thread_id(&self) -> &str {
        &self.thread.source_id
    }

    async fn import_to_exomind(&self, exm: &ExomindClient) -> anyhow::Result<()> {
        // TODO: Should have a "current object" to compare and only import what is necessary
        exm.import_thread(self).await?;

        Ok(())
    }

    async fn add_to_gmail_inbox(&self, gmc: &GmailClient) -> anyhow::Result<Option<HistoryId>> {
        let thread = gmc.add_label(self.thread_id(), "INBOX".to_string()).await?;
        Ok(thread.history_id)
    }

    async fn remove_from_gmail_inbox(
        &self,
        gmc: &GmailClient,
    ) -> anyhow::Result<Option<HistoryId>> {
        let thread = gmc
            .remove_label(self.thread_id(), "INBOX".to_string())
            .await?;
        Ok(thread.history_id)
    }
}
