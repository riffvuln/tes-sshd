use core::panic;

use ntex::web;
use thirtyfour::{common::print, prelude::*};
use std::sync::{Mutex, Arc, Once};
use lazy_static::lazy_static;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
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
}

async fn get_or_create_driver() -> Result<WebDriver, WebDriverError> {
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
    
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    
    // Navigate to a blank page initially
    driver.goto("about:blank").await?;
    
    Ok(driver)
}

async fn reset_driver(driver: &WebDriver) -> Result<(), WebDriverError> {
    // Clear cookies
    driver.delete_all_cookies().await?;
    
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
    
    // Create a new tab
    let tab = driver.new_tab().await?;
    let tab_id = tab.to_string(); // Convert tab handle to string for logging
    
    // Switch to the new tab
    driver.switch_to_window(tab.clone()).await?;
    
    println!("Created new tab: {}", tab_id);
    
    // Navigate to the requested URL
    driver.goto(&url).await?;
    
    // Wait a moment for page to fully load
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Get the page source
    let html = driver.source().await?;
    
    // Clean up - close the tab
    driver.close_window().await?;
    
    // Mark request as completed
    PENDING_REQUESTS.write().await.remove(&request_id);
    
    // If no more pending requests, navigate to default page
    if PENDING_REQUESTS.read().await.is_empty() {
        // Navigate back to default window if there's one
        if let Ok(windows) = driver.windows().await {
            if !windows.is_empty() {
                driver.switch_to_window(windows[0].clone()).await?;
                driver.goto("about:blank").await?;
            }
        }
    }
    
    println!("Completed request: {}", request_id);
    Ok(html)
}

#[web::post("/bp")]
async fn bp(req_body: String) -> impl web::Responder {
    println!("Request body: {}", req_body);
    
    // Get or create the WebDriver instance
    let driver = match get_or_create_driver().await {
        Ok(driver) => driver,
        Err(e) => {
            eprintln!("Error getting WebDriver: {}", e);
            return web::HttpResponse::InternalServerError().body(format!("WebDriver error: {}", e));
        }
    };
    
    // Generate a unique request ID
    let request_id = generate_request_id();
    
    // Add the request to pending requests
    PENDING_REQUESTS.write().await.insert(request_id.clone(), true);
    
    // Clone what we need to move into the task
    let driver_clone = driver.clone();
    let url = req_body.clone();
    let request_id_clone = request_id.clone();
    
    // Process the URL in a separate tokio task
    let result = task::spawn(async move {
        process_url_in_tab(driver_clone, url, request_id_clone).await
    }).await;
    
    // Handle the result
    match result {
        Ok(Ok(html)) => web::HttpResponse::Ok().body(html),
        Ok(Err(e)) => {
            eprintln!("Error processing URL: {}", e);
            // Clean up the pending request if there was an error
            PENDING_REQUESTS.write().await.remove(&request_id);
            web::HttpResponse::InternalServerError().body(format!("Processing error: {}", e))
        },
        Err(e) => {
            eprintln!("Task join error: {}", e);
            // Clean up the pending request if there was an error
            PENDING_REQUESTS.write().await.remove(&request_id);
            web::HttpResponse::InternalServerError().body(format!("Task error: {}", e))
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