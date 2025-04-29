use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::io;
use std::sync::{Arc, Mutex};
use std::error::Error;

#[tokio::main]
pub async fn extract_cpanel() -> Result<(), Box<dyn Error>> {
    // Get filename from user input
    println!("[+] submit file want to be procces: ");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename)?;
    let filename = filename.trim();

    // Open input file
    let file = File::open(filename).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Create output directory if it doesn't exist
    tokio::fs::create_dir_all("output").await?;
    
    // Open output file
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output/dbforcp.txt")
        .await?;
    let output_file = Arc::new(Mutex::new(tokio::io::BufWriter::new(output_file)));

    // Process the file
    let mut data = String::new();
    
    while let Some(line_result) = lines.next_line().await? {
        let line = line_result.trim();
        
        if line.is_empty() {
            continue;
        }
        
        if !line.contains('|') {
            data.push_str(line);
        } else {
            data.push_str(line);
            let data_clone = data.clone();
            
            // Use rayon for CPU-bound processing
            let result = process_line(&data_clone)?;
            
            if let Some(processed_line) = result {
                let mut file_guard = output_file.lock().unwrap();
                file_guard.write_all(processed_line.as_bytes()).await?;
            }
            
            data.clear();
        }
    }
    
    // Check if there's any remaining data
    if !data.is_empty() {
        println!("Data tidak lengkap: {}", data);
    }
    
    // Ensure all data is flushed to disk
    let mut file_guard = output_file.lock().unwrap();
    file_guard.flush().await?;
    
    Ok(())
}

// CPU-bound processing function suitable for rayon
fn process_line(data: &str) -> Result<Option<String>, Box<dyn Error>> {
    let data_parts: Vec<&str> = data.split('|').collect();
    
    if data_parts.len() >= 4 {
        // Process domain to remove http:// or https:// and get the first part
        let domain = data_parts[0]
            .replace("http://", "")
            .replace("https://", "")
            .split('/')
            .next()
            .unwrap_or("")
            .to_string();
        
        let data_pilihan = vec![domain, data_parts[2].to_string(), data_parts[3].to_string()];
        let data_pilihan_string = format!("{}|\n", data_pilihan.join("|"));
        
        Ok(Some(data_pilihan_string))
    } else {
        Ok(None)
    }
}
