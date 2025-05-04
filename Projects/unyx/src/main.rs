use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

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

