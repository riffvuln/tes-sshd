use color_eyre::Result;

mod rats;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui_term()
}


fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let app_result = rats::RatApp::new().run(terminal);
    ratatui::restore();
    app_result
}

