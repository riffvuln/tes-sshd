use std::error::Error;
use regex::Regex;
use tokio::process::Command;
use clap::Parser;
use url::Url;
use std::time::Duration;
use tokio::sync::Semaphore;
use std::sync::Arc;

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

async fn search_page(query: &str, page: u32, timeout_secs: u64) -> Option<Vec<String>> {
    let search_url = format!("https://www.google.com/search?q={}&start={}", urlencoding::encode(query), page * 10);
    
    let output = match tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        Command::new("lynx")
            .arg("-listonly")
            .arg("-dump")
            .arg(&search_url)
            .output()
    ).await {
        Ok(result) => match result {
            Ok(output) => output,
            Err(_) => return None,
        },
        Err(_) => return None, // Timeout occurred
    };

    if !output.status.success() {
        return None;
    }

    let lynx_output = match String::from_utf8(output.stdout) {
        Ok(text) => text,
        Err(_) => return None,
    };
    
    let results = match extract_urls(&lynx_output) {
        Ok(results) => results,
        Err(_) => return None,
    };
    
    // If no results are found, return None to indicate end of pages
    if results.is_empty() {
        return None;
    }
    
    // Collect non-Google URLs
    let mut page_urls = Vec::new();
    for result in results {
        if !result.url.contains("google.com") {
            page_urls.push(result.url);
        }
    }
    
    // If no valid URLs found, consider it as an empty page
    if page_urls.is_empty() {
        return None;
    }
    
    Some(page_urls)
}

async fn search_until_end_page(query: &str, timeout_secs: u64) -> Result<Vec<String>, Box<dyn Error>> {
    let semaphore = Arc::new(Semaphore::new(20)); // Limit to 20 concurrent tasks
    let mut urls: Vec<String> = Vec::new();
    let mut page = 0;
    let mut tasks = Vec::new();

    // Start initial batch of tasks
    loop {
        let permit = semaphore.clone().acquire_owned().await?;
        let query_clone = query.to_string();
        
        let task = tokio::spawn(async move {
            let result = search_page(&query_clone, page, timeout_secs).await;
            (page, result, permit) // Return page number with result
        });
        
        tasks.push(task);
        page += 1;
        
        // If we've spawned 20 initial tasks, break to start processing results
        if page >= 20 {
            break;
        }
    }
    
    // Process results
    let mut found_last_page = false;
    let mut max_page_processed = 0;
    
    while let Some(task) = tasks.pop() {
        let (page_num, result, _permit) = task.await?; // permit is dropped here, freeing a slot
        
        max_page_processed = max_page_processed.max(page_num);
        
        match result {
            Some(page_urls) => {
                urls.extend(page_urls);
                
                // If we haven't found the last page yet, keep creating new tasks
                if !found_last_page {
                    let permit = semaphore.clone().acquire_owned().await?;
                    let query_clone = query.to_string();
                    
                    let new_page = page;
                    page += 1;
                    
                    let task = tokio::spawn(async move {
                        let result = search_page(&query_clone, new_page, timeout_secs).await;
                        (new_page, result, permit)
                    });
                    
                    tasks.push(task);
                }
            },
            None => {
                found_last_page = true;
            }
        }
        
        if found_last_page && tasks.is_empty() {
            break;
        }
    }
    
    Ok(urls)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let search_query = args.query.join(" ");
    
    check_lynx_installed().await?;
    
    // Search with concurrent tasks
    let urls = search_until_end_page(&search_query, args.timeout).await?;
    if urls.is_empty() {
        println!("No results found.");
        return Ok(());
    }
    
    // Print the URLs
    for (indx, url) in urls.iter().enumerate() {
        println!("{}: {}", indx + 1, url);
    }
    Ok(())
}