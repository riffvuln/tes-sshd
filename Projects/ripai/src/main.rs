use reqwest;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up a client with appropriate headers to appear more like a browser
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Lynx/2.8.9rel.1; libwww-FM/2.14)")
        .build()?;
    
    // Fetch the Google search page
    let url = "https://www.google.com/search?q=Rust&oq=Rust";
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    
    // Process the body to extract links and print them in lynx's format
    println!("References\n");
    
    // Extract Google redirect URLs and process them
    // This regex finds all Google redirect URLs and captures the actual destination URL
    let re = regex::Regex::new(r"https://www\.google\.com/url\?q=([^&]+)")?;
    
    // Find all matches
    for (i, cap) in re.captures_iter(&body).enumerate() {
        if let Some(url_match) = cap.get(1) {
            // URL decode the extracted URL
            let decoded_url = urlencoding::decode(url_match.as_str())?;
            println!("  {}. {}", i+1, decoded_url);
        }
    }
    
    Ok(())
}