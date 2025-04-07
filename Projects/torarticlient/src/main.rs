use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

const DOMAIN: &'static str = "httpbin.io";
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
    let request = format!(
        "GET /headers HTTP/1.1\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
        Accept-Encoding: gzip, deflate, br, zstd\r\n\
        Accept-Language: en-US,en;q=0.5\r\n\
        Connection: keep-alive\r\n\
        Dnt: 1\r\n\
        Host: {DOMAIN}\r\n\
        Priority: u=0, i\r\n\
        Sec-Fetch-Dest: document\r\n\
        Sec-Fetch-Mode: navigate\r\n\
        Sec-Fetch-Site: cross-site\r\n\
        Sec-Fetch-User: ?1\r\n\
        Sec-Gpc: 1\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/136.0\r\n\
        \r\n"
    );
    stream.write_all(request.as_bytes()).await?;

    // Flush the stream to ensure the request is sent
    stream.flush().await?;

    // Read response
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    Ok(())
}
