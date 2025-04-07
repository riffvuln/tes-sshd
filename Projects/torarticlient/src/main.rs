use arti_client::*;
use std::error::Error;
use reqwest_impersonate as reqwest;
use reqwest::impersonate::Impersonate;

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

    // Build a reqwest client using Chrome impersonation and our Tor connector
    let reqwest_client = reqwest::Client::builder()
        .impersonate(Impersonate::Chrome123)
        .enable_ech_grease()
        .permute_extensions()
        .cookie_store(true)
        .tcp_connector(tor_connector)
        .build()?;

    // Make the HTTP request through Tor
    let url = format!("https://{}/{}", DOMAIN, PATH);
    let response = reqwest_client.get(url).send().await?;
    
    // Print the response
    println!("Status: {}", response.status());
    println!("Response: {}", response.text().await?);

    Ok(())
}
