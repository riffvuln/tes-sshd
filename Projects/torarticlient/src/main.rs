use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use flate2::read::GzDecoder;
use std::io::Read;

const DOMAIN: &'static str = "erhrkm6c4qzohi74yu7gb3bjgywecs3m6jyaok2hsw5qbozhv6ck32ad.onion";
const PORT: u16 = 80;
const PATH: &'static str = "";

#[tokio::main]
pub (crate) async fn main() -> anyhow::Result<()> {
    // Set up native TLS configuration
    // let tls_conn = TlsConnector::new()?;
    // let tls_conn = TokioTlsConnector::from(tls_conn);

    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Make stream to the target domain with tor
    let mut stream = client.connect((DOMAIN, PORT)).await?;

    // Wrap the stream with TLS
    // let mut stream = tls_conn.connect(DOMAIN, stream).await?;

    // Send HTTP GET request
    let request = format!(
        "GET /{PATH} HTTP/1.1\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        Accept-Language: en-US,en;q=0.9\r\n\
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
        \r\n"
    );
    stream.write_all(request.as_bytes()).await?;

    // Flush the stream to ensure the request is sent
    stream.flush().await?;

    // Read response
    let mut buffah = Vec::new();
    stream.read_to_end(&mut buffah).await?;
    
    // Process the HTTP response
    // First find the headers
    let mut headers_end = 0;
    for i in 0..(buffah.len().saturating_sub(3)) {
        if buffah[i] == b'\r' && buffah[i+1] == b'\n' && 
           buffah[i+2] == b'\r' && buffah[i+3] == b'\n' {
            headers_end = i + 4;
            break;
        }
    }
    
    if headers_end > 0 {
        // Parse headers
        let headers_str = String::from_utf8_lossy(&buffah[0..headers_end]);
        println!("=== HEADERS ===");
        println!("{}", headers_str);
        
        // Check if content is gzipped
        let is_gzipped = headers_str.to_lowercase().contains("content-encoding: gzip");
        let body_bytes = &buffah[headers_end..];
        
        // Auto-detect gzip even if not in headers (gzip magic bytes: 1F 8B)
        let is_likely_gzip = body_bytes.len() >= 2 && body_bytes[0] == 0x1F && body_bytes[1] == 0x8B;
        
        if is_gzipped || is_likely_gzip {
            println!("\n=== DECOMPRESSING GZIPPED CONTENT ===");
            let mut decoder = GzDecoder::new(&body_bytes[..]);
            let mut decompressed = Vec::new();
            
            match decoder.read_to_end(&mut decompressed) {
                Ok(_) => {
                    match String::from_utf8(decompressed) {
                        Ok(content) => {
                            println!("=== DECOMPRESSED UTF-8 BODY ===");
                            println!("{}", content);
                        },
                        Err(_) => {
                            println!("Decompressed content is not valid UTF-8");
                            println!("Showing as UTF-8 anyway (with replacements): {}", 
                                     String::from_utf8_lossy(&decompressed));
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to decompress: {}", e);
                    println!("Showing raw body as UTF-8 (may show garbage): {}", 
                             String::from_utf8_lossy(body_bytes));
                }
            }
        } else {
            // Regular non-gzipped content
            println!("\n=== BODY ===");
            println!("{}", String::from_utf8_lossy(body_bytes));
        }
    } else {
        // No headers found, treat as raw data
        println!("No HTTP headers detected. Raw response:");
        println!("{}", String::from_utf8_lossy(&buffah));
    }
    
    Ok(())
}