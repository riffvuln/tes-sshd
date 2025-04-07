use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up native TLS configuration
    let tls_connector = TlsConnector::new()?;
    let tls_connector = TokioTlsConnector::from(tls_connector);
    
    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Connect to the site via Tor (port 443 for HTTPS)
    let stream = client
        .connect(("example.com", 443)).await?;
    
    // Set up TLS connection over Tor
    let domain = "example.com";
    let mut stream = tls_connector.connect(domain, stream).await?;
    
    // Send HTTP request over TLS
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .await?;
    stream.flush().await?;
    
    // Read response
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    Ok(())
}
