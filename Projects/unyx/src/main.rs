use color_eyre::Result;

mod rats;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui_term()
}


fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    
    // Create an Arc<Mutex<>> to share mutable state between threads
    let bot_log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let bot_log_clone = bot_log.clone();
    
    std::thread::spawn(move ||{
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut log = bot_log_clone.lock().unwrap();
        log.push("Hello from the bot!".to_string());
    });
    
    let app_result = rat_app.run(terminal, bot_log);
    ratatui::restore();
    app_result
}

