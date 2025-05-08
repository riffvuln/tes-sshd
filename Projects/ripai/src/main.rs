use std::error::Error;
use std::process::Command;
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if lynx is installed
    if let Err(_) = Command::new("lynx").arg("--version").output() {
        eprintln!("Error: lynx is not installed or not in PATH");
        eprintln!("Please install lynx with your package manager (e.g., apt install lynx)");
        std::process::exit(1);
    }

    // Use lynx to get the links from Google search
    let output = Command::new("lynx")
        .arg("-listonly")
        .arg("-dump")
        .arg("https://www.google.com/search?q=Rust&oq=Rust")
        .output()?;

    if !output.status.success() {
        eprintln!("Error running lynx: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    let lynx_output = String::from_utf8(output.stdout)?;
    
    // Use regex to extract URLs from Google redirects
    let re = Regex::new(r"https://www\.google\.com/url\?q=([^&]+)")?;

    for line in lynx_output.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(url_match) = captures.get(1) {
                // URL decode the extracted URL
                let decoded_url = urlencoding::decode(url_match.as_str())?;
                println!("{}", decoded_url);
            }
        }
    }

    Ok(())
}