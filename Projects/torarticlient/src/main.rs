use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::Protocol;
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use flate2::read::GzDecoder;
use std::io::Read;
use std::cmp::min;

const DOMAIN: &'static str = "myinstafollow.com";
const PORT: u16 = 443;
const PATH: &'static str = "free-tiktok-views";

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up enhanced TLS configuration to mimic Firefox
    let mut tls_builder = native_tls::TlsConnector::builder();
    
    // Configure TLS for browser-like behavior
    tls_builder
        .min_protocol_version(Some(Protocol::Tlsv12))
        .use_sni(true)
        // Disabling ALPN for now as it might be causing HTTP/2 negotiation issues
        // .request_alpns(&["h2", "http/1.1"])
        ;
    
    let tls_conn = tls_builder.build()?;
    let tls_conn = TokioTlsConnector::from(tls_conn);

    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Make stream to the target domain with tor
    let stream = client.connect((DOMAIN, PORT)).await?;

    println!("Connected to target through Tor");

    // Wrap the stream with TLS
    let mut stream = tls_conn.connect(DOMAIN, stream).await?;
    
    println!("TLS connection established");

    // Send HTTP GET request with enhanced headers
    let request = format!(
        "GET /{PATH} HTTP/1.1\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8\r\n\
        Accept-Encoding: identity\r\n\
        Accept-Language: en-US,en;q=0.9\r\n\
        Connection: close\r\n\
        Host: {DOMAIN}\r\n\
        Sec-Ch-Ua: \"Chromium\";v=\"116\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"116\"\r\n\
        Sec-Ch-Ua-Mobile: ?0\r\n\
        Sec-Ch-Ua-Platform: \"Windows\"\r\n\
        Sec-Fetch-Dest: document\r\n\
        Sec-Fetch-Mode: navigate\r\n\
        Sec-Fetch-Site: none\r\n\
        Sec-Fetch-User: ?1\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36\r\n\
        Cache-Control: max-age=0\r\n\
        DNT: 1\r\n\
        \r\n"
    );
    stream.write_all(request.as_bytes()).await?;

    // Flush the stream to ensure the request is sent
    stream.flush().await?;
    
    println!("Request sent");

    // Read response in chunks to better analyze what we're receiving
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    
    println!("Reading response in chunks:");
    let mut total_bytes = 0;
    
    loop {
        match stream.read(&mut chunk).await {
            Ok(0) => {
                println!("End of stream reached");
                break;
            },
            Ok(n) => {
                println!("Read {} bytes", n);
                total_bytes += n;
                
                // Print first few bytes of this chunk
                println!("Chunk hex preview:");
                for byte in chunk.iter().take(min(n, 32)) {
                    print!("{:02X} ", byte);
                }
                println!();
                
                // Check if chunk starts with HTTP
                if n >= 4 && &chunk[0..4] == b"HTTP" {
                    println!("Chunk appears to be HTTP");
                }
                
                // Add to our buffer
                buffer.extend_from_slice(&chunk[..n]);
                
                // Try to interpret as text
                if let Ok(text) = std::str::from_utf8(&chunk[..n]) {
                    if text.trim().len() > 0 {
                        println!("Text preview: {}", 
                                 if text.len() > 100 { &text[..100] } else { text });
                    }
                }
            },
            Err(e) => {
                println!("Read error: {}", e);
                break;
            }
        }
        
        // Break after some reasonable amount to avoid infinite loops
        if total_bytes > 1_000_000 {
            println!("Reached maximum read size");
            break;
        }
    }
    
    println!("Total bytes received: {}", total_bytes);
    
    // Analysis of the complete response
    if !buffer.is_empty() {
        // This looks like HTTP/2 or some other protocol data
        // Try to identify what it might be
        if buffer.len() >= 24 && buffer[0] == 0 && buffer[1] == 0 {
            println!("Response appears to be a binary protocol (possibly HTTP/2)");
            
            // Print more detailed hex dump for debugging
            println!("Full hex dump:");
            for (i, byte) in buffer.iter().enumerate() {
                if i % 16 == 0 {
                    print!("\n{:04X}: ", i);
                }
                print!("{:02X} ", byte);
            }
            println!();
            
            // If this is HTTP/2, we might need a different approach
            println!("Consider using a dedicated HTTP/2 client library for this connection");
        }
    }

    Ok(())
}

// Helper function to find end of HTTP headers
fn find_end_of_headers(buffer: &[u8]) -> Option<usize> {
    for i in 0..buffer.len().saturating_sub(3) {
        if buffer[i] == b'\r' && buffer[i+1] == b'\n' && 
           buffer[i+2] == b'\r' && buffer[i+3] == b'\n' {
            return Some(i + 4);
        }
    }
    None
}
