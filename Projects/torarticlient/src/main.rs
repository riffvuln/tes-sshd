use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use std::time::Duration;

// You can change this for testing
const DOMAIN: &'static str = "erhrkm6c4qzohi74yu7gb3bjgywecs3m6jyaok2hsw5qbozhv6ck32ad.onion";
const PORT: u16 = 443;
const PATH: &'static str = "";

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up native TLS configuration
    let tls_conn = TlsConnector::new()?;
    let tls_conn = TokioTlsConnector::from(tls_conn);

    // Set up Tor client with adjusted timeouts
    let mut cfg = TorClientConfig::default();
    
    println!("Bootstrapping Tor client...");
    let client = TorClient::create_bootstrapped(cfg).await?;
    println!("Tor client bootstrapped successfully");
    
    // Make stream to the target domain with tor
    println!("Connecting to {DOMAIN}:{PORT}...");
    let stream = match client.connect((DOMAIN, PORT)).await {
        Ok(s) => {
            println!("Connected to onion service successfully");
            s
        },
        Err(e) => {
            println!("Failed to connect to onion service: {}", e);
            return Err(e.into());
        }
    };

    // Wrap the stream with TLS
    println!("Establishing TLS connection...");
    let mut stream = match tls_conn.connect(DOMAIN, stream).await {
        Ok(s) => {
            println!("TLS connection established");
            s
        },
        Err(e) => {
            println!("TLS connection failed: {}", e);
            return Err(e.into());
        }
    };

    // Send HTTP GET request
    let request = format!(
        "GET /{PATH} HTTP/1.1\r\n\
        Host: {DOMAIN}\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
        Accept-Language: en-US,en;q=0.9\r\n\
        Connection: close\r\n\
        \r\n"
    );
    
    println!("Sending HTTP request...");
    stream.write_all(request.as_bytes()).await?;

    // Flush the stream to ensure the request is sent
    stream.flush().await?;
    println!("Request sent, waiting for response...");

    // Read response with timeout
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    
    loop {
        match tokio::time::timeout(Duration::from_secs(30), stream.read(&mut chunk)).await {
            Ok(Ok(0)) => break, // End of stream
            Ok(Ok(n)) => {
                println!("Received {} bytes", n);
                buffer.extend_from_slice(&chunk[..n]);
            },
            Ok(Err(e)) => {
                println!("Error reading from stream: {}", e);
                return Err(e.into());
            },
            Err(_) => {
                println!("Timeout while reading response");
                break;
            }
        }
    }
    
    if buffer.is_empty() {
        println!("Received empty response");
    } else {
        println!("Response length: {} bytes", buffer.len());
        println!("Response headers:");
        
        // Print headers for debugging
        if let Some(headers_end) = memmem::find(&buffer, b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&buffer[..headers_end]);
            println!("{}", headers);
            
            // Print a small preview of the body
            let body = &buffer[headers_end + 4..];
            let preview_size = std::cmp::min(200, body.len());
            println!("\nBody preview (first {} bytes):", preview_size);
            println!("{}", String::from_utf8_lossy(&body[..preview_size]));
        } else {
            println!("Could not find end of headers");
            println!("Raw response: {}", String::from_utf8_lossy(&buffer));
        }
    }
    
    Ok(())
}