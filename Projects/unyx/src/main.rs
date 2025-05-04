use color_eyre::Result;

mod rats;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui_term()
}


fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    std::thread::spawn(||{
        std::thread::sleep(std::time::Duration::from_secs(2));
        rat_app.bot_log.push("Hello from the bot!".to_string());
    });
    let app_result = rat_app.run(terminal);
    ratatui::restore();
    app_result
}

