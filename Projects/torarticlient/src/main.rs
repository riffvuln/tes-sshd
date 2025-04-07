use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

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
    if let Ok(response_str) = String::from_utf8(buffah.clone()) {
        // Split headers and body
        if let Some(header_body_split) = response_str.split_once("\r\n\r\n") {
            println!("=== HEADERS ===");
            println!("{}", header_body_split.0);
            println!("\n=== BODY ===");
            println!("{}", header_body_split.1);
        } else {
            println!("Could not split headers and body");
            println!("Raw response: {}", response_str);
        }
    } else {
        // Handle binary data - show headers if possible
        let mut headers_end = 0;
        for i in 0..(buffah.len().saturating_sub(3)) {
            if buffah[i] == b'\r' && buffah[i+1] == b'\n' && 
               buffah[i+2] == b'\r' && buffah[i+3] == b'\n' {
                headers_end = i + 4;
                break;
            }
        }
        
        if headers_end > 0 {
            println!("=== HEADERS ===");
            println!("{}", String::from_utf8_lossy(&buffah[0..headers_end]));
            println!("\n=== BINARY BODY ===");
            println!("Length: {} bytes", buffah.len() - headers_end);
            // Print first 100 bytes as hex for debugging
            let preview_len = std::cmp::min(100, buffah.len() - headers_end);
            for byte in &buffah[headers_end..headers_end + preview_len] {
                print!("{:02X} ", byte);
            }
            println!("\n");
        } else {
            println!("Binary response: {} bytes", buffah.len());
        }
    }
    
    Ok(())
}