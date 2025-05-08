use reqwest;
use regex::Regex;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up a user agent to avoid being blocked
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;

    // Fetch the Google search page
    let url = "https://www.google.com/search?q=Rust&oq=Rust";
    let response = client.get(url).send().await?;
    let body = response.text().await?;

    // Create regex to extract URLs similar to the grep pattern
    let re = Regex::new(r"https://www\.google\.com/url\?q=([^&]+)")?;

    // Extract and process all matches
    for cap in re.captures_iter(&body) {
        if let Some(url_match) = cap.get(1) {
            // URL decode the extracted URL
            let decoded_url = urlencoding::decode(url_match.as_str())?;
            println!("{}", decoded_url);
        }
    }

    Ok(())
}