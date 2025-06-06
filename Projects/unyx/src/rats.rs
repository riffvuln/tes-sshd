use color_eyre::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

use crate::azal::CommandType;


pub struct RatApp {
    input: String,
    char_idx: usize,
    input_mode: InputMode,
    pub bot_log: Arc<Mutex<Vec<String>>>,
    pub server_msgs: Arc<Mutex<Vec<String>>>,
}

enum InputMode {
    Normal,
    Insert,
}

impl RatApp {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            char_idx: 0,
            input_mode: InputMode::Normal,
            bot_log: Arc::new(Mutex::new(Vec::new())),
            server_msgs: Arc::new(Mutex::new(Vec::new())),
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

    fn submit_msg(&mut self, tx_input: &std::sync::mpsc::Sender<CommandType>) {
        self.process_command(tx_input);
        if let Ok(mut bot_log) = self.bot_log.lock() {
            bot_log.push(self.input.clone());
        }
        self.input.clear();
        self.reset_cursor();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal, tx_input: std::sync::mpsc::Sender<CommandType>) -> Result<()> {
        
        let mut last_draw = Instant::now();
        let draw_interval = Duration::from_millis(200);
        
        loop {
            let now = Instant::now();
            if now.duration_since(last_draw) >= draw_interval {
                let _ = terminal.draw(|frame| {
                    self.draw(frame);
                });
                last_draw = now;
            }
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('i') => {
                                self.input_mode = InputMode::Insert;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Insert if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => self.submit_msg(&tx_input),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },
                        InputMode::Insert => {}
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, logs_area] = vertical.areas(frame.area());
        
        // Split the logs area horizontally for bot_log and server_msgs
        let horizontal = Layout::horizontal([
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ]);
        let [bot_log_area, server_msgs_area] = horizontal.areas(logs_area);
        
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "i".bold(),
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
                InputMode::Insert => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Insert => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.char_idx as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        // Bot Log section
        let bot_messages: Vec<ListItem> = if let Ok(mut bot_log) = self.bot_log.lock() {
            if bot_log.len() > 10 {
                bot_log.clear();
            }
            bot_log.iter()
                .map(|m| {
                    let content = Line::from(Span::raw(format!("{m}")));
                    ListItem::new(content)
                })
                .collect()
        } else {
            Vec::new()
        };
        let bot_messages_list = List::new(bot_messages).block(Block::bordered().title("Bot Log"));
        frame.render_widget(bot_messages_list, bot_log_area);
        
        // Server Messages section
        let server_messages: Vec<ListItem> = if let Ok(mut server_msgs) = self.server_msgs.lock() {
            if server_msgs.len() > 10 {
                server_msgs.clear();
            }
            server_msgs.iter()
                .map(|m| {
                    let content = Line::from(Span::raw(format!("{m}")));
                    ListItem::new(content)
                })
                .collect()
        } else {
            Vec::new()
        };
        let server_messages_list = List::new(server_messages).block(Block::bordered().title("Server Messages"));
        frame.render_widget(server_messages_list, server_msgs_area);
    }

    fn process_command(&mut self, tx_input: &std::sync::mpsc::Sender<CommandType>) -> Result<bool> {
        // Skip if input is empty
        if self.input.is_empty() {
            return Ok(false);
        }

        // Check for command prefix
        if let Some(cmd) = self.input.split_whitespace().next() {
            let args = self.input.trim_start_matches(cmd).trim().to_string();
            
            // Match command to CommandType
            let command = match cmd.to_lowercase().as_str() {
                "chat" => Some(CommandType::Chat(args)),
                "goto" => Some(CommandType::Goto(args)),
                "mobkillaura" => {
                    let mut enabled = false;
                    if args == "on" {
                        enabled = true;
                    } else if args == "off" {
                        enabled = false;
                    }
                    Some(CommandType::Mobkillaura(enabled))
                }
                "mine" => Some(CommandType::Mine(args)),
                // Add more command mappings here as needed
                _ => None,
            };

            
            // Process if valid command was found
            if let Some(command) = command {
                tx_input.send(command)?;

            }
        }
        
        Ok(false)
    }
}