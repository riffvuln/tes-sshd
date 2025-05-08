use std::error::Error;
use std::process::Command;
use regex::Regex;
use tokio::process::Command;
use futures::future::try_join_all;

async fn check_lynx_installed() -> Result<(), Box<dyn Error>> {
    let result = Command::new("lynx").arg("--version").output().await;
    if result.is_err() {
        eprintln!("Error: lynx is not installed or not in PATH");
        eprintln!("Please install lynx with your package manager (e.g., apt install lynx)");
        std::process::exit(1);
    }
    Ok(())
}

async fn fetch_google_search_results(query: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("lynx")
        .arg("-listonly")
        .arg("-dump")
        .arg(format!("https://www.google.com/search?q={}", query))
        .output()
        .await?;

    if !output.status.success() {
        eprintln!("Error running lynx: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn extract_urls(lynx_output: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let re = Regex::new(r"https://www\.google\.com/url\?q=([^&]+)")?;
    let mut urls = Vec::new();

    for line in lynx_output.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(url_match) = captures.get(1) {
                // URL decode the extracted URL
                let decoded_url = urlencoding::decode(url_match.as_str())?;
                urls.push(decoded_url.into_owned());
            }
        }
    }

    Ok(urls)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    check_lynx_installed().await?;
    
    let search_query = "Rust";
    let lynx_output = fetch_google_search_results(search_query).await?;
    let urls = extract_urls(&lynx_output)?;
    
    for url in urls {
        println!("{}", url);
    }

    Ok(())
}