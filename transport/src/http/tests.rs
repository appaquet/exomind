use exocore_core::{
    cell::{FullCell, LocalNode},
    framing::CapnpFrameBuilder,
    futures::spawn_future,
    protos::generated::store_transport_capnp::mutation_request,
    protos::generated::store_transport_capnp::mutation_response,
    protos::generated::store_transport_capnp::query_request,
    protos::generated::store_transport_capnp::query_response,
    sec::auth_token::AuthToken,
    time::Clock,
};
use hyper::{body::Buf, Body, Client, Request, Response, StatusCode};

use crate::{testing::TestableTransportHandle, ServiceType, TransportServiceHandle};

use super::*;

#[tokio::test]
async fn invalid_requests() -> anyhow::Result<()> {
    let node = LocalNode::generate();
    let cell = FullCell::generate(node.clone());
    let clock = Clock::new();

    let _entities_handle = start_server(&cell, &clock, 3007).await;

    {
        // invalid authentication token
        let url = "http://127.0.0.1:3007/entities/query?token=invalid_token";
        let resp_chan = send_http_request(url, b"query body");
        let resp = resp_chan.await??;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    {
        // valid token, but invalid signature
        let auth_token = {
            let auth_token = AuthToken::new(cell.cell(), &clock, None)?;
            let mut auth_token_proto = auth_token.as_proto().clone();
            auth_token_proto.signature = vec![1, 3, 3, 7];
            let auth_token = AuthToken::from_proto(auth_token_proto)?;
            auth_token.encode_base58_string()
        };

        let url = format!("http://127.0.0.1:3007/entities/query?token={}", auth_token);
        let resp_chan = send_http_request(url, b"query body");
        let resp = resp_chan.await??;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    {
        // invalid cell
        let cell = FullCell::generate(node.clone());
        let auth_token = AuthToken::new(cell.cell(), &clock, None)?;
        let auth_token = auth_token.encode_base58_string();
        let url = format!("http://127.0.0.1:3007/entities/query?token={}", auth_token);
        let resp_chan = send_http_request(url, b"query body");
        let resp = resp_chan.await??;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    {
        // invalid request type
        let auth_token = AuthToken::new(cell.cell(), &clock, None)?;
        let auth_token = auth_token.encode_base58_string();
        let url = format!("http://127.0.0.1:3007/invalid/type?token={}", auth_token);
        let resp_chan = send_http_request(url, b"query body");
        let resp = resp_chan.await??;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[tokio::test]
async fn entities_query() -> anyhow::Result<()> {
    let node = LocalNode::generate();
    let cell = FullCell::generate(node.clone());
    let clock = Clock::new();

    let auth_token = AuthToken::new(cell.cell(), &clock, None)?;
    let auth_token = auth_token.encode_base58_string();

    let mut entities_handle = start_server(&cell, &clock, 3008).await;

    let url = format!("http://127.0.0.1:3008/entities/query?token={}", auth_token);
    let resp_chan = send_http_request(url, b"query");

    entities_receive_response_query(&mut entities_handle, b"query", b"response").await?;

    let resp_body = resp_chan.await??;
    let body = hyper::body::aggregate(resp_body).await?;
    assert_eq!(body.bytes(), b"response");

    Ok(())
}

#[tokio::test]
async fn entities_mutation() -> anyhow::Result<()> {
    let node = LocalNode::generate();
    let cell = FullCell::generate(node.clone());
    let clock = Clock::new();

    let auth_token = AuthToken::new(cell.cell(), &clock, None)?;
    let auth_token = auth_token.encode_base58_string();

    let mut entities_handle = start_server(&cell, &clock, 3009).await;

    let url = format!("http://127.0.0.1:3009/entities/mutate?token={}", auth_token);
    let resp_chan = send_http_request(url, b"mutation");

    {
        let query_request = entities_handle.recv_msg().await;
        let query_frame = query_request.get_data_as_framed_message::<mutation_request::Owned>()?;
        let query_reader = query_frame.get_reader()?;
        let query_body = query_reader.get_request()?;
        assert_eq!(query_body, b"mutation");

        let mut frame_builder = CapnpFrameBuilder::<mutation_response::Owned>::new();
        let mut b: mutation_response::Builder = frame_builder.get_builder();
        b.set_response(b"response");

        let resp_msg = query_request.to_response_message(entities_handle.cell(), frame_builder)?;
        entities_handle.send_message(resp_msg).await;
    }

    let resp_body = resp_chan.await??;
    let body = hyper::body::aggregate(resp_body).await?;
    assert_eq!(body.bytes(), b"response");

    Ok(())
}

async fn start_server(cell: &FullCell, clock: &Clock, port: u16) -> TestableTransportHandle {
    let listen_addr = format!("http://127.0.0.1:{}", port);

    let config = HTTPTransportConfig {
        listen_addresses: vec![listen_addr.parse().unwrap()],
        ..Default::default()
    };

    let mut server = HTTPTransportServer::new(cell.local_node().clone(), config, clock.clone());
    let handle = server
        .get_handle(cell.cell().clone(), ServiceType::Store)
        .unwrap();

    spawn_future(async move {
        server.run().await.unwrap();
    });

    handle.on_started().await;

    TestableTransportHandle::new(handle, cell.cell().clone())
}

fn send_http_request<T: Into<String>>(
    url: T,
    body: &[u8],
) -> futures::channel::oneshot::Receiver<Result<Response<Body>, hyper::Error>> {
    let req = Request::builder()
        .method("POST")
        .uri(url.into())
        .body(Body::from(body.to_vec()))
        .unwrap();

    let (req_sender, req_recv) = futures::channel::oneshot::channel();
    spawn_future(async move {
        let http_client = Client::new();
        let resp = http_client.request(req).await;
        req_sender.send(resp).unwrap();
    });

    req_recv
}

async fn entities_receive_response_query(
    handle: &mut TestableTransportHandle,
    expected_query_data: &[u8],
    result_data: &[u8],
) -> anyhow::Result<()> {
    let query_request = handle.recv_msg().await;
    let query_frame = query_request.get_data_as_framed_message::<query_request::Owned>()?;
    let query_reader = query_frame.get_reader()?;
    let query_body = query_reader.get_request()?;
    assert_eq!(query_body, expected_query_data);

    let mut frame_builder = CapnpFrameBuilder::<query_response::Owned>::new();
    let mut b: query_response::Builder = frame_builder.get_builder();
    b.set_response(result_data);

    let resp_msg = query_request.to_response_message(handle.cell(), frame_builder)?;
    handle.send_message(resp_msg).await;

    Ok(())
}
