use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{thread_rng, Rng};
use regex::Regex;
use reqwest::Client;
use anyhow::{Result, Context};
use std::process::Command;
use tokio;

// Define constants
const DOWNLOADS_FOLDER: &str = "pypasser/reCaptchaV2/Downloads";

/// Parses the recaptcha anchor URL to extract endpoint and parameters
pub fn parse_url(anchor_url: &str) -> Option<HashMap<String, String>> {
    let regex = Regex::new(r"(?P<endpoint>[api2|enterprise]+)/anchor\?(?P<params>.*)").ok()?;
    
    regex.captures(anchor_url).map(|caps| {
        let mut result = HashMap::new();
        result.insert("endpoint".to_string(), caps.name("endpoint")?.as_str().to_string());
        result.insert("params".to_string(), caps.name("params")?.as_str().to_string());
        Some(result)
    })?
}

/// Creates a proxy dictionary with appropriate formatting
pub fn proxy_dict(
    proxy_type: &str,
    host: &str, 
    port: &str, 
    username: Option<&str>,
    password: Option<&str>
) -> HashMap<String, String> {
    let mut proxies = HashMap::new();
    
    if let (Some(username), Some(password)) = (username, password) {
        proxies.insert(
            "http".to_string(),
            format!("{}://{}:{}@{}:{}", 
                    proxy_type.replace("https", "http"),
                    username,
                    password,
                    host,
                    port)
        );
        
        proxies.insert(
            "https".to_string(),
            format!("{}://{}:{}@{}:{}", 
                    proxy_type,
                    username,
                    password,
                    host,
                    port)
        );
    } else {
        proxies.insert(
            "http".to_string(),
            format!("{}://{}:{}",
                    proxy_type.replace("https", "http"),
                    host,
                    port)
        );
        
        proxies.insert(
            "https".to_string(),
            format!("{}://{}:{}",
                    proxy_type,
                    host,
                    port)
        );
    }
    
    proxies
}

