use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::Protocol;
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use flate2::read::GzDecoder;
use std::io::Read;

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
        // native_tls::Protocol doesn't have Tlsv13, so we don't set max version
        // which means it will use the highest available version
        .use_sni(true)
        // Enable ALPN for HTTP/2 support (mimicking browser)
        .request_alpns(&["h2", "http/1.1"]);
    
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
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
        Accept-Encoding: identity\r\n\
        Accept-Language: en-US,en;q=0.9\r\n\
        Connection: keep-alive\r\n\
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

    // Read the complete response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    
    println!("Received response of {} bytes", buffer.len());

    // Process the response
    if buffer.is_empty() {
        println!("Empty response received");
        return Ok(());
    }

    // Try to parse as HTTP response
    if let Ok(response_text) = String::from_utf8(buffer.clone()) {
        if response_text.starts_with("HTTP/") {
            // Check if response is gzipped
            if response_text.contains("Content-Encoding: gzip") {
                println!("Response is gzipped, attempting to decode");
                
                if let Some(body_start) = find_end_of_headers(&buffer) {
                    let body_data = &buffer[body_start..];
                    let mut decoder = GzDecoder::new(body_data);
                    let mut decoded_data = Vec::new();
                    
                    if decoder.read_to_end(&mut decoded_data).is_ok() {
                        println!("Decoded gzipped content:");
                        if let Ok(decoded_text) = String::from_utf8(decoded_data) {
                            println!("{}", decoded_text);
                        } else {
                            println!("Decoded content is not valid UTF-8");
                        }
                    } else {
                        println!("Failed to decode gzipped content");
                    }
                }
            } else {
                // Not gzipped, print as-is
                println!("Response (first 1000 chars):");
                if response_text.len() > 1000 {
                    println!("{}", &response_text[..1000]);
                    println!("... [truncated]");
                } else {
                    println!("{}", response_text);
                }
            }
        } else {
            println!("Response doesn't look like HTTP, showing first 100 bytes as hex:");
            for byte in buffer.iter().take(100) {
                print!("{:02X} ", byte);
            }
            println!();
        }
    } else {
        println!("Response is not valid UTF-8, showing first 100 bytes as hex:");
        for byte in buffer.iter().take(100) {
            print!("{:02X} ", byte);
        }
        println!();
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
