use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::chords::Chord;

pub struct App {
    pub input: String,
    pub chords: Vec<Chord>,
    pub output: String,
}

impl App {
    pub fn new(chords: Vec<Chord>) -> Self {
        Self {
            input: String::new(),
            chords,
            output: String::from("Type a chord and press Enter"),
        }
    }

    fn lookup(&mut self) {
        let raw = self.input.trim();
        if raw.is_empty() {
            self.output = "Please enter one or more chords, separated by commas".into();
        } else {
            let mut diagrams = Vec::new();

            for entry in raw.split(',') {
                let key = entry.trim();
                if key.is_empty() {
                    continue;
                }

                if let Some(chord) = self.chords.iter().find(|c| c.matches_name(key)) {
                    
                    // Render the stored chord, then replace header with what the user typed
                    let mut d = chord.render();
                    if let Some(pos) = d.find('\n') {
                        let rest = &d[pos..];
                        d = format!("Chord: {}\n{}", key, rest);
                    }
                    diagrams.push(d);
                } else {
                    diagrams.push(format!("Chord not found: {}", key));
                }
            }

            // Join each diagram block with a blank line
            self.output = diagrams.join("\n\n");
        }

        // Clear input for next round
        self.input.clear();
    }
}

pub fn run_tui(mut app: App) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
                .split(area);

            // Input box
            let input = Paragraph::new(app.input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Chord"));
            f.render_widget(input, chunks[0]);

            // Output area
            let output = Paragraph::new(app.output.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Diagram")
                        .border_style(Style::default().add_modifier(Modifier::BOLD)),
                );
            f.render_widget(output, chunks[1]);
        })?;

        // Event handling
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::ZERO);

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key {
                    // 1) Ctrl+C exits
                    KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                        break;
                    }
            
                    // 2) Esc exits
                    KeyEvent { code: KeyCode::Esc, .. } => {
                        break;
                    }
            
                    // 3) Normal character input
                    KeyEvent { code: KeyCode::Char(c), .. } => {
                        app.input.push(c);
                    }
            
                    // 4) Backspace
                    KeyEvent { code: KeyCode::Backspace, .. } => {
                        app.input.pop();
                    }
            
                    // 5) Enter triggers lookup
                    KeyEvent { code: KeyCode::Enter, .. } => {
                        app.lookup();
                    }
            
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
