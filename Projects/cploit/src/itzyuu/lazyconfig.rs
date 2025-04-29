use colored::Colorize;
use rayon::prelude::*;
use reqwest::{Client, header};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;
use rand::seq::SliceRandom;
use std::error::Error;

pub struct LazyConfig {
    paths: Vec<String>,
    websites: Vec<String>,
    pool_size: usize,
    user_agents: Vec<String>,
}

impl LazyConfig {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        println!("[+] give your path list config: ");
        let mut path_file = String::new();
        io::stdin().read_line(&mut path_file)?;
        path_file = path_file.trim().to_string();

        let paths = fs::read_to_string(path_file)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        println!("[+] submit your weblist: ");
        let mut list_path = String::new();
        io::stdin().read_line(&mut list_path)?;
        list_path = list_path.trim().to_string();

        let websites = fs::read_to_string(list_path)?
            .lines()
            .map(|s| format!("http://{}", s.trim()))
            .collect();

        println!("[+] enter the pool threads: ");
        let mut pool_size = String::new();
        io::stdin().read_line(&mut pool_size)?;
        let pool_size = pool_size.trim().parse::<usize>().unwrap_or(10);

        let user_agents = BufReader::new(File::open("user-agent.txt")?)
            .lines()
            .collect::<Result<Vec<String>, _>>()?;

        // Ensure output directory exists
        if !Path::new("output").exists() {
            fs::create_dir("output")?;
        }

        Ok(LazyConfig {
            paths,
            websites,
            pool_size,
            user_agents,
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        // Create a tokio runtime
        let rt = Runtime::new()?;
        
        // Set up a Reqwest client with rustls-tls
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .use_rustls_tls()
            .build()?;
        
        let client = Arc::new(client);
        let paths = Arc::new(self.paths.clone());
        let user_agents = Arc::new(self.user_agents.clone());

        // Use Rayon for parallelization
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.pool_size)
            .build_global()?;

        rt.block_on(async {
            let tasks: Vec<_> = self.websites.par_iter().map(|site| {
                let site = site.clone();
                let client = client.clone();
                let paths = paths.clone();
                let user_agents = user_agents.clone();
                
                tokio::spawn(async move {
                    Self::check_site(&site, &paths, &client, &user_agents).await;
                })
            }).collect();

            // Wait for all tasks to complete
            for task in tasks {
                let _ = task.await;
            }

            Ok(())
        })
    }

    async fn check_site(
        site: &str, 
        paths: &[String], 
        client: &Client, 
        user_agents: &[String]
    ) {
        // Select a random user agent
        let user_agent = user_agents
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"Mozilla/5.0".to_string())
            .to_string();

        // Create headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&user_agent).unwrap_or_else(|_| header::HeaderValue::from_static("Mozilla/5.0")),
        );

        for path in paths {
            let url = format!("{}{}", site, path.trim());
            
            match client.get(&url).headers(headers.clone()).timeout(std::time::Duration::from_secs(10)).send().await {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => {
                            if text.contains("DB_HOST") {
                                println!("{}", format!("[Found Config] {}", site).green());
                                if let Err(e) = Self::append_to_file("output/configfound.txt", &format!("{}{}\n", site, path.trim())) {
                                    eprintln!("Error writing to file: {}", e);
                                }
                            } else if text.contains("save_before_upload") || text.contains("uploadOnSave") {
                                println!("{}", format!("[Found FTP] {}", site).green());
                                if let Err(e) = Self::append_to_file("output/sftpfound.txt", &format!("{}{}\n", site, path.trim())) {
                                    eprintln!("Error writing to file: {}", e);
                                }
                            } else {
                                println!("{}", format!("[Not Found] {}{}", site, path.trim()).red());
                            }
                        },
                        Err(_) => {
                            println!("{}", format!("[Unknown Error] {}{}", site, path.trim()).yellow());
                        }
                    }
                },
                Err(_) => {
                    println!("{}", format!("[Unknown Error] {}{}", site, path.trim()).yellow());
                }
            }
        }
    }

    fn append_to_file(path: &str, content: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

// Function to run the tool
pub fn run_lazy_config() -> Result<(), Box<dyn Error>> {
    let config = LazyConfig::new()?;
    config.run()
}
