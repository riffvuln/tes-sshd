use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

const DOMAIN: &'static str = "example.com";
const PORT: u16 = 443;

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up native TLS configuration
    let tls_conn = TlsConnector::new()?;
    let tls_conn = TokioTlsConnector::from(tls_conn);

    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Make stream to the target domain with tor
    let stream = client.connect((DOMAIN, PORT)).await?;

    // Wrap the stream with TLS
    let mut stream = tls_conn.connect(DOMAIN, stream).await?;

    // Send HTTP GET request
    let request = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", DOMAIN);
    stream.write_all(request.as_bytes()).await?;

    // Read response
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    Ok(())
}
