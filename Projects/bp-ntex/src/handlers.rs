use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, Mutex as TokioMutex};
use tokio::task;
use lazy_static::lazy_static;
use ntex::web;
use thirtyfour::prelude::WebDriverError;

use crate::utils::generate_request_id;
use crate::driver::{get_or_create_driver, reset_browser_state};
use crate::browser::process_url_in_tab;

// Global state for request tracking
lazy_static! {
    // Track pending requests to know when to return to default page
    pub static ref PENDING_REQUESTS: Arc<RwLock<HashMap<String, bool>>> = Arc::new(RwLock::new(HashMap::new()));
    // Lock to synchronize WebDriver operations
    pub static ref DRIVER_LOCK: Arc<TokioMutex<()>> = Arc::new(TokioMutex::new(()));
}

/// Main index route handler
#[web::get("/")]
pub async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

/// Handle POST requests to /bp endpoint
#[web::post("/bp")]
pub async fn bp(req_body: String) -> impl web::Responder {
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
