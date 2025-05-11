use std::sync::{Arc, Mutex, Once};
use std::time::Duration;
use lazy_static::lazy_static;
use thirtyfour::prelude::*;
use crate::utils::{DEFAULT_PAGE, WEBDRIVER_URL};

// Global static reference to the WebDriver instance
lazy_static! {
    // The WebDriver instance shared across all requests
    pub static ref DRIVER: Arc<Mutex<Option<WebDriver>>> = Arc::new(Mutex::new(None));
    pub static ref INIT: Once = Once::new();
}

/// Create a new WebDriver instance with the desired capabilities
pub async fn create_driver() -> Result<WebDriver, WebDriverError> {
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

/// Get the existing WebDriver instance or create a new one
pub async fn get_or_create_driver() -> Result<WebDriver, WebDriverError> {
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

/// Clean up the WebDriver state
pub async fn reset_browser_state(driver: &WebDriver) -> Result<(), WebDriverError> {
    println!("Resetting browser state...");
    
    // Clear cookies
    if let Err(e) = driver.delete_all_cookies().await {
        eprintln!("Warning: Failed to delete cookies: {}", e);
    }
    
    // Navigate back to default page
    driver.goto(DEFAULT_PAGE).await?;
    
    Ok(())
}
