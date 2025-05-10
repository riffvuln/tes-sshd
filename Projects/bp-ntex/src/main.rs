use core::panic;

use ntex::web;
use thirtyfour::{common::print, prelude::*};

#[web::get("/")]
async fn index() -> impl web::Responder {
    web::HttpResponse::Ok().body("Nyari apa bg?")
}

#[web::post("/bp")]
async fn bp(req_body: String) -> impl web::Responder {
    println!("Request body: {}", req_body);
    // l/ Initialize WebDriver capabilities
    let mut caps = DesiredCapabilities::firefox();

    // Configure user-agent and other preferences
    // let pref = FirefoxPreferences::new();
    // pref.set("general.useragent.override", rand_agents::user_agent())?;

    // Configure headless mode
    caps.set_headless().unwrap();

    // // Disable GPU and configure for non-GUI environment
    // pref.set("browser.display.use_document_fonts", 0).unwrap();
    // pref.set("browser.sessionstore.resume_from_crash", false).unwrap();
    // pref.set("gfx.font_rendering.fontconfig.max_generic_substitutions", 127).unwrap();
    
    let driver = match WebDriver::new("http://localhost:4444", caps).await {
        Ok(driver) => driver,
        Err(e) => {
            eprintln!("Error creating WebDriver session: {}", e);
            // return Ok(false);
            panic!("Error creating WebDriver session: {}", e);
        }
    };
    
    driver.goto(&req_body).await.unwrap();
    // Get the page source (HTML content)
    let html = driver.source().await.unwrap();
    println!("HTML: {}", html);
    // Close the browser session
    driver.quit().await.unwrap();

    // Return the HTML as the response
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