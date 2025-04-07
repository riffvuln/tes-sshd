use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rustls::ClientConfig;
use tokio_rustls::TlsConnector;
use rustls::OwnedTrustAnchor;
use std::sync::Arc;
use webpki_roots::TLS_SERVER_ROOTS;

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up TLS configuration
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    
    let tls_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(tls_config));
    
    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Connect to the site via Tor (note port changed to 443)
    let stream = client
        .connect(("example.com", 443)).await?;
    
    // Set up TLS connection over Tor
    let domain = rustls::ServerName::try_from("example.com")
        .map_err(|_| anyhow::anyhow!("Invalid DNS name"))?;
    let mut stream = connector.connect(domain, stream).await?;
    
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
