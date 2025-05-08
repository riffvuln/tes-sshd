use reqwest;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use chrono::prelude::*;
use std::collections::HashSet;
use html_escape::decode_html_entities;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Display current time in UTC
    let now = Utc::now();
    println!("Current Date and Time (UTC): {}", now.format("%Y-%m-%d %H:%M:%S"));
    println!("Current User's Login: riffvulnyang");
    
    // Set up a client with headers that mimic lynx
    let client = reqwest::Client::builder()
        .user_agent("Lynx/2.8.9rel.1 libwww-FM/2.14 SSL-MM/1.4.1 OpenSSL/1.1.1k")
        .build()?;
    
    // Fetch the Google search page
    let url = "https://www.google.com/search?q=Rust&oq=Rust";
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    
    // Parse the HTML document
    let dom = parse_document(RcDom::default(), Default::default())
        .one(body.as_bytes()).unwrap();
    
    // Extract all links in lynx format
    println!("\nReferences\n");
    let mut links = Vec::new();
    let mut visited = HashSet::new();
    extract_links(&dom.document, &mut links, &mut visited);
    
    // Print links in lynx format
    for (i, link) in links.iter().enumerate() {
        println!("   {}. {}", i + 1, link);
    }
    
    // Now extract only the Google redirect URLs and decode them
    // This simulates the grep and cut part of your pipeline
    let mut redirect_urls = Vec::new();
    for link in &links {
        if link.starts_with("https://www.google.com/url?q=") {
            // Extract the URL part after q= and before the first &
            if let Some(start_idx) = link.find("q=") {
                let start = start_idx + 2;
                if let Some(end_idx) = link[start..].find('&') {
                    let url = &link[start..start + end_idx];
                    // URL decode the extracted URL
                    match urlencoding::decode(url) {
                        Ok(decoded) => redirect_urls.push(decoded.to_string()),
                        Err(_) => redirect_urls.push(url.to_string()),
                    }
                }
            }
        }
    }
    
    // Print the extracted redirect URLs
    println!("\nExtracted Redirect URLs:\n");
    for url in redirect_urls {
        println!("{}", url);
    }
    
    Ok(())
}

// Function to extract links from the HTML document
fn extract_links(handle: &Handle, links: &mut Vec<String>, visited: &mut HashSet<*const Handle>) {
    // Avoid cycles in the DOM
    if !visited.insert(handle as *const Handle) {
        return;
    }
    
    let node = handle;
    
    match node.data {
        NodeData::Element { ref name, ref attrs, .. } => {
            if name.local.eq_str_ignore_ascii_case("a") {
                // Extract href attribute from <a> tags
                for attr in attrs.borrow().iter() {
                    if attr.name.local.eq_str_ignore_ascii_case("href") {
                        let href = attr.value.to_string();
                        // Normalize URL
                        let full_url = if href.starts_with('/') {
                            format!("https://www.google.com{}", href)
                        } else if !href.starts_with("http") && !href.starts_with("javascript:") {
                            format!("https://www.google.com/{}", href)
                        } else {
                            href
                        };
                        
                        // Decode HTML entities in the URL
                        let decoded_url = decode_html_entities(&full_url).to_string();
                        links.push(decoded_url);
                    }
                }
            }
            
            // Recursively process child nodes
            for child in node.children.borrow().iter() {
                extract_links(child, links, visited);
            }
        },
        NodeData::Document => {
            // Process child nodes of the document
            for child in node.children.borrow().iter() {
                extract_links(child, links, visited);
            }
        },
        _ => {
            // Process child nodes for other node types
            for child in node.children.borrow().iter() {
                extract_links(child, links, visited);
            }
        }
    }
}