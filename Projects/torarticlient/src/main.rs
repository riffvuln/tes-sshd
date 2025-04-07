use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::{TlsConnector, Protocol};
use tokio_native_tls::TlsConnector as TokioTlsConnector;

const DOMAIN: &'static str = "myinstafollow.com";
const PORT: u16 = 443;
const PATH: &'static str = "free-tiktok-views";

// Firefox-like ciphers (mimicking NSS)
const FIREFOX_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384";

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

    // Wrap the stream with TLS
    let mut stream = tls_conn.connect(DOMAIN, stream).await?;

    // Send HTTP GET request with enhanced headers
    let request = format!(
        "GET /{PATH} HTTP/1.1\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
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

    // Read response with timeout to handle possible streaming responses
    let mut buffah = Vec::new();
    let mut chunk = [0u8; 4096];
    
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(10), 
            stream.read(&mut chunk)
        ).await {
            Ok(Ok(0)) => break, // EOF
            Ok(Ok(n)) => {
                buffah.extend_from_slice(&chunk[..n]);
                // If we detect end of HTTP response, break
                if let Some(body_pos) = find_end_of_headers(&buffah) {
                    if buffah.len() > body_pos && buffah.len() > body_pos + 4 {
                        break;
                    }
                }
            },
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => break, // Timeout
        }
    }
    
    println!("Response: {}", String::from_utf8_lossy(&buffah));
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
