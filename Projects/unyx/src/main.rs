use color_eyre::Result;

mod rats;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui_term()
}


fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    
    let app_result = rat_app.run(terminal);
    ratatui::restore();
    app_result
}

