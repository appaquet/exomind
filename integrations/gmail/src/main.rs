use exocore::core::cell::Cell;
use exocore::core::futures::spawn_future;
use exocore::core::protos::index::{Reference, Trait};
use exocore::core::protos::prost::ProstAnyPackMessageExt;
use exocore::core::time::Clock;
use exocore::index::mutation::MutationBuilder;
use exocore::index::remote::{Client, ClientHandle};
use exocore::transport::{Libp2pTransport, TransportLayer};
use exomind;
use exomind::protos::base::{Account, Collection, CollectionChild, Email, EmailThread};
use log::LevelFilter;
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use exocore::transport::TransportHandle;

mod parsing;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

#[tokio::main]
async fn main() {
    exocore::core::logging::setup(None);

    // TODO: Save tokens into account by persisting to a temp file and save to exocore when changed
    let account = Account {
        key: "appaquet@gmail.com".to_string(),
        name: "Personal".to_string(),
        ..Default::default()
    };

    let gmail_client = new_gmail_client().await;
    let exocore_client = new_exocore_client().await;

    {
        let inbox_trait = Trait {
            id: "inbox".to_string(),
            message: Some(
                Collection {
                    name: "Inbox".to_string(),
                    ..Default::default()
                }
                .pack_to_any()
                .unwrap(),
            ),
            ..Default::default()
        };
        let mutation = MutationBuilder::new().put_trait("inbox".to_string(), inbox_trait);
        let _ = exocore_client.mutate(mutation).await.unwrap();
    }

    let list: google_gmail1::schemas::ListThreadsResponse = gmail_client
        .users()
        .threads()
        .list("me")
        .label_ids("INBOX".to_string())
        .execute()
        .unwrap();

    let threads = list.threads.unwrap_or_else(|| Vec::new());
    for partial_thread in threads {
        let thread_id = partial_thread.id.unwrap().clone();
        let thread: google_gmail1::schemas::Thread = gmail_client
            .users()
            .threads()
            .get("me", thread_id.clone())
            .execute()
            .unwrap();

        // {
        //     let path = format!(
        //         "integrations/gmail/fixtures/threads/{}.new.json",
        //         thread.id.as_ref().unwrap()
        //     );
        //     let mut f = std::fs::File::create(path).unwrap();
        //     let json = serde_json::to_string_pretty(&thread).unwrap();
        //     f.write_all(json.as_bytes()).unwrap();
        // }

        let parsing::ParsedThread {
            mut thread,
            mut emails,
            labels,
        } = parsing::parse_thread(thread).unwrap();

        let thread_entity_id = thread_entity_id(&thread);

        thread.source = Some(account.clone());
        for email in &mut emails {
            email.source = Some(account.clone());
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

        {
            let thread_trait = Trait {
                id: thread_entity_id.clone(),
                message: Some(thread.pack_to_any().unwrap()),
                creation_date: thread_create_date,
                modification_date: thread_modification_date,
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), thread_trait);
            let _ = exocore_client.mutate(mutation).await.unwrap();
        }

        for email in emails.into_iter() {
            let creation_date = email.received_date.clone();
            let email_trait = Trait {
                id: email_trait_id(&email),
                message: Some(email.pack_to_any().unwrap()),
                creation_date,
                modification_date: None,
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), email_trait);
            let _ = exocore_client.mutate(mutation).await.unwrap();
        }

        {
            let child_trait = Trait {
                id: format!("child_{}", thread_entity_id),
                message: Some(
                    CollectionChild {
                        collection: Some(Reference {
                            entity_id: "inbox".to_string(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                ..Default::default()
            };
            let mutation = MutationBuilder::new().put_trait(thread_entity_id.clone(), child_trait);
            let _ = exocore_client.mutate(mutation).await.unwrap();
        }
    }
}

async fn new_exocore_client() -> ClientHandle {
    let config = exocore::core::cell::node_config_from_yaml_file("local_conf/config.yaml").unwrap();
    let (cells, local_node) = Cell::new_from_local_node_config(config).unwrap();
    let either_cell = cells.first().unwrap();
    let cell = either_cell.cell();

    let clock = Clock::new();

    let mut transport = Libp2pTransport::new(local_node.clone(), Default::default());
    let index_transport = transport
        .get_handle(cell.clone(), TransportLayer::Index)
        .unwrap();

    spawn_future(async move {
        let res = transport.run().await;
        info!("Transport done: {:?}", res);
    });

    let store_client =
        Client::new(Default::default(), cell.clone(), clock, index_transport).unwrap();
    let store_handle = store_client.get_handle();

    spawn_future(async move {
        let res = store_client.run().await;
        info!("Remote client done: {:?}", res);
    });

    store_handle.on_start().await;

    store_handle
}

async fn new_gmail_client() -> google_gmail1::Client {
    let secret = yup_oauth2::read_application_secret("local_conf/client_secret.json")
        .await
        .unwrap();

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .persist_tokens_to_disk("local_conf/tokencache.json")
        .build()
        .await
        .unwrap();

    let scopes = &["https://mail.google.com/"];
    let token = auth.token(scopes).await.unwrap();

    google_gmail1::Client::new(YupAuth { token })
}

fn thread_entity_id(thread: &EmailThread) -> String {
    format!("bgt{}", thread.source_id)
}

fn email_trait_id(email: &Email) -> String {
    format!("bge{}", email.source_id)
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
