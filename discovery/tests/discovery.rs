use std::time::Duration;

use exocore_discovery::{client::Error, Client, Server, ServerConfig};

#[tokio::test(threaded_scheduler)]
async fn golden_path() -> anyhow::Result<()> {
    let config = ServerConfig {
        port: 3010,
        ..Default::default()
    };
    let server = Server::new(config);

    tokio::task::spawn(async move {
        server.start().await.unwrap();
    });

    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;

    let client = Client::new("http://127.0.0.1:3010")?;
    let create_resp = client.create(b"hello world").await?;

    let get_res = client.get(100_000_000).await;
    match get_res {
        Err(Error::NotFound) => {}
        Err(err) => panic!("Expected not found error, got {}", err),
        _ => panic!("Expected not found error, got a response"),
    }

    let get_res = client.get(create_resp.id).await?;
    assert_eq!(b"hello world", get_res.as_slice());

    Ok(())
}

#[tokio::test(threaded_scheduler)]
async fn payloads_expiration() -> anyhow::Result<()> {
    let config = ServerConfig {
        port: 3011,
        expiration: Duration::from_millis(100),
        cleanup_interval: Duration::from_millis(10),
        ..Default::default()
    };
    let server = Server::new(config);

    tokio::task::spawn(async move {
        server.start().await.unwrap();
    });

    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;

    let client = Client::new("http://127.0.0.1:3011")?;
    let _create_resp = client.create(b"hello world").await?;

    tokio::time::delay_for(std::time::Duration::from_millis(110)).await;

    let get_res = client.get(100_000_000).await;
    match get_res {
        Err(Error::NotFound) => {}
        Err(err) => panic!("Expected not found error, got {}", err),
        _ => panic!("Expected not found error, got a response"),
    }

    Ok(())
}
