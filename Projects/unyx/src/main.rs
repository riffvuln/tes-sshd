use color_eyre::Result;

mod rats;

fn main() -> Result<()> {
    color_eyre::install()?;
    std::thread::spawn(ratatui_term);
}


fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    
    let bot_log_clone = rat_app.bot_log.clone();
    
    std::thread::spawn(move ||{
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut bot_log = bot_log_clone.lock().unwrap();
        bot_log.push("Hello from the bot!".to_string());
    });
    
    let app_result = rat_app.run(terminal);
    ratatui::restore();
    app_result
}

