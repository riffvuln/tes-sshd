use core::panic;

use ntex::web;
use thirtyfour::{common::print, prelude::*};
use std::time::Duration;

#[web::get("/")]
async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

/// Creates a new WebDriver instance for a request
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
    
    // Navigate to the requested URL
    if let Err(e) = driver.goto(&req_body).await {
        eprintln!("Error navigating to URL: {}", e);
        return web::HttpResponse::InternalServerError().body(format!("Navigation error: {}", e));
    }
    
    // Wait a moment for page to fully load
    std::thread::sleep(Duration::from_millis(500));
    
    // Get the page source
    let html = match driver.source().await {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error getting page source: {}", e);
            return web::HttpResponse::InternalServerError().body(format!("Source error: {}", e));
        }
    };
    
    // Reset the driver for the next request
    if let Err(e) = reset_driver(&driver).await {
        eprintln!("Error resetting driver: {}", e);
        // Continue anyway
    }
    
    // Return the HTML response
    web::HttpResponse::Ok().body(html)
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