#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[macro_use]
extern crate log;

mod logging;

use exocore_common::cell::Cell;
use exocore_common::crypto::keys::{Keypair, PublicKey};
use exocore_common::node::{LocalNode, Node};
use exocore_common::time::Clock;
use exocore_index::domain::schema::Schema;
use exocore_index::domain::serialization::with_schema;
use exocore_index::query::Query;
use exocore_index::store::remote::{RemoteStore, StoreConfiguration, StoreHandle};
use exocore_index::store::AsyncStore;
use exocore_transport::lp2p::Libp2pTransportConfig;
use exocore_transport::{Libp2pTransport, TransportLayer};
use futures::prelude::*;
use libc;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[repr(u8)]
enum Error {
    Success = 0,
}

pub struct Context {
    runtime: Runtime,
    store_handle: StoreHandle,
    schema: Arc<Schema>,
}

impl Context {
    fn new() -> Result<Context, Error> {
        logging::setup(None);
        info!("Initializing...");

        let mut runtime = Runtime::new().expect("Couldn't start runtime");

        // TODO: To be cleaned up when cell management will be ironed out: https://github.com/appaquet/exocore/issues/80
        let local_node = LocalNode::new_from_keypair(Keypair::decode_base58_string("ae4WbDdfhv3416xs8S2tQgczBarmR8HKABvPCmRcNMujdVpDzuCJVQADVeqkqwvDmqYUUjLqv7kcChyCYn8R9BNgXP").unwrap());
        let local_addr = "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("Couldn't parse local node");
        local_node.add_address(local_addr);

        let transport_config = Libp2pTransportConfig::default();
        let mut transport = Libp2pTransport::new(local_node.clone(), transport_config);

        let cell_pk =
            PublicKey::decode_base58_string("pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ")
                .expect("Couldn't decode cell publickey");
        let cell = Cell::new(cell_pk, local_node.clone());
        let clock = Clock::new();
        let schema = create_test_schema();

        let remote_node_pk =
            PublicKey::decode_base58_string("peFdPsQsdqzT2H6cPd3WdU1fGdATDmavh4C17VWWacZTMP")
                .expect("Couldn't decode cell publickey");
        let remote_node = Node::new_from_public_key(remote_node_pk);
        let remote_addr = "/ip4/192.168.2.67/tcp/3330"
            .parse()
            .expect("Couldn't parse remote node addr");
        remote_node.add_address(remote_addr);
        {
            cell.nodes_mut().add(remote_node.clone());
        }

        let store_transport = transport
            .get_handle(cell.clone(), TransportLayer::Index)
            .expect("Couldn't get transport handle for remote index");
        let remote_store_config = StoreConfiguration::default();
        let remote_store = RemoteStore::new(
            remote_store_config,
            cell,
            clock,
            schema.clone(),
            store_transport,
            remote_node,
            Box::new(tokio_future_spawner),
        )
        .expect("Couldn't start remote store");

        let store_handle = remote_store
            .get_handle()
            .expect("Couldn't get store handle");

        runtime.spawn(
            transport
                .map(|_| {
                    error!("Transport is done");
                })
                .map_err(|err| {
                    error!("Error in transport: {}", err);
                }),
        );

        runtime.spawn(
            remote_store
                .map(|_| {
                    error!("Remote store is done");
                })
                .map_err(|err| {
                    error!("Error in remote store: {}", err);
                }),
        );

        Ok(Context {
            runtime,
            schema,
            store_handle,
        })
    }
}

#[repr(C)]
pub struct ContextResult {
    status: Error,
    context: *mut Context,
}

#[no_mangle]
pub extern "C" fn exocore_context_new() -> ContextResult {
    let context = match Context::new() {
        Ok(context) => context,
        Err(err) => {
            return ContextResult {
                status: err,
                context: std::ptr::null_mut(),
            }
        }
    };

    ContextResult {
        status: Error::Success,
        context: Box::into_raw(Box::new(context)),
    }
}

#[no_mangle]
pub extern "C" fn exocore_send_query(ctx: *mut Context, _query: *const libc::c_char) {
    let context = unsafe { ctx.as_mut().unwrap() };

    match context
        .runtime
        .block_on(context.store_handle.query(Query::match_text("hello")))
    {
        Ok(res) => {
            let json = with_schema(&context.schema, || serde_json::to_string(&res)).unwrap();
            println!("got results: {:?}", json);
        }
        Err(err) => println!("got err: {}", err),
    }
}

// TODO: To be cleaned up in https://github.com/appaquet/exocore/issues/104
fn create_test_schema() -> Arc<Schema> {
    Arc::new(
        Schema::parse(
            r#"
        name: myschema
        traits:
            - id: 0
              name: contact
              fields:
                - id: 0
                  name: name
                  type: string
                  indexed: true
                - id: 1
                  name: email
                  type: string
                  indexed: true
            - id: 1
              name: email
              fields:
                - id: 0
                  name: subject
                  type: string
                  indexed: true
                - id: 1
                  name: body
                  type: string
                  indexed: true
        "#,
        )
        .unwrap(),
    )
}

// TODO: To be moved https://github.com/appaquet/exocore/issues/123
fn tokio_future_spawner(future: Box<dyn Future<Item = (), Error = ()> + Send>) {
    tokio::spawn(future);
}
