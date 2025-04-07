use arti_client::*;
use std::error::Error;
use reqwest;
use hyper;
use hyper_tls;

const DOMAIN: &'static str = "myinstafollow.com";
const PORT: u16 = 443;
const PATH: &'static str = "free-tiktok-views";

#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn Error>> {
    // Set up Tor client
    let cfg = TorClientConfig::default();
    let client = TorClient::create_bootstrapped(cfg).await?;

    // Create a custom connector that routes through Tor
    let tor_connector = tower::service_fn(move |req: hyper::Uri| {
        let client = client.clone();
        let host = req.host().unwrap_or("").to_string();
        let port = req.port_u16().unwrap_or(443);
        
        async move {
            let stream = client.connect((host.as_str(), port)).await?;
            Ok::<_, arti_client::Error>(stream)
        }
    });

    // Build a reqwest client using our Tor connector and browser-like headers
    let https = hyper_tls::HttpsConnector::from((tor_connector, native_tls::TlsConnector::new()?));
    let client = hyper::Client::builder().build(https);

    // Create a reqwest client with custom headers to simulate Chrome
    let reqwest_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
            headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
            headers.insert("Cache-Control", "max-age=0".parse().unwrap());
            headers.insert("Sec-Ch-Ua", "\"Google Chrome\";v=\"123\", \"Not:A-Brand\";v=\"8\"".parse().unwrap());
            headers.insert("Sec-Ch-Ua-Mobile", "?0".parse().unwrap());
            headers.insert("Sec-Ch-Ua-Platform", "\"Windows\"".parse().unwrap());
            headers
        })
        .cookie_store(true)
        .build()?;

    // Make the HTTP request through Tor
    let url = format!("https://{}/{}", DOMAIN, PATH);
    let response = reqwest_client.get(url).send().await?;
    
    // Print the response
    println!("Status: {}", response.status());
    println!("Response: {}", response.text().await?);

    Ok(())
}
