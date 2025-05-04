use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    
    Ok(())
}


fn ratatui_term() {
    let terminal = ratatui::init();
    
}

struct RatApp {
    input: String,
    char_idx: usize,
    input_mode: InputMode,
    bot_log: Vec<String>,
    server_msgs: Vec<String>,
}

enum InputMode {
    Normal,
    Insert,
}

impl RatApp {
    fn new() -> Self {
        Self {
            input: String::new(),
            char_idx: 0,
            input_mode: InputMode::Normal,
            bot_log: Vec::new(),
            server_msgs: Vec::new(),
        }
    }

    fn move_cursor_left(&mut self) {
        let new_idx = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor(new_idx);
    }

    fn move_cursor_right(&mut self) {
        let new_idx = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor(new_idx);
    }

    fn enter_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.input.insert(idx, c);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}