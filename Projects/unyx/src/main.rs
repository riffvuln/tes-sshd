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

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_idx != 0;
        if is_not_cursor_leftmost {
            let cur_idx = self.char_idx;
            let from_left_to_cur_idx = cur_idx - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_cur_idx);
            let after_char_to_delete = self.input.chars().skip(cur_idx);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        self.char_idx = 0;
    }

    fn submit_msg(&mut self) {
        self.bot_log.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn run(&mut self, terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| {
                
            })
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, bot_log_area] = vertical.areas(frame.area());
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Insert => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to exit, ".into(),
                    "Enter".bold(),
                    " to send.".bold(),
                ],
                Style::default(),
            )
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }
}