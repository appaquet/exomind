use exocore::core::protos::index::{Reference, Trait};
use exocore::core::protos::prost::{ProstAnyPackMessageExt, ProstTimestampExt};
use exocore::core::time::Utc;
use exocore::{
    index::{entity::EntityExt, mutation::MutationBuilder},
    protos::index::Entity,
};
use exomind::{email_trait_id, thread_entity_id, ExomindClient};
use exomind_core::protos::base::{CollectionChild, Email, EmailThread};
use gmail::{GmailAccount, GmailClient, HistoryAction, HistoryId};
use log::LevelFilter;
use std::{collections::HashSet, str::FromStr, time::Duration};
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

// TODO:
//  if opt.save_fixtures {
//             let path = format!("{}.new.json", thread.id.as_ref().unwrap());
//             let mut f = std::fs::File::create(path)?;
//             let json = serde_json::to_string_pretty(&thread)?;
//             f.write_all(json.as_bytes())?;
//         }

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

struct AccountSyncer {
    account: GmailAccount,
    last_history_id: Option<HistoryId>,
    client: GmailClient,
}

impl AccountSyncer {
    fn update_history_id(&mut self, history_id: Option<HistoryId>) {
        let history_id = if let Some(history_id) = history_id {
            history_id
        } else {
            return;
        };

        match self.last_history_id {
            Some(last_history_id) if last_history_id < history_id => {
                self.last_history_id = Some(history_id);
            }
            None => {
                self.last_history_id = Some(history_id);
            }
            _ => {}
        }
    }
}

async fn start(config: cli::Config, opt: cli::StartOptions) -> anyhow::Result<()> {
    let exm = ExomindClient::new(&config).await?;
    exm.create_base_objects().await?; // TODO: This shouldn't be here

    let accounts = exm.get_accounts(true).await?;
    let mut account_synchronizers = Vec::new();
    for account in accounts {
        account_synchronizers.push(AccountSyncer {
            account: account.clone(),
            client: GmailClient::new(&config, account).await?,
            last_history_id: None,
        });
    }

    let mut last_exomind_check = Utc::now();

    let exomind_inbox = exm
        .list_inbox_entities()
        .await?
        .into_iter()
        .flat_map(ThreadEntity::from_exomind)
        .collect::<Vec<_>>();
    for sync in &mut account_synchronizers {
        info!("Initial inbox sync for account {}", sync.account.email());

        // import threads from gmail to exomind inbox
        let threads = sync.client.list_inbox_threads(true).await?;
        let thread_entities = threads
            .into_iter()
            .flat_map(|th| ThreadEntity::from_gmail(sync.account.clone(), th))
            .collect::<Vec<_>>();
        let mut gmail_threads = HashSet::new();
        for thread_entity in thread_entities {
            sync.update_history_id(thread_entity.history_id);
            gmail_threads.insert(thread_entity.thread_id().to_string());
            thread_entity.import_to_exomind(&exm).await?;
        }

        // move to inbox emails that are in exomind's inbox
        for thread in &exomind_inbox {
            if thread.account_entity_id == sync.account.entity_id
                && !gmail_threads.contains(thread.thread_id())
            {
                let history_id = thread.add_to_gmail_inbox(&sync.client).await?;
                sync.update_history_id(history_id);
            }
        }
    }

    loop {
        for sync in &mut account_synchronizers {
            match sync.last_history_id {
                Some(last_history_id) => {
                    let history_list = sync.client.list_history(last_history_id).await?;
                    info!("Fetch {} history from gmail", history_list.len());
                    for history in history_list {
                        match history {
                            HistoryAction::AddToInbox(history_id, thread) => {
                                let thread_entity =
                                    ThreadEntity::from_gmail(sync.account.clone(), thread);
                                if let Some(thread_entity) = thread_entity {
                                    thread_entity.import_to_exomind(&exm).await?;
                                }

                                sync.update_history_id(Some(history_id));
                            }
                            HistoryAction::RemoveFromInbox(history_id, thread_id) => {
                                exm.remove_from_inbox(&thread_id).await?;
                                sync.update_history_id(Some(history_id));
                            }
                        }
                    }
                }
                None => {
                    error!("Require full inbox fetch");
                    // TODO: Aggregate full
                }
            }
        }

        // TODO: Aggregate history from Gmail. If no history, check inbox and insert all.
        // TODO: Ignore all history from last round
        // TODO: Note "just added" threads
        // TODO: Note "just removed" threads

        // TODO: Fetch history from Exomind. Note addition or removal of CollectionChild since last check
        // TODO: Don't add back to gmail threads that are in "just added"
        // TODO: Don't remove from gmail threads that are in "just removed"

        // TODO: Wipe "just added" / "just removed"
        // TODO: Add / remove from gmail
        // TODO: Note history ids of modified

        delay_for(Duration::from_secs(5)).await;
    }

    // loop {
    //     for account in &accounts {
    //         println!("Syncing account {}", account.email());

    //         let account_inbox_tracker = inbox_tracker.for_account(&account);

    //         let inbox_entities = exm.list_inbox_entities().await?;

    //         let inbox_threads = TrackedThread::from_entities(&inbox_entities)
    //             .into_iter()
    //             .filter(|thread| thread.account_entity_id == account.entity_id)
    //             .collect::<Vec<_>>();
    //         println!("In inbox for account: {:?}", inbox_threads);

    //         let gmc = GmailClient::new(&config, account.clone()).await?;
    //         let threads = gmc.list_inbox_threads().await?;
    //     }

    //     delay_for(Duration::from_secs(5)).await;
    // }
}

pub struct ThreadEntity {
    account_entity_id: String,
    thread: EmailThread,
    emails: Vec<Email>,
    inbox_child: Option<CollectionChild>,
    history_id: Option<HistoryId>,
}

impl ThreadEntity {
    fn from_exomind(entity: Entity) -> Option<ThreadEntity> {
        let thread: EmailThread = entity.trait_of_type::<EmailThread>()?;
        let account_entity_id = thread.account.as_ref()?.entity_id.clone();

        let inbox_child = entity
            .traits_of_type::<CollectionChild>()
            .into_iter()
            .find(|c| c.collection.as_ref().map(|c| c.entity_id.as_ref()) == Some("inbox"));

        let emails: Vec<Email> = entity.traits_of_type::<Email>();

        Some(ThreadEntity {
            account_entity_id,
            thread,
            emails,
            inbox_child,
            history_id: None,
        })
    }

    fn from_gmail(
        account: GmailAccount,
        thread: google_gmail1::schemas::Thread,
    ) -> Option<ThreadEntity> {
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

        Some(ThreadEntity {
            account_entity_id: account.entity_id.clone(),
            thread,
            emails,
            inbox_child,
            history_id,
        })
    }

    fn thread_id(&self) -> &str {
        &self.thread.source_id
    }

    async fn import_to_exomind(&self, exm: &ExomindClient) -> anyhow::Result<()> {
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
