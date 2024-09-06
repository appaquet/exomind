use std::{sync::Arc, time::Duration};

use exocore_core::{
    cell::{FullCell, LocalNode},
    futures::{sleep, spawn_future},
    tests_utils::{assert_equal_res, assert_res, async_expect_eventually},
    time::{ConsistentTimestamp, Instant},
};
use futures::{io::Cursor, AsyncRead, AsyncReadExt};

use super::*;
use crate::{
    testing::TestableTransportHandle, transport::ConnectionStatus, OutMessage, ServiceType,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_integration() -> anyhow::Result<()> {
    let n1 = LocalNode::generate();
    n1.add_p2p_address("/ip4/127.0.0.1/tcp/3003".parse()?);
    let n1_cell = FullCell::generate(n1.clone())?;

    let n2 = LocalNode::generate();
    n2.add_p2p_address("/ip4/127.0.0.1/tcp/3004".parse()?);
    let n2_cell = n1_cell.clone().with_local_node(n2.clone());

    n1_cell.cell().nodes_mut().add(n2.node().clone());
    n2_cell.cell().nodes_mut().add(n1.node().clone());

    let mut transport1 = Libp2pTransport::new(n1.clone(), Libp2pTransportConfig::default());
    let handle1 = transport1.get_handle(n1_cell.cell().clone(), ServiceType::Chain)?;
    let mut handle1 = TestableTransportHandle::new(handle1, n1_cell.cell().clone());
    spawn_future(async {
        let res = transport1.run().await;
        info!("Transport done: {:?}", res);
    });

    let mut transport2 = Libp2pTransport::new(n2.clone(), Libp2pTransportConfig::default());
    let handle2 = transport2.get_handle(n2_cell.cell().clone(), ServiceType::Chain)?;
    let mut handle2 = TestableTransportHandle::new(handle2, n2_cell.cell().clone());
    spawn_future(async {
        let res = transport2.run().await;
        info!("Transport done: {:?}", res);
    });

    // wait for nodes to be connected
    async_expect_eventually(|| async {
        assert_equal_res(
            handle1.node_status(n2.id()).await,
            Some(ConnectionStatus::Connected),
        )?;

        assert_equal_res(
            handle2.node_status(n1.id()).await,
            Some(ConnectionStatus::Connected),
        )?;

        Ok(())
    })
    .await;

    {
        // send 1 to 2
        handle1.send_rdv(n2.node().clone(), 123).await;
        let msg = handle2.recv_rdv(123).await;

        // reply to message
        let msg_frame = TestableTransportHandle::empty_message_frame();
        let reply_msg = msg.to_response_message(n1_cell.cell(), msg_frame)?;
        handle2.send_message(reply_msg).await;
        handle1.recv_rdv(123).await;

        // send 2 to 1, should expect receiving 1 new messages (so total 3 because of
        // prev reply)
        handle2.send_rdv(n1.node().clone(), 345).await;
        async_expect_eventually(|| async { assert_equal_res(handle1.received_count().await, 3) })
            .await;
    }

    {
        // send message with stream
        let stream = str_to_stream("hello world");
        handle1
            .send_stream_msg(n2.node().clone(), 124, stream)
            .await;

        let msg = handle2.recv_rdv(124).await;
        let mut out = String::new();
        msg.stream.unwrap().read_to_string(&mut out).await.unwrap();
        assert_eq!(out, "hello world");
    }

    Ok(())
}

fn str_to_stream(str: &'static str) -> Box<dyn AsyncRead + Send + Unpin> {
    Box::new(Cursor::new(str.to_string()))
}

#[tokio::test]
async fn handle_removal_and_transport_kill() -> anyhow::Result<()> {
    let n1 = LocalNode::generate();
    n1.add_p2p_address("/ip4/127.0.0.1/tcp/0".parse()?);
    let n1_cell = FullCell::generate(n1.clone())?;

    let n2 = LocalNode::generate();
    n2.add_p2p_address("/ip4/127.0.0.1/tcp/0".parse()?);
    let n2_cell = FullCell::generate(n2)?;

    let mut transport = Libp2pTransport::new(n1, Libp2pTransportConfig::default());
    let inner_weak = Arc::downgrade(&transport.get_service_handles());

    // we create 2 handles
    let handle1 = transport.get_handle(n1_cell.cell().clone(), ServiceType::Chain)?;
    let handle2 = transport.get_handle(n2_cell.cell().clone(), ServiceType::Chain)?;

    spawn_future(async {
        let res = transport.run().await;
        info!("Transport done: {:?}", res);
    });

    // we drop first handle, we expect inner to now contain its handle anymore
    drop(handle1);
    async_expect_eventually(|| async {
        let inner = inner_weak.upgrade().unwrap();
        let inner = inner.read().unwrap();
        assert_equal_res(inner.service_handles.len(), 1)
    })
    .await;

    // we drop second handle, we expect inner to be dropped and therefor transport
    // killed
    drop(handle2);
    async_expect_eventually(|| async { assert_res(inner_weak.upgrade().is_none()) }).await;

    Ok(())
}

#[tokio::test]
async fn should_queue_message_until_connected() -> anyhow::Result<()> {
    let n1 = LocalNode::generate();
    n1.add_p2p_address("/ip4/127.0.0.1/tcp/3005".parse()?);
    let n1_cell = FullCell::generate(n1.clone())?;

    let n2 = LocalNode::generate();
    n2.add_p2p_address("/ip4/127.0.0.1/tcp/3006".parse()?);
    let n2_cell = n1_cell.clone().with_local_node(n2.clone());

    n1_cell.cell().nodes_mut().add(n2.node().clone());
    n2_cell.cell().nodes_mut().add(n1.node().clone());

    let mut t2 = Libp2pTransport::new(n1, Libp2pTransportConfig::default());
    let h1 = t2.get_handle(n1_cell.cell().clone(), ServiceType::Chain)?;
    let mut h1 = TestableTransportHandle::new(h1, n1_cell.cell().clone());
    spawn_future(async {
        let res = t2.run().await;
        info!("Transport done: {:?}", res);
    });

    // send 1 to 2, but 2 is not yet connected. It should queue
    h1.send_rdv(n2.node().clone(), 1).await;

    // send 1 to 2, but with expired message, which shouldn't be delivered
    let msg_frame = TestableTransportHandle::empty_message_frame();
    let msg = OutMessage::from_framed_message(n1_cell.cell(), ServiceType::Chain, msg_frame)?
        .with_expiration(Instant::now().checked_sub(Duration::from_secs(5)))
        .with_rdv(ConsistentTimestamp(2))
        .with_destination(n2.node().clone());
    h1.send_message(msg).await;

    // leave some time for first messages to arrive
    std::thread::sleep(Duration::from_millis(100));

    // we create second node
    let mut t2 = Libp2pTransport::new(n2.clone(), Libp2pTransportConfig::default());
    let h2 = t2.get_handle(n2_cell.cell().clone(), ServiceType::Chain)?;
    let mut h2 = TestableTransportHandle::new(h2, n2_cell.cell().clone());
    spawn_future(async {
        let res = t2.run().await;
        info!("Transport done: {:?}", res);
    });

    // leave some time to start listening and connect
    sleep(Duration::from_millis(100)).await;

    // send another message to force redial
    h1.send_rdv(n2.node().clone(), 3).await;

    // should receive 1 & 3, but not 2 since it had expired
    h2.recv_rdv(1).await;
    h2.recv_rdv(3).await;
    assert!(!h2.has_msg().await?);

    Ok(())
}
