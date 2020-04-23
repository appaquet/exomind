use exocore::core::cell::Cell;
use exocore::core::futures::spawn_future;
use exocore::core::protos::index::Trait;
use exocore::core::protos::prost::ProstAnyPackMessageExt;
use exocore::core::time::Clock;
use exocore::index::mutation::MutationBuilder;
use exocore::index::remote::{Client, ClientHandle};
use exocore::transport::{Libp2pTransport, TransportLayer};
use exomind;
use log::LevelFilter;
use std::io::Write;
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

mod parsing;

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

#[tokio::main]
async fn main() {
    exocore::core::logging::setup(Some(LevelFilter::Info));

    let gmail_client = new_gmail_client().await;
    let exocore_client = new_exocore_client().await;

    // let list: google_gmail1::schemas::ListLabelsResponse =
    //     gmail.users().labels().list("me").execute().unwrap();
    // println!("{:?}", list);

    // TODO: Check for lettre https://github.com/lettre/lettre/pull/282/files
    //             and https://crates.io/crates/mailparse

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
        let thread_snippet = partial_thread.snippet.unwrap_or_default().clone();

        let thread: google_gmail1::schemas::Thread = gmail_client
            .users()
            .threads()
            .get("me", thread_id.clone())
            .execute()
            .unwrap();

        {
            let path = format!(
                "integrations/gmail/fixtures/threads/{}.new.json",
                thread.id.as_ref().unwrap()
            );
            let mut f = std::fs::File::create(path).unwrap();
            let json = serde_json::to_string_pretty(&thread).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }

        let _parsed_thread = parsing::parse_thread(thread).unwrap();


        // parsed_thread.thread.last_email = Some(Reference {
        //     entity_id:
        // })
        // TODO: thread.source + emails.source

        // let mut thread_proto = exomind::protos::base::EmailThread {
        //     source: None,
        //     source_id: String::new(),
        //     from: None,
        //     subject: String::new(),
        //     snippet: thread_snippet,
        //     last_email: None,
        // };
        // let trt = Trait {
        //     id: thread_id.clone(),
        //     creation_date: None,
        //     modification_date: None,
        //     message: Some(thread_proto.pack_to_any().unwrap()),
        // };
        //
        // let mutation = MutationBuilder::put_trait(thread_id, trt);
        // let _ = exocore_client.mutate(mutation).await.unwrap();
    }
}

async fn new_exocore_client() -> ClientHandle {
    let config = exocore::core::cell::node_config_from_yaml_file("config.yaml").unwrap();
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
        info!("Remove client done: {:?}", res);
    });

    store_handle
}

async fn new_gmail_client() -> google_gmail1::Client {
    let secret = yup_oauth2::read_application_secret("client_secret.json")
        .await
        .unwrap();

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();

    let scopes = &["https://mail.google.com/"];
    let token = auth.token(scopes).await.unwrap();

    google_gmail1::Client::new(YupAuth { token })
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
