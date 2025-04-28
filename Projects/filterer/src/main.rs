use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use colored::*;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;

fn print_banner() {
    let banner = r#"
____ _    ____ ____ _  _ _   _ ____ _    ____ ____ _  _ 
|___ |    |__| [__  |__|  \_/  |___ |    |__| [__  |__| 
|    |___ |  | ___] |  |   |   |    |___ |  | ___] |  | 
                                                        
    "#;
    println!("{}", banner.magenta().on_black().bold());
    thread::sleep(Duration::from_secs(1));
}

fn sanitize_filename(filename: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\w\-_\. ]").unwrap();
    }
    RE.replace_all(filename, "_").to_string()
}

fn process_file(
    input_file: &str,
    keywords: &[&str],
    output_dir: &str,
) -> io::Result<()> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Prepare keyword patterns
    let keyword_patterns: HashMap<&str, Regex> = keywords
        .iter()
        .map(|&k| (k, Regex::new(&regex::escape(k)).unwrap()))
        .collect();

    // Read file contents
    let file = File::open(input_file)?;
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    // Prepare output files with BufWriter
    let file_writers: Arc<Mutex<HashMap<String, BufWriter<File>>>> = 
        Arc::new(Mutex::new(HashMap::new()));

    println!("\n{}\n", "🚀 Processing started... Please wait!".yellow());

    // Process lines in parallel
    lines.par_iter().enumerate().for_each(|(idx, line)| {
        // Check for each keyword
        for (&keyword, pattern) in &keyword_patterns {
            if pattern.is_match(line) {
                let sanitized_keyword = sanitize_filename(keyword);
                let output_path = format!("{}/{}_{}", output_dir, sanitized_keyword, "results.txt");
                
                let mut writers = file_writers.lock().unwrap();
                
                // Get or create writer
                if !writers.contains_key(&output_path) {
                    let file = File::create(&output_path).unwrap();
                    writers.insert(output_path.clone(), BufWriter::new(file));
                }
                
                if let Some(writer) = writers.get_mut(&output_path) {
                    writeln!(writer, "{}", line).unwrap();
                }
            }
        }
    });
    
    println!("\n{}\n", "🎉 Processing complete!".green());

    // Flush and close all writers
    let mut writers = file_writers.lock().unwrap();
    for (_, writer) in writers.iter_mut() {
        writer.flush()?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    print_banner();

    let input_file = "LOGS.txt";
    let keywords = [
        ":2082", ":2083", ":2086", ":2087", ":2096", ":8443", ":5000", ":2222", 
        ":5555", ":8888", "/administrator/index.php", "ftp.", "smtp", "/adminer.php", 
        "/typo3", "/adminXYZ", "/user/login", "/bitrix/admin/", "/index.php/login/", 
        "/zp-core/admin.php", "/admin/index.php", "/login", ":8090", ":8080", 
        ":8433", ":2443", ":3306", "", ":8083", ":8443", ":2030", ":2031", "whmcs", 
        "/RDWeb/", "/admin/login", "/admin/", "phpMyAdmin", "phpmyadmin", 
        "/wp-login.php", "/wp-admin", "wordpress", "aapanel", "cpanel",
    ];
    let output_dir = "LOGS FILTER";

    process_file(input_file, &keywords, output_dir)
}
