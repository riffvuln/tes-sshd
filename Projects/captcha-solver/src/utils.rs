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

/// Downloads audio from a given link and returns the file path
pub async fn download_audio(link: &str) -> Result<PathBuf> {
    // Generate a unique filename
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_secs();
    
    let random_num: u32 = thread_rng().gen_range(10000..100000);
    let file_name = format!("{}_{}.mp3", timestamp, random_num);
    
    // Create the downloads directory if it doesn't exist
    let downloads_dir = Path::new(DOWNLOADS_FOLDER);
    fs::create_dir_all(downloads_dir)
        .context("Failed to create downloads directory")?;
    
    let file_path = downloads_dir.join(file_name);
    
    // Download the file
    let response = Client::new()
        .get(link)
        .send()
        .await
        .context("Failed to download audio")?;
    
    let content = response.bytes()
        .await
        .context("Failed to read response content")?;
    
    // Write the content to file
    let mut file = File::create(&file_path)
        .context("Failed to create file")?;
    file.write_all(&content)
        .context("Failed to write to file")?;
    
    Ok(file_path)
}

/// Converts an MP3 file to WAV format and returns the new file path
pub async fn convert_to_wav(file_path: PathBuf) -> Result<PathBuf> {
    // Create the output wav file path by replacing the mp3 extension
    let wav_path_str = file_path
        .to_str()
        .context("Invalid file path")?
        .replace(".mp3", ".wav");
    
    let wav_file_path = PathBuf::from(wav_path_str);
    
    // Use ffmpeg to convert the file (similar to how AudioSegment works in Python)
    // Note: This assumes ffmpeg is installed on the system
    let status = Command::new("ffmpeg")
        .args([
            "-i", file_path.to_str().unwrap(),
            "-acodec", "pcm_s16le",
            "-ar", "44100",
            wav_file_path.to_str().unwrap()
        ])
        .status()
        .context("Failed to execute ffmpeg")?;
    
    if !status.success() {
        anyhow::bail!("ffmpeg conversion failed");
    }
    
    // Remove the original mp3 file
    fs::remove_file(file_path).context("Failed to remove mp3 file")?;
    
    Ok(wav_file_path)
}
