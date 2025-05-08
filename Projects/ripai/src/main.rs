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
    title: Option<String>,
}

async fn check_lynx_installed() -> Result<(), Box<dyn Error>> {
    let result = Command::new("lynx").arg("--version").output().await;
    if result.is_err() {
        eprintln!("Error: lynx is not installed or not in PATH");
        eprintln!("Please install lynx with your package manager (e.g., apt install lynx)");
        return Err("lynx command not found".into());
    }
    Ok(())
}

async fn fetch_google_search_results(query: &str, timeout_secs: u64) -> Result<String, Box<dyn Error>> {
    // Properly encode the query for URL
    let encoded_query = urlencoding::encode(query);
    let search_url = format!("https://www.google.com/search?q={}", encoded_query);
    
    // Use a timeout for the command
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        Command::new("lynx")
            .arg("-listonly")
            .arg("-dump")
            .arg(&search_url)
            .output()
    ).await??;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error running lynx: {}", error_msg).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn extract_urls(lynx_output: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    // More robust regex that handles different Google URL formats
    let re = Regex::new(r"(?:\d+\.\s+)?(https://www\.google\.com/url\?(?:[^&]*&)*q=([^&]+)(?:&[^&]*)*)")?;
    let mut results = Vec::new();

    for line in lynx_output.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(url_match) = captures.get(2) {
                // URL decode the extracted URL
                let decoded_url = urlencoding::decode(url_match.as_str())?;
                
                // Extract title if available (usually after the URL in lynx output)
                let title = line.split(" - ").nth(1).map(String::from);
                
                // Validate the URL
                if let Ok(parsed_url) = Url::parse(&decoded_url) {
                    // Only include http/https URLs
                    if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                        results.push(SearchResult {
                            url: decoded_url.into_owned(),
                            title,
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

async fn fetch_page_title(url: &str, timeout: Duration) -> Result<Option<String>, Box<dyn Error>> {
    // Create a client with timeout
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()?;
    
    // Fetch just the headers to be efficient
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; RipAI/1.0)")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Ok(None);
    }
    
    // Get the full page content
    let text = response.text().await?;
    
    // Extract title using regex (simple approach)
    let title_re = Regex::new(r"<title[^>]*>([^<]+)</title>")?;
    if let Some(captures) = title_re.captures(&text) {
        if let Some(title) = captures.get(1) {
            return Ok(Some(title.as_str().trim().to_string()));
        }
    }
    
    Ok(None)
}

async fn process_search_results(results: &[SearchResult], timeout: Duration, limit: usize) -> Vec<SearchResult> {
    let mut futures = Vec::new();
    
    // Only process up to the limit
    for result in results.iter().take(limit) {
        let url = result.url.clone();
        let title_clone = result.title.clone();
        
        futures.push(async move {
            // Return a Result<SearchResult, Box<dyn Error>> instead of SearchResult directly
            let title = match fetch_page_title(&url, timeout).await {
                Ok(Some(title)) => Some(title),
                _ => title_clone, // Fall back to the original title if any
            };
            
            Ok::<_, Box<dyn Error>>(SearchResult {
                url,
                title,
            })
        });
    }
    
    // Use join_all instead of try_join_all since we're now handling errors inside each future
    match futures::future::join_all(futures).await.into_iter().collect::<Vec<_>>() {
        results_vec if !results_vec.is_empty() => {
            // Filter out any errors and keep successful results
            results_vec.into_iter()
                .filter_map(|res| res.ok())
                .collect()
        },
        _ => Vec::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Combine the query terms
    let search_query = args.query.join(" ");
    let timeout_duration = Duration::from_secs(args.timeout);
    
    // Check if lynx is installed
    check_lynx_installed().await?;
    
    // Fetch search results
    println!("Searching for: {}", search_query);
    let lynx_output = fetch_google_search_results(&search_query, args.timeout).await?;
    let search_results = extract_urls(&lynx_output)?;
    
    if search_results.is_empty() {
        println!("No results found for query: {}", search_query);
        return Ok(());
    }
    
    // Process the search results to get better titles
    println!("Found {} results, fetching details for top {}...", search_results.len(), args.limit);
    let enhanced_results = process_search_results(&search_results, timeout_duration, args.limit).await;
    
    // Display results
    println!("\nSearch Results for '{}':", search_query);
    println!("==========================================");
    
    for (idx, result) in enhanced_results.iter().enumerate() {
        println!("{}. {}", idx + 1, result.url);
        if let Some(title) = &result.title {
            println!("   Title: {}", title);
        }
        println!();
    }
    
    Ok(())
}