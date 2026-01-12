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
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use slip39_calculator::{encode, wordlist};
use std::{error::Error, io};

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
}

impl App {
    pub fn new(paper_mode: bool) -> App {
        App {
            input: String::new(),
            suggestions: Vec::new(),
            suggestion_index: 0,
            saved_words: Vec::new(),
            saved_index: None,
            all_words: wordlist().iter().map(|s| s.to_string()).collect(),
            paper_mode,
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
        if let Some(word) = self.suggestions.get(self.suggestion_index) {
            if self.paper_mode {
                self.saved_words.clear();
                self.saved_words.push(word.clone());
                self.saved_index = Some(0);
            } else if self.saved_words.len() < 20 {
                self.saved_words.push(word.clone());
                self.saved_index = Some(self.saved_words.len() - 1);
            }
            // Clear input after adding
            self.input.clear();
            self.update_suggestions();
        }
    }
}

pub fn run(paper_mode: bool) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(paper_mode);
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
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char('q') if app.input.is_empty() => return Ok(()),

                    // Input handling
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.update_suggestions();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.update_suggestions();
                    }

                    // Carousel Navigation
                    KeyCode::Left => {
                        if !app.suggestions.is_empty() {
                            if app.suggestion_index > 0 {
                                app.suggestion_index -= 1;
                            } else {
                                app.suggestion_index = app.suggestions.len() - 1;
                                // Wrap around
                            }
                        }
                    }
                    KeyCode::Right => {
                        if !app.suggestions.is_empty() {
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

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Suggestions Carousel
            Constraint::Min(10),   // Main Grid
            Constraint::Length(3), // Input & Help
        ])
        .split(f.area());

    render_carousel(f, app, chunks[0]);
    render_grid(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
}

fn render_carousel(f: &mut Frame, app: &App, area: Rect) {
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
        let w = &app.saved_words[idx];
        let i = app.all_words.iter().position(|x| x == w).unwrap_or(0);
        let b = encode(w).unwrap_or_else(|_| "0000000000".to_string());
        (Some(w.clone()), Some(i), Some(b))
    } else {
        (None, None, None)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
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
        Style::default().fg(Color::Yellow),
    )));

    // Header Row with vertical bars
    let mut row1 = Vec::new();
    row1.push(Span::styled("│", Style::default().fg(Color::Yellow)));
    for val in bit_values.iter() {
        row1.push(Span::styled(
            format!("{:^5}", val),
            Style::default().fg(Color::Yellow),
        ));
        row1.push(Span::styled("│", Style::default().fg(Color::Yellow)));
    }
    grid_lines.push(Line::from(row1));

    grid_lines.push(Line::from(Span::styled(
        middle_border,
        Style::default().fg(Color::Yellow),
    )));

    // Bit Row
    let mut row2 = Vec::new();
    row2.push(Span::styled("│", Style::default().fg(Color::Yellow)));
    if let Some(b_str) = binary {
        for c in b_str.chars() {
            let s = format!("{:^5}", c);
            if c == '1' {
                row2.push(Span::styled(
                    s,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                row2.push(Span::styled(s, Style::default().fg(Color::Gray)));
            }
            row2.push(Span::styled("│", Style::default().fg(Color::Yellow)));
        }
    } else {
        for _ in 0..10 {
            row2.push(Span::styled("  ?  ", Style::default().fg(Color::Gray)));
            row2.push(Span::styled("│", Style::default().fg(Color::Yellow)));
        }
    }
    grid_lines.push(Line::from(row2));

    grid_lines.push(Line::from(Span::styled(
        bottom_border,
        Style::default().fg(Color::Yellow),
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
        // Inputting: "5 / 20"
        format!(" Word #{}/20 ", app.saved_words.len())
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
    let block = Block::default().borders(Borders::ALL).title(" Search ");

    let prompt = if app.paper_mode {
        "Word > ".to_string()
    } else {
        format!("Word #{} > ", app.saved_words.len() + 1)
    };

    let input_text = format!("{}{}{}", prompt, app.input, "_"); // Cursor
    let p = Paragraph::new(input_text)
        .block(block)
        .style(Style::default().fg(Color::White));
    f.render_widget(p, area);

    let help_text =
        "Esc: Exit | Enter: Select | \u{2190}\u{2192}: Suggest | \u{2191}\u{2193}: History";
    let help_p = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().fg(Color::DarkGray));
    let help_rect = Rect::new(area.x, area.y + 2, area.width - 2, 1); // Bottom line of block
                                                                      // We can render checking constraints, but for now just comment or leave unused if logic isn't perfect
                                                                      // f.render_widget(help_p, help_rect);
                                                                      // To silence warning, use help_rect
    let _ = help_rect;
    let _ = help_p;
    // Or actually render it if it fits?
    // Let's rely on main input block having enough height (3 lines), so help fits in 3rd line.
    // input takes 2nd line.
    // Border takes 1st and 3rd.
    // So we can't really put text below without overwriting border or extending.
    // Let's just suppress warning.
}
