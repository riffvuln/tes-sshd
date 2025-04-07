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

    // Read response in chunks
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
                total_bytes += n;
                buffer.extend_from_slice(&chunk[..n]);
                
                // Just log byte count for cleaner output
                println!("Read {} bytes (total: {})", n, total_bytes);
            },
            Err(e) => {
                println!("Read error: {}", e);
                break;
            }
        }
    }
    
    println!("Total bytes received: {}", total_bytes);
    
    // Process the complete HTTP response
    if !buffer.is_empty() {
        // Try to parse as HTTP response
        if let Ok(full_response) = String::from_utf8(buffer.clone()) {
            // Look for the HTTP header/body separator
            if let Some(headers_end) = full_response.find("\r\n\r\n") {
                let headers = &full_response[0..headers_end];
                let body = &full_response[headers_end + 4..];
                
                println!("\n===== HTTP HEADERS =====");
                println!("{}", headers);
                
                println!("\n===== CONTENT PREVIEW =====");
                if body.len() > 1000 {
                    println!("{}", &body[0..1000]);
                    println!("... [truncated, total body size: {} bytes]", body.len());
                } else {
                    println!("{}", body);
                }
                
                // Save the HTML to a file for easier analysis
                let output_file = "response.html";
                std::fs::write(output_file, body)?;
                println!("\nFull HTML content saved to '{}'", output_file);
                
                // Check for common anti-bot measures in the response
                check_anti_bot_measures(body);
            } else {
                println!("Could not find HTTP headers separator");
                println!("Raw response preview: {}", 
                    if full_response.len() > 200 { &full_response[0..200] } else { &full_response });
            }
        } else {
            println!("Response is not valid UTF-8");
        }
    }

    Ok(())
}

// Helper function to check for common anti-bot protection mechanisms
fn check_anti_bot_measures(content: &str) {
    println!("\n===== PROTECTION ANALYSIS =====");
    
    // Check for common anti-bot solutions
    if content.contains("Cloudflare") {
        println!("⚠️ Cloudflare protection detected");
    }
    
    if content.contains("CAPTCHA") || content.contains("captcha") {
        println!("⚠️ CAPTCHA challenge detected");
    }
    
    if content.contains("webdriver") || content.contains("navigator.") {
        println!("⚠️ Browser fingerprinting detected (checking for automation)");
    }
    
    if content.contains("document.cookie") {
        println!("⚠️ Cookie-based verification detected");
    }
    
    if content.contains("window.location") {
        println!("⚠️ Redirect mechanism detected");
    }
    
    if content.contains("hCaptcha") || content.contains("h-captcha") {
        println!("⚠️ hCaptcha detected");
    }
    
    if content.contains("recaptcha") || content.contains("grecaptcha") {
        println!("⚠️ Google reCAPTCHA detected");
    }
    
    if content.contains("PerimeterX") || content.contains("px-captcha") {
        println!("⚠️ PerimeterX bot protection detected");
    }
    
    if content.contains("Imperva") || content.contains("incapsula") {
        println!("⚠️ Imperva/Incapsula protection detected");
    }

    // Look for JavaScript fingerprinting
    let fingerprinting_patterns = [
        "navigator.userAgent", "navigator.plugins", "navigator.platform",
        "screen.width", "screen.height", "canvas.toDataURL", "webgl",
        "AudioContext", "fontFamily", "userAgentCheck"
    ];
    
    for pattern in fingerprinting_patterns {
        if content.contains(pattern) {
            println!("⚠️ Browser fingerprinting detected: {}", pattern);
        }
    }
    
    // If we didn't detect anything specific
    println!("Note: The site may be using custom or obfuscated protection mechanisms");
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
