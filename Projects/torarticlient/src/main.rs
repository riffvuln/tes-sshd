use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    let mut stream = client
        .connect(("example.com", 443)).await?;
    stream
    .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
    .await?;
    stream.flush().await?;
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    Ok(())
}
