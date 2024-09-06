use std::time::Duration;

use exocore_discovery::{client::Error, Client, Server, ServerConfig};

#[tokio::test(flavor = "multi_thread")]
async fn create_and_get() -> anyhow::Result<()> {
    let config = ServerConfig {
        port: 3010,
        ..Default::default()
    };
    let server = Server::new(config);

    tokio::task::spawn(async move {
        server.start().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = Client::new("http://127.0.0.1:3010")?;
    let create_resp = client.create(b"hello world", false).await?;

    let get_res = client.get(100_000_000).await;
    match get_res {
        Err(Error::NotFound) => {}
        Err(err) => panic!("Expected not found error, got {}", err),
        _ => panic!("Expected not found error, got a response"),
    }

    let get_resp = client.get(create_resp.pin).await?;
    assert_eq!(
        b"hello world",
        get_resp.decode_payload().unwrap().as_slice()
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn create_and_reply_and_reply() -> anyhow::Result<()> {
    let config = ServerConfig {
        port: 3011,
        ..Default::default()
    };
    let server = Server::new(config);

    tokio::task::spawn(async move {
        server.start().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = Client::new("http://127.0.0.1:3011")?;

    // payload 1
    let create_resp = client.create(b"payload1", true).await?;
    assert!(create_resp.reply_pin.is_some());
    let get_resp = client.get(create_resp.pin).await?;
    assert_eq!(create_resp.reply_pin, get_resp.reply_pin);
    assert_eq!(b"payload1", get_resp.decode_payload().unwrap().as_slice());

    // payload 2 as reply of 1
    let reply_resp = client
        .reply(
            get_resp.reply_pin.unwrap(),
            get_resp.reply_token.unwrap(),
            b"payload2",
            true,
        )
        .await?;
    assert!(reply_resp.reply_pin.is_some());
    let get_resp = client.get(reply_resp.pin).await?;
    assert_eq!(reply_resp.reply_pin, get_resp.reply_pin);
    assert_eq!(b"payload2", get_resp.decode_payload().unwrap().as_slice());

    // reply to 2 with wrong token should fail
    let reply_resp2_err = client
        .reply(reply_resp.reply_pin.unwrap(), 1337, b"payload3", false)
        .await;
    assert!(reply_resp2_err.is_err());

    // payload 3 as reply of 2
    let reply_resp2 = client
        .reply(
            get_resp.reply_pin.unwrap(),
            get_resp.reply_token.unwrap(),
            b"payload3",
            false,
        )
        .await
        .unwrap();
    assert!(reply_resp2.reply_pin.is_none());

    let get_resp = client.get(reply_resp2.pin).await?;
    assert_eq!(b"payload3", get_resp.decode_payload().unwrap().as_slice());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn payloads_expiration() -> anyhow::Result<()> {
    let config = ServerConfig {
        port: 3012,
        payload_expiration: Duration::from_millis(100),
        cleanup_interval: Duration::from_millis(10),
        ..Default::default()
    };
    let server = Server::new(config);

    tokio::task::spawn(async move {
        server.start().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = Client::new("http://127.0.0.1:3012")?;
    let _create_resp = client.create(b"hello world", false).await?;

    tokio::time::sleep(std::time::Duration::from_millis(110)).await;

    let get_resp = client.get(100_000_000).await;
    match get_resp {
        Err(Error::NotFound) => {}
        Err(err) => panic!("Expected not found error, got {}", err),
        _ => panic!("Expected not found error, got a response"),
    }

    Ok(())
}
