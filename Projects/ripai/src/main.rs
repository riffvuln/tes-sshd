use std::error::Error;
use regex::Regex;
use tokio::process::Command;
use clap::Parser;
use url::Url;
use std::time::Duration;

/// Extract URLs from search results
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Search query
    #[clap(required = true)]
    query: Vec<String>,
    
    /// Timeout in seconds
    #[clap(short, long, default_value = "10")]
    timeout: u64,
}

#[derive(Debug)]
struct SearchResult {
    url: String,
}

async fn check_lynx_installed() -> Result<(), Box<dyn Error>> {
    if Command::new("lynx").arg("--version").output().await.is_err() {
        eprintln!("Error: lynx is not installed or not in PATH");
        eprintln!("Please install lynx with your package manager (e.g., apt install lynx)");
        return Err("lynx command not found".into());
    }
    Ok(())
}

async fn fetch_google_search_results(query: &str, timeout_secs: u64) -> Result<String, Box<dyn Error>> {
    let search_url = format!("https://www.google.com/search?q={}", urlencoding::encode(query));
    
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        Command::new("lynx")
            .arg("-listonly")
            .arg("-dump")
            .arg(&search_url)
            .output()
    ).await??;

    if !output.status.success() {
        return Err(format!("Error running lynx: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn extract_urls(lynx_output: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let re = Regex::new(r"(?:\d+\.\s+)?https://www\.google\.com/url\?(?:[^&]*&)*q=([^&]+)")?;
    let mut results = Vec::new();

    for line in lynx_output.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(url_match) = captures.get(1) {
                let decoded_url = urlencoding::decode(url_match.as_str())?;
                
                // Validate the URL
                if let Ok(parsed_url) = Url::parse(&decoded_url) {
                    if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                        results.push(SearchResult {
                            url: decoded_url.into_owned(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

async fn search_until_end_page(query: &str, timeout_secs: u64) -> Result<Vec<String>, Box<dyn Error>> {
    let mut page: u32 = 1; // if that possible for google search make overflow u32??
    let mut urls: Vec<String> = Vec::new();
    loop {
        let search_url = format!("https://www.google.com/search?q={}&start={}", urlencoding::encode(query), page * 10);
        
        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            Command::new("lynx")
                .arg("-listonly")
                .arg("-dump")
                .arg(&search_url)
                .output()
        ).await??;

        if !output.status.success() {
            return Err(format!("Error running lynx: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let lynx_output = String::from_utf8(output.stdout)?;
        let results = extract_urls(&lynx_output)?;
        for result in results {
            if result.url.contains("google.com") {
                continue; // Skip Google URLs
            }
            urls.push(result.url);
        };

        page += 1;
    }
    Ok(urls)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let search_query = args.query.join(" ");
    
    check_lynx_installed().await?;
    
    let lynx_output = fetch_google_search_results(&search_query, args.timeout).await?;
    let results = extract_urls(&lynx_output)?;
    
    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }
    
    // Print only the URLs, one per line - clean output format
    for result in results {
        println!("{}", result.url);
    }
    
    Ok(())
}