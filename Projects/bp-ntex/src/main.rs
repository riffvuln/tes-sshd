use core::panic;

use ntex::web;
use thirtyfour::{common::print, prelude::*};
use std::sync::{Mutex, Arc, Once};
use lazy_static::lazy_static;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::{RwLock, Mutex as TokioMutex};
use tokio::task;
use rand::Rng;

// Constants for configuration
const DEFAULT_PAGE: &str = "about:blank";
const PAGE_LOAD_WAIT_MS: u64 = 1500;
const WEBDRIVER_URL: &str = "http://localhost:4444";

#[web::get("/")]
async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

// Global static reference to the WebDriver instance and request tracking
lazy_static! {
    // The WebDriver instance shared across all requests
    static ref DRIVER: Arc<Mutex<Option<WebDriver>>> = Arc::new(Mutex::new(None));
    static ref INIT: Once = Once::new();
    // Track pending requests to know when to return to default page
    static ref PENDING_REQUESTS: Arc<RwLock<HashMap<String, bool>>> = Arc::new(RwLock::new(HashMap::new()));
    // Lock to synchronize WebDriver operations
    static ref DRIVER_LOCK: Arc<TokioMutex<()>> = Arc::new(TokioMutex::new(()));
}

/// Get the existing WebDriver instance or create a new one
async fn get_or_create_driver() -> Result<WebDriver, WebDriverError> {
    // Get exclusive access to driver creation
    let _driver_lock = DRIVER_LOCK.lock().await;
    
    let driver_option = DRIVER.lock().unwrap().clone();
    
    match driver_option {
        Some(driver) => {
            // Driver exists, check if it's still valid
            match driver.title().await {
                Ok(_) => Ok(driver), // Driver is responsive
                Err(e) => {
                    println!("Driver became unresponsive: {}", e);
                    println!("Creating a new WebDriver instance...");
                    let new_driver = create_driver().await?;
                    *DRIVER.lock().unwrap() = Some(new_driver.clone());
                    Ok(new_driver)
                }
            }
        },
        None => {
            // First time, create the driver
            println!("Creating WebDriver for the first time");
            let new_driver = create_driver().await?;
            *DRIVER.lock().unwrap() = Some(new_driver.clone());
            Ok(new_driver)
        }
    }
}

/// Create a new WebDriver instance with the desired capabilities
async fn create_driver() -> Result<WebDriver, WebDriverError> {
    let mut caps = DesiredCapabilities::firefox();
    
    // Configure browser capabilities
    caps.set_headless()?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    
    // Create the WebDriver session
    let driver = WebDriver::new(WEBDRIVER_URL, caps).await?;
    
    // Navigate to default page initially
    driver.goto(DEFAULT_PAGE).await?;
    
    Ok(driver)
}

/// Clean up the WebDriver state
async fn reset_browser_state(driver: &WebDriver) -> Result<(), WebDriverError> {
    println!("Resetting browser state...");
    
    // Clear cookies
    if let Err(e) = driver.delete_all_cookies().await {
        eprintln!("Warning: Failed to delete cookies: {}", e);
    }
    
    // Navigate back to default page
    driver.goto(DEFAULT_PAGE).await?;
    
    Ok(())
}

/// Generate a unique request ID for tracking
fn generate_request_id() -> String {
    let mut rng = rand::thread_rng();
    let random_num: u64 = rng.gen_range(100000..999999);
    format!("req-{}", random_num)
}

/// Process a URL in a new tab and return the HTML content
async fn process_url_in_tab(
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

/// Handle POST requests to /bp endpoint
#[web::post("/bp")]
async fn bp(req_body: String) -> impl web::Responder {
    println!("Received request: {}", req_body);
    
    // Generate a unique request ID for tracking
    let request_id = generate_request_id();
    println!("[{}] Processing new request", request_id);
    
    // Add the request to pending requests tracking
    PENDING_REQUESTS.write().await.insert(request_id.clone(), true);
    
    // Get or create the WebDriver instance
    let driver = match get_or_create_driver().await {
        Ok(driver) => driver,
        Err(e) => {
            eprintln!("[{}] Error getting WebDriver: {}", request_id, e);
            // Clean up the pending request
            PENDING_REQUESTS.write().await.remove(&request_id);
            return web::HttpResponse::InternalServerError().body(format!("WebDriver error: {}", e));
        }
    };
    
    // Clone what we need for the async task
    let driver_clone = driver.clone();
    let url_clone = req_body;
    let request_id_clone = request_id.clone();
    let driver_lock = DRIVER_LOCK.clone();
    
    // Process the URL in a separate tokio task with proper locking
    let result = task::spawn(async move {
        // Get exclusive access to WebDriver operations
        let _guard = driver_lock.lock().await;
        
        // Process the URL in a new tab
        process_url_in_tab(driver_clone, url_clone, &request_id_clone).await
    }).await;
    
    // Remove from pending requests
    PENDING_REQUESTS.write().await.remove(&request_id);
    
    // Check if this was the last request and clean up if needed
    if PENDING_REQUESTS.read().await.is_empty() {
        println!("[{}] No more pending requests, returning to default page", request_id);
        if let Ok(driver) = get_or_create_driver().await {
            if let Err(e) = reset_browser_state(&driver).await {
                eprintln!("[{}] Error resetting browser state: {}", request_id, e);
            }
        }
    }
    
    // Return appropriate response based on the result
    match result {
        Ok(Ok(html)) => {
            println!("[{}] Successfully returning HTML response", request_id);
            web::HttpResponse::Ok().body(html)
        },
        Ok(Err(e)) => {
            eprintln!("[{}] Error processing URL: {}", request_id, e);
            web::HttpResponse::InternalServerError().body(format!("Processing error: {}", e))
        },
        Err(e) => {
            eprintln!("[{}] Task execution error: {}", request_id, e);
            web::HttpResponse::InternalServerError().body(format!("Task error: {}", e))
        }
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Starting WebDriver proxy server on 127.0.0.1:8080");
    
    web::HttpServer::new(|| {
        web::App::new()
            .service(index)
            .service(bp)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}