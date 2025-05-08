use reqwest;
use scraper::{Html, Selector};
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

    // Parse HTML
    let document = Html::parse_document(&body);
    let link_selector = Selector::parse("a[href]").unwrap();
    
    // Extract and print all links in a format similar to lynx -listonly -dump
    println!("References\n");
    
    let mut counter = 1;
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            // Only include full URLs (skip javascript:void etc.)
            if href.starts_with("http") || href.starts_with("/") {
                let full_url = if href.starts_with("/") {
                    format!("https://www.google.com{}", href)
                } else {
                    href.to_string()
                };
                
                println!("  {}. {}", counter, full_url);
                counter += 1;
            }
        }
    }

    Ok(())
}