use core::panic;

use ntex::web;
use thirtyfour::{common::print, prelude::*};
use std::time::Duration;
use std::process::{Command, Child};
use std::net::{TcpListener, SocketAddr};
use rand::{thread_rng, Rng};
use std::sync::Arc;

#[web::get("/")]
async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

// Structure to hold the GeckoDriver process and WebDriver
struct GeckoSession {
    driver: WebDriver,
    process: Child,
    port: u16,
}

impl GeckoSession {
    // Properly clean up resources when the session is done
    async fn quit(self) -> Result<(), Box<dyn std::error::Error>> {
        // Quit the WebDriver session first
        self.driver.quit().await?;
        
        // Then terminate the GeckoDriver process
        // We use drop here to consume self and make the compiler happy
        drop(self.process);
        
        Ok(())
    }
}

// Find an available port in the specified range
fn find_available_port() -> Option<u16> {
    let mut rng = thread_rng();
    for _ in 0..100 { // Try up to 100 times
        let port = rng.gen_range(3000..65535);
        if TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Creates a new WebDriver instance with its own GeckoDriver process
async fn create_driver() -> Result<GeckoSession, Box<dyn std::error::Error>> {
    // Find an available port
    let port = find_available_port().ok_or("Failed to find an available port")?;
    let port_str = port.to_string();
    
    // Start GeckoDriver on the selected port
    println!("Starting GeckoDriver on port {}", port);
    let process = Command::new("geckodriver")
        .arg("--port")
        .arg(&port_str)
        .spawn()
        .map_err(|e| format!("Failed to start GeckoDriver: {}", e))?;
    
    // Give GeckoDriver a moment to start up
    std::thread::sleep(Duration::from_secs(1));
    
    // Create WebDriver capabilities
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;
    
    // Add needed capabilities for running in CI environment
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    
    // Connect WebDriver to the GeckoDriver instance on our selected port
    let driver = WebDriver::new(&format!("http://localhost:{}", port), caps).await?;
    
    // Navigate to a blank page initially
    driver.goto("about:blank").await?;
    
    Ok(GeckoSession {
        driver,
        process,
        port,
    })
}

#[web::post("/bp")]
async fn bp(req_body: String) -> impl web::Responder {
    println!("Request body: {}", req_body);
    
    // Create a new GeckoDriver instance for this request with its own port
    let session = match create_driver().await {
        Ok(session) => session,
        Err(e) => {
            eprintln!("Error creating GeckoDriver session: {}", e);
            return web::HttpResponse::InternalServerError().body(format!("WebDriver error: {}", e));
        }
    };
    
    println!("Created GeckoDriver session on port {}", session.port);
    
    // Navigate to the requested URL
    if let Err(e) = session.driver.goto(&req_body).await {
        eprintln!("Error navigating to URL: {}", e);
        // Make sure to clean up resources
        let _ = session.quit().await;
        return web::HttpResponse::InternalServerError().body(format!("Navigation error: {}", e));
    }
    
    // Wait a moment for page to fully load
    std::thread::sleep(Duration::from_millis(500));
    
    // Get the page source
    let html = match session.driver.source().await {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error getting page source: {}", e);
            let _ = session.quit().await;
            return web::HttpResponse::InternalServerError().body(format!("Source error: {}", e));
        }
    };
    
    // Clean up resources (both WebDriver session and GeckoDriver process)
    if let Err(e) = session.quit().await {
        eprintln!("Error closing GeckoDriver session: {}", e);
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