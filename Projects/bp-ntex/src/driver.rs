use std::sync::{Arc, Mutex, Once};
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;
use lazy_static::lazy_static;
use thirtyfour::prelude::*;
use crate::utils::{DEFAULT_PAGE, WEBDRIVER_URL};

// Global static reference to the WebDriver instance
lazy_static! {
    // The WebDriver instance shared across all requests
    pub static ref DRIVER: Arc<Mutex<Option<WebDriver>>> = Arc::new(Mutex::new(None));
    pub static ref INIT: Once = Once::new();
    // Async mutex for driver creation
    pub static ref DRIVER_CREATION_LOCK: Arc<TokioMutex<()>> = Arc::new(TokioMutex::new(()));
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
    // First, check if driver already exists without acquiring the heavy creation lock
    {
        let driver_option = DRIVER.lock().unwrap().clone();
        if let Some(driver) = driver_option {
            // Driver exists, check if it's still valid
            match driver.title().await {
                Ok(_) => return Ok(driver), // Driver is responsive, return early
                Err(e) => {
                    println!("Driver became unresponsive: {}", e);
                    // We'll create a new one, but we need the lock first
                }
            }
        }
    }
    
    // At this point, either no driver exists or it's unresponsive
    // Acquire the creation lock to ensure only one thread creates a driver
    let _lock = DRIVER_CREATION_LOCK.lock().await;
    
    // Check again after acquiring lock (another thread might have created it while we were waiting)
    {
        let driver_option = DRIVER.lock().unwrap().clone();
        if let Some(driver) = driver_option {
            // Try again to see if this driver is valid
            match driver.title().await {
                Ok(_) => return Ok(driver), // Driver is responsive
                Err(_) => {
                    // Will continue to create a new one
                    println!("Creating a new WebDriver instance...");
                }
            }
        } else {
            println!("Creating WebDriver for the first time");
        }
    }
    
    // Create the driver
    let new_driver = create_driver().await?;
    
    // Store it for future use
    *DRIVER.lock().unwrap() = Some(new_driver.clone());
    
    Ok(new_driver)
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
