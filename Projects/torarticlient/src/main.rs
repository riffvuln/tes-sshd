use arti_client::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use native_tls::TlsConnector;
use tokio_native_tls::TlsConnector as TokioTlsConnector;

const DOMAIN: &'static str = "erhrkm6c4qzohi74yu7gb3bjgywecs3m6jyaok2hsw5qbozhv6ck32ad.onion";
const PORT: u16 = 80;
const PATH: &'static str = "";

// Define connection types
enum ConnectionType {
    WithTLS,    // For regular networks
    WithoutTLS, // For Tor/onion networks
}

// Set the connection type here to toggle between TLS and non-TLS
const CONNECTION_TYPE: ConnectionType = ConnectionType::WithoutTLS;

#[tokio::main]
pub(crate) async fn main() -> anyhow::Result<()> {
    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;
    
    // Connect using the selected connection type
    match CONNECTION_TYPE {
        ConnectionType::WithTLS => connect_with_tls(client).await?,
        ConnectionType::WithoutTLS => connect_without_tls(client).await?,
    }
    
    Ok(())
}

// Function for TLS connection
async fn connect_with_tls(client: TorClient) -> anyhow::Result<()> {
    // Set up native TLS configuration
    let tls_conn = TlsConnector::new()?;
    let tls_conn = TokioTlsConnector::from(tls_conn);
    
    // Make stream to the target domain with tor
    let stream = client.connect((DOMAIN, PORT)).await?;

    // Wrap the stream with TLS
    let mut stream = tls_conn.connect(DOMAIN, stream).await?;

    // Send HTTP request and handle response
    send_request_and_handle_response(&mut stream).await?;
    
    Ok(())
}

// Function for non-TLS connection
async fn connect_without_tls(client: TorClient) -> anyhow::Result<()> {
    // Make stream to the target domain with tor (without TLS)
    let mut stream = client.connect((DOMAIN, PORT)).await?;
    
    // Send HTTP request and handle response
    send_request_and_handle_response(&mut stream).await?;
    
    Ok(())
}

// Common function to send request and handle response
async fn send_request_and_handle_response<T: AsyncReadExt + AsyncWriteExt + Unpin>(stream: &mut T) -> anyhow::Result<()> {
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
    println!("Response: {}", String::from_utf8_lossy(&buffah));
    
    Ok(())
}