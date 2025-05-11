use rand::Rng;

// Constants for configuration
pub const DEFAULT_PAGE: &str = "about:blank";
pub const PAGE_LOAD_WAIT_MS: u64 = 1500;
pub const WEBDRIVER_URL: &str = "http://localhost:4444";

/// Generate a unique request ID for tracking
pub fn generate_request_id() -> String {
    let mut rng = rand::thread_rng();
    let random_num: u64 = rng.gen_range(100000..999999);
    format!("req-{}", random_num)
}
