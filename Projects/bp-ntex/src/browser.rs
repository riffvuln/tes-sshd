use std::time::Duration;
use thirtyfour::prelude::*;
use crate::utils::PAGE_LOAD_WAIT_MS;

/// Process a URL in a new tab and return the HTML content
pub async fn process_url_in_tab(
    driver: WebDriver, 
    url: String, 
    request_id: &str
) -> Result<String, WebDriverError> {
    println!("[{}] Processing URL: {}", request_id, url);
    
    // Create a new tab
    let tab = match driver.new_tab().await {
        Ok(tab) => tab,
        Err(e) => {
            eprintln!("[{}] Failed to create new tab: {}", request_id, e);
            return Err(e);
        }
    };
    
    println!("[{}] Created new tab: {}", request_id, tab.to_string());
    
    // Switch to the new tab
    match driver.switch_to_window(tab.clone()).await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("[{}] Failed to switch to new tab: {}", request_id, e);
            return Err(e);
        }
    }
    
    // Navigate to the requested URL
    match driver.goto(&url).await {
        Ok(_) => println!("[{}] Navigated to URL successfully", request_id),
        Err(e) => {
            eprintln!("[{}] Failed to navigate to URL: {}", request_id, e);
            // Try to close the tab even if navigation failed
            let _ = driver.close_window().await;
            return Err(e);
        }
    }
    
    // Wait for page to fully load
    println!("[{}] Waiting for page to load...", request_id);
    tokio::time::sleep(Duration::from_millis(PAGE_LOAD_WAIT_MS)).await;
    
    // Get the page source
    let html = match driver.source().await {
        Ok(source) => {
            println!("[{}] Successfully retrieved page content", request_id);
            source
        },
        Err(e) => {
            eprintln!("[{}] Failed to get page source: {}", request_id, e);
            // Try to close the tab
            let _ = driver.close_window().await;
            return Err(e);
        }
    };
    
    // Clean up - close the tab
    match driver.close_window().await {
        Ok(_) => println!("[{}] Closed tab successfully", request_id),
        Err(e) => {
            eprintln!("[{}] Warning: Failed to close tab: {}", request_id, e);
            // Continue anyway, since we have the HTML
        }
    }
    
    // Return to another window/tab if any exist
    if let Ok(windows) = driver.windows().await {
        if !windows.is_empty() {
            let _ = driver.switch_to_window(windows[0].clone()).await;
        }
    }
    
    println!("[{}] Request completed successfully", request_id);
    Ok(html)
}
