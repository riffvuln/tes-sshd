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

#[web::get("/")]
async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

// Global static reference to the WebDriver instance and request tracking
lazy_static! {
    static ref DRIVER: Arc<Mutex<Option<WebDriver>>> = Arc::new(Mutex::new(None));
    static ref INIT: Once = Once::new();
    static ref PENDING_REQUESTS: Arc<RwLock<HashMap<String, bool>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref DRIVER_LOCK: Arc<TokioMutex<()>> = Arc::new(TokioMutex::new(()));
}

async fn get_or_create_driver() -> Result<WebDriver, WebDriverError> {
    // Get exclusive access to driver creation
    let _driver_lock = DRIVER_LOCK.lock().await;
    
    let driver_option = DRIVER.lock().unwrap().clone();
    
    match driver_option {
        Some(driver) => {
            // Driver exists, check if it's still valid
            match driver.title().await {
                Ok(_) => Ok(driver), // Driver is responsive
                Err(_) => {
                    // Driver is no longer valid, create a new one
                    println!("Driver is no longer responsive, creating a new one");
                    let new_driver = create_driver().await?;
                    *DRIVER.lock().unwrap() = Some(new_driver.clone());
                    Ok(new_driver)
                }
            }
        },
        None => {
            // First time, create the driver
            println!("Creating driver for the first time");
            let new_driver = create_driver().await?;
            *DRIVER.lock().unwrap() = Some(new_driver.clone());
            Ok(new_driver)
        }
    }
}

async fn create_driver() -> Result<WebDriver, WebDriverError> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    
    // Add needed capabilities for running in CI environment
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    
    // Add connection timeout capabilities
    caps.add_timeouts("script", 30000)?;
    caps.add_timeouts("pageLoad", 30000)?;
    caps.add_timeouts("implicit", 10000)?;
    
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    
    // Navigate to a blank page initially
    driver.goto("about:blank").await?;
    
    Ok(driver)
}

async fn cleanup_tab(driver: &WebDriver) -> Result<(), WebDriverError> {
    // Clear cookies
    if let Err(e) = driver.delete_all_cookies().await {
        println!("Warning: Failed to delete cookies: {}", e);
    }
    
    // Navigate back to a blank page
    driver.goto("about:blank").await?;
    
    Ok(())
}

// Function to generate a random request ID
fn generate_request_id() -> String {
    let mut rng = rand::thread_rng();
    let random_num: u64 = rng.gen_range(100000..999999);
    format!("req-{}", random_num)
}

// Process a URL in a new tab and return the HTML
async fn process_url_in_tab(driver: WebDriver, url: String, request_id: String) -> Result<String, WebDriverError> {
    println!("Processing request {} for URL: {}", request_id, url);
    
    let driver_lock = DRIVER_LOCK.clone();
    
    // Create the task that processes the URL
    let result = task::spawn(async move {
        // Get exclusive access to the WebDriver
        let _guard = driver_lock.lock().await;
        
        // Safety check if driver is still responsive
        match driver.title().await {
            Ok(_) => {}, // Driver is good
            Err(e) => {
                println!("Driver became unresponsive before processing request {}: {}", request_id, e);
                return Err(WebDriverError::CustomError(format!("Driver became unresponsive: {}", e)));
            }
        }
        
        // Create a new tab with retry mechanism
        let tab = match driver.new_tab().await {
            Ok(tab) => tab,
            Err(e) => {
                println!("Failed to create new tab for request {}: {}", request_id, e);
                return Err(e);
            }
        };
        
        let tab_id = tab.to_string();
        println!("Created new tab: {}", tab_id);
        
        // Switch to the new tab
        match driver.switch_to_window(tab.clone()).await {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to switch to new tab for request {}: {}", request_id, e);
                return Err(e);
            }
        }
        
        // Navigate to the requested URL
        match driver.goto(&url).await {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to navigate to URL for request {}: {}", request_id, e);
                
                // Try to close the tab even if navigation failed
                let _ = driver.close_window().await;
                
                return Err(e);
            }
        }
        
        // Wait a moment for page to fully load
        tokio::time::sleep(Duration::from_millis(1500)).await;
        
        // Get the page source
        let html = match driver.source().await {
            Ok(source) => source,
            Err(e) => {
                println!("Failed to get page source for request {}: {}", request_id, e);
                
                // Try to close the tab
                let _ = driver.close_window().await;
                
                return Err(e);
            }
        };
        
        // Clean up - close the tab
        match driver.close_window().await {
            Ok(_) => {},
            Err(e) => {
                println!("Warning: Failed to close tab for request {}: {}", request_id, e);
                // Continue anyway, since we have the HTML
            }
        }
        
        // Try to get available windows
        let windows = match driver.windows().await {
            Ok(w) => w,
            Err(e) => {
                println!("Warning: Failed to get window handles for request {}: {}", request_id, e);
                Vec::new()
            }
        };
        
        // If there are remaining windows, switch to the first one
        if !windows.is_empty() {
            if let Err(e) = driver.switch_to_window(windows[0].clone()).await {
                println!("Warning: Failed to switch to first window for request {}: {}", request_id, e);
            }
        }
        
        println!("Completed request: {}", request_id);
        Ok(html)
    }).await;
    
    // Handle any errors from the task itself
    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(WebDriverError::CustomError(format!("Task error: {}", e))),
    }
}

#[web::post("/bp")]
async fn bp(req_body: String) -> impl web::Responder {
    println!("Request body: {}", req_body);
    
    // Generate a unique request ID
    let request_id = generate_request_id();
    
    // Add the request to pending requests
    PENDING_REQUESTS.write().await.insert(request_id.clone(), true);
    
    // Get or create the WebDriver instance
    let driver = match get_or_create_driver().await {
        Ok(driver) => driver,
        Err(e) => {
            eprintln!("Error getting WebDriver: {}", e);
            // Clean up the pending request
            PENDING_REQUESTS.write().await.remove(&request_id);
            return web::HttpResponse::InternalServerError().body(format!("WebDriver error: {}", e));
        }
    };
    
    // Clone what we need
    let url = req_body.clone();
    let request_id_clone = request_id.clone();
    
    // Process the URL and get the result
    let result = process_url_in_tab(driver, url, request_id_clone).await;
    
    // Mark request as completed
    PENDING_REQUESTS.write().await.remove(&request_id);
    
    // Check if this was the last request and navigate to default if needed
    if PENDING_REQUESTS.read().await.is_empty() {
        if let Ok(driver) = get_or_create_driver().await {
            // Use the cleanup function to restore the default state
            let _ = cleanup_tab(&driver).await;
        }
    }
    
    // Return appropriate response
    match result {
        Ok(html) => web::HttpResponse::Ok().body(html),
        Err(e) => {
            eprintln!("Error processing URL: {}", e);
            web::HttpResponse::InternalServerError().body(format!("Processing error: {}", e))
        }
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| {
        web::App::new()
            .service(index)
            .service(bp)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}