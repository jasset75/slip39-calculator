use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use slip39_calculator::{decode, encode, wordlist};
use std::{error::Error, io};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Word,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    Startup,
    Running,
}

/// TUI Application state
pub struct App {
    /// Current input in the search field
    pub input: String,
    /// Filtered suggestions based on input
    pub suggestions: Vec<String>,
    /// Index of the selected suggestion in the carousel
    pub suggestion_index: usize,
    /// List of words added by the user (max 20)
    pub saved_words: Vec<String>,
    /// Index of the currently selected saved word (for viewing grid)
    pub saved_index: Option<usize>,
    /// Complete wordlist reference
    pub all_words: Vec<String>,
    /// Paper mode (don't accumulate words)
    pub paper_mode: bool,

    // New Fields
    state: AppState,
    input_mode: Option<InputMode>, // None during Startup
    modal_selection: InputMode,    // Which mode is highlighted in the modal
}

impl App {
    pub fn new(paper_mode: bool, mode: Option<InputMode>) -> Self {
        let (state, input_mode) = if let Some(m) = mode {
            (AppState::Running, Some(m))
        } else {
            (AppState::Startup, None)
        };

        Self {
            input: String::new(),
            suggestions: Vec::new(),
            suggestion_index: 0,
            saved_words: Vec::new(),
            saved_index: None,
            all_words: wordlist().iter().map(|s| s.to_string()).collect(),
            paper_mode,
            state,
            input_mode,
            modal_selection: InputMode::Word, // Default selection
        }
    }

    /// Update suggestions based on input
    pub fn update_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions = self.all_words.clone();
        } else {
            let query = self.input.to_lowercase();
            self.suggestions = self
                .all_words
                .iter()
                .filter(|w| w.starts_with(&query))
                .cloned()
                .collect();
        }

        // Reset index safely
        if self.suggestions.is_empty() || self.suggestion_index >= self.suggestions.len() {
            self.suggestion_index = 0;
        }
    }

    pub fn add_current_word(&mut self) {
        let word_to_add = match self.input_mode {
            Some(InputMode::Binary) => {
                if self.input.len() == 10 {
                    decode(&self.input).ok()
                } else {
                    None
                }
            }
            Some(InputMode::Word) | None => self.suggestions.get(self.suggestion_index).cloned(),
        };

        if let Some(word) = word_to_add {
            if self.paper_mode {
                self.saved_words.clear();
                self.saved_words.push(word);
                self.saved_index = Some(0);
            } else if self.saved_words.len() < 20 {
                self.saved_words.push(word);
                self.saved_index = Some(self.saved_words.len() - 1);
            }
            // Clear input after adding
            self.input.clear();
            self.update_suggestions();
        }
    }
}

pub fn run(paper_mode: bool, mode: Option<InputMode>) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(paper_mode, mode);
    app.update_suggestions(); // Init suggestions

    // Run loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()>
where
    B::Error: Into<io::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app)).map_err(|e| e.into())?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.state {
                    AppState::Startup => match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Left | KeyCode::Right => {
                            app.modal_selection = match app.modal_selection {
                                InputMode::Word => InputMode::Binary,
                                InputMode::Binary => InputMode::Word,
                            };
                        }
                        KeyCode::Enter => {
                            app.input_mode = Some(app.modal_selection);
                            app.state = AppState::Running;
                        }
                        _ => {}
                    },
                    AppState::Running => {
                        match key.code {
                            KeyCode::Esc => return Ok(()),

                            // Input handling
                            KeyCode::Char(c) => {
                                // Always switch to input mode (deselect history) when typing
                                app.saved_index = None;

                                match app.input_mode {
                                    Some(InputMode::Binary) => {
                                        if (c == '0' || c == '1') && app.input.len() < 10 {
                                            app.input.push(c);
                                        }
                                    }
                                    Some(InputMode::Word) | None => {
                                        app.input.push(c);
                                        app.update_suggestions();
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                // Always switch to input mode (deselect history) when editing
                                app.saved_index = None;

                                app.input.pop();
                                match app.input_mode {
                                    Some(InputMode::Word) | None => app.update_suggestions(),
                                    _ => {}
                                }
                            }

                            // Carousel Navigation
                            KeyCode::Left => {
                                // Switch to suggestion view
                                app.saved_index = None;

                                if app.input_mode == Some(InputMode::Word)
                                    && !app.suggestions.is_empty()
                                {
                                    if app.suggestion_index > 0 {
                                        app.suggestion_index -= 1;
                                    } else {
                                        app.suggestion_index = app.suggestions.len() - 1;
                                        // Wrap around
                                    }
                                }
                            }
                            KeyCode::Right => {
                                // Switch to suggestion view
                                app.saved_index = None;

                                if app.input_mode == Some(InputMode::Word)
                                    && !app.suggestions.is_empty()
                                {
                                    if app.suggestion_index < app.suggestions.len() - 1 {
                                        app.suggestion_index += 1;
                                    } else {
                                        app.suggestion_index = 0; // Wrap around
                                    }
                                }
                            }

                            // Saved words Navigation
                            KeyCode::Up => {
                                if !app.saved_words.is_empty() {
                                    if let Some(curr) = app.saved_index {
                                        if curr > 0 {
                                            app.saved_index = Some(curr - 1);
                                        }
                                    } else {
                                        app.saved_index = Some(app.saved_words.len() - 1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if !app.saved_words.is_empty() {
                                    if let Some(curr) = app.saved_index {
                                        if curr < app.saved_words.len() - 1 {
                                            app.saved_index = Some(curr + 1);
                                        } else {
                                            app.saved_index = None; // Exit review mode
                                        }
                                    } else {
                                        app.saved_index = Some(0);
                                    }
                                }
                            }

                            // Selection
                            KeyCode::Enter => {
                                app.add_current_word();
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Suggestions Carousel
            Constraint::Min(10),   // Main Grid
            Constraint::Length(3), // Input & Help
            Constraint::Length(2), // Disclaimer (2 lines)
        ])
        .split(f.area());

    render_carousel(f, app, chunks[0]);
    render_grid(f, app, chunks[1]);
    render_input(f, app, chunks[2]);

    // Render Disclaimer Footer
    let disclaimer = "Note: Stateless mode encodes data using the SLIP-39 format,\nbut generated phrases are independent and cannot be combined for recovery.";
    let footer = Paragraph::new(disclaimer)
        .style(Style::default().fg(Color::Yellow).bg(Color::Reset))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(footer, chunks[3]);

    if app.state == AppState::Startup {
        render_modal(f, app, f.area());
    }
}

fn render_carousel(f: &mut Frame, app: &App, area: Rect) {
    // If in Binary Mode, we can use this area to show the "Decoded Word" when complete
    if app.input_mode == Some(InputMode::Binary) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Decoded Word ");

        // Check if input is valid 10-bit
        let content = if app.input.len() == 10 {
            match decode(&app.input) {
                Ok(w) => Span::styled(
                    format!("[ {} ]", w.to_uppercase()),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Err(_) => Span::styled("Invalid Binary", Style::default().fg(Color::Red)),
            }
        } else {
            Span::raw("Enter 10 bits...")
        };

        let p = Paragraph::new(content)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Suggestions ");

    let mut spans = Vec::new();

    if !app.suggestions.is_empty() {
        let window_size = 7;
        let start = app.suggestion_index.saturating_sub(window_size / 2);
        let end = (start + window_size).min(app.suggestions.len());
        // Adjust start if near end
        let start = if end == app.suggestions.len() {
            end.saturating_sub(window_size)
        } else {
            start
        };

        for i in start..end {
            let word = &app.suggestions[i];
            let is_selected = i == app.suggestion_index;

            if i > start {
                spans.push(Span::raw("   "));
            }

            if is_selected {
                spans.push(Span::styled(
                    format!("[ {} ]", word),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw(word));
            }
        }
    } else {
        spans.push(Span::raw("No matches"));
    }

    let p = Paragraph::new(Line::from(spans))
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(p, area);
}

fn render_grid(f: &mut Frame, app: &App, area: Rect) {
    let (word, index, binary) = if let Some(idx) = app.saved_index {
        // Viewing history (Priority)
        let w = &app.saved_words[idx];
        let i = app.all_words.iter().position(|x| x == w).unwrap_or(0);
        let b = encode(w).unwrap_or_else(|_| "0000000000".to_string());
        (Some(w.clone()), Some(i), Some(b))
    } else if app.input_mode == Some(InputMode::Binary) {
        // Live Binary Input
        // If we have input, show it.
        // If 10 bits, show word.
        let b = app.input.clone();
        let (w, i) = if b.len() == 10 {
            match decode(&b) {
                Ok(word) => {
                    let idx = app.all_words.iter().position(|x| x == &word).unwrap_or(0);
                    (Some(word), Some(idx))
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };
        (w, i, if b.is_empty() { None } else { Some(b) })
    } else {
        // Word Mode (Show current suggestion if available)
        if !app.suggestions.is_empty() && app.suggestion_index < app.suggestions.len() {
            let w = &app.suggestions[app.suggestion_index];
            let i = app.all_words.iter().position(|x| x == w).unwrap_or(0);
            let b = encode(w).unwrap_or_else(|_| "0000000000".to_string());
            (Some(w.clone()), Some(i), Some(b))
        } else {
            (None, None, None)
        }
    };

    let base_color = if app.paper_mode {
        Color::Red
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(base_color).add_modifier(Modifier::BOLD))
        .title(" Memory Grid ");

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let bit_values = [512, 256, 128, 64, 32, 16, 8, 4, 2, 1];

    let center_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(6), // Grid height
            Constraint::Length(2), // Word Index
            Constraint::Min(1),
        ])
        .split(inner_area);

    // Custom ASCII Grid construction
    let top_border = "┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐";
    let middle_border = "├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤";
    let bottom_border = "└─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘";

    let mut grid_lines = Vec::new();
    grid_lines.push(Line::from(Span::styled(
        top_border,
        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
    )));

    // Header Row with vertical bars
    let mut row1 = Vec::new();
    row1.push(Span::styled(
        "│",
        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
    ));
    for val in bit_values.iter() {
        row1.push(Span::styled(
            format!("{:^5}", val),
            Style::default().fg(base_color).add_modifier(Modifier::BOLD),
        ));
        row1.push(Span::styled(
            "│",
            Style::default().fg(base_color).add_modifier(Modifier::BOLD),
        ));
    }
    grid_lines.push(Line::from(row1));

    grid_lines.push(Line::from(Span::styled(
        middle_border,
        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
    )));

    // Bit Row
    let mut row2 = Vec::new();
    row2.push(Span::styled(
        "│",
        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
    ));
    if let Some(b_str) = binary {
        for c in b_str.chars() {
            let s = format!("{:^5}", c);
            if c == '1' {
                row2.push(Span::styled(
                    s,
                    Style::default().fg(base_color).add_modifier(Modifier::BOLD),
                ));
            } else {
                row2.push(Span::styled(s, Style::default().fg(Color::Gray)));
            }
            row2.push(Span::styled(
                "│",
                Style::default().fg(base_color).add_modifier(Modifier::BOLD),
            ));
        }
    } else {
        for _ in 0..10 {
            row2.push(Span::styled("  #  ", Style::default().fg(Color::Gray)));
            row2.push(Span::styled(
                "│",
                Style::default().fg(base_color).add_modifier(Modifier::BOLD),
            ));
        }
    }
    grid_lines.push(Line::from(row2));

    grid_lines.push(Line::from(Span::styled(
        bottom_border,
        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
    )));

    let p_ascii_grid = Paragraph::new(grid_lines).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p_ascii_grid, center_chunk[1]);

    // Word Info
    if let Some(w) = word {
        let idx = index.unwrap_or(0);
        let info = format!("Word: {} | Index: {}", w.to_uppercase(), idx);
        let p_info = Paragraph::new(info)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p_info, center_chunk[2]);
    } else {
        let info = "Select a word to view details";
        let p_info = Paragraph::new(info)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p_info, center_chunk[2]);
    }

    // Counter overlay (top right of block)
    let count_text = if app.paper_mode {
        " < Paper Mode > ".to_string()
    } else if let Some(idx) = app.saved_index {
        // Reviewing history: "4 / 5 [20]"
        format!(" Word #{}/{} [20] ", idx + 1, app.saved_words.len())
    } else {
        // Inputting: "5 / 20" -> "6 / 20"
        format!(" Word #{}/20 ", app.saved_words.len() + 1)
    };

    let count_p = Paragraph::new(count_text)
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().fg(if app.paper_mode {
            Color::Red
        } else {
            Color::Cyan
        }));
    let count_rect = Rect::new(area.x, area.y, area.width - 2, 1); // -2 for borders
    f.render_widget(count_p, count_rect);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .title(" Search ");

    let prompt = if app.input_mode == Some(InputMode::Binary) {
        if app.paper_mode {
            "Bits/> ".to_string()
        } else {
            format!("Bits #{}/> ", app.saved_words.len() + 1)
        }
    } else if app.paper_mode {
        "Word/> ".to_string()
    } else {
        format!("Word #{}/> ", app.saved_words.len() + 1)
    };

    let input_text = format!("{}{}{}", prompt, app.input, "_"); // Cursor
    let p = Paragraph::new(input_text).block(block).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(p, area);

    let help_text =
        "Esc: Exit | Enter: Select | \u{2190}\u{2192}: Suggest | \u{2191}\u{2193}: History";
    let help_p = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Right)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    // Render help text on the bottom border of the input block
    let help_rect = Rect::new(area.x + 1, area.y + 2, area.width - 2, 1);
    f.render_widget(help_p, help_rect);
}

fn render_modal(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Input Mode ")
        .style(Style::default().bg(Color::Black));

    // Center the modal
    let modal_area = centered_rect(60, 20, area);
    f.render_widget(Clear, modal_area); // Clear background
    f.render_widget(block, modal_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Padding
            Constraint::Length(3), // Buttons
            Constraint::Length(2), // Padding
        ])
        .split(modal_area);

    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(35), // Word Button
            Constraint::Percentage(10), // Gap
            Constraint::Percentage(35), // Binary Button
            Constraint::Percentage(10),
        ])
        .split(layout[1]);

    // Word Button
    let word_style = if app.modal_selection == InputMode::Word {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let word_btn = Paragraph::new("Word Input")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(word_style),
        )
        .style(word_style)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(word_btn, button_layout[1]);

    // Binary Button
    let binary_style = if app.modal_selection == InputMode::Binary {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let binary_btn = Paragraph::new("Binary Input")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(binary_style),
        )
        .style(binary_style)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(binary_btn, button_layout[3]);

    // Help text
    let help = Paragraph::new("Use \u{2190}/\u{2192} to select, Enter to confirm")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help, layout[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
