use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use unicode_width::UnicodeWidthStr;

use crate::chords::Chord;

pub struct App {
    input: String,
    chords: Vec<Chord>,
    diagrams: Vec<String>,
    scroll: u16, // vertical scroll
}

impl App {
    pub fn new(chords: Vec<Chord>) -> Self {
        Self {
            input: String::new(),
            chords,
            diagrams: vec!["Type a chord and press Enter".into()],
            scroll: 0,
        }
    }

    fn lookup(&mut self) {
        let raw = self.input.trim();
        self.diagrams.clear();
        if raw.is_empty() {
            self.diagrams.push("Please enter one or more chords, separated by commas".into());
        } else {
            for entry in raw.split(',') {
                let key = entry.trim();
                if key.is_empty() {
                    continue;
                }
                if let Some(chord) = self.chords.iter().find(|c| c.matches_name(key)) {
                    let mut d = chord.render();
                    if let Some(pos) = d.find('\n') {
                        let rest = &d[pos..];
                        d = format!("Chord: {}\n{}", key, rest);
                    }
                    self.diagrams.push(d);
                } else {
                    self.diagrams.push(format!("Chord not found: {}", key));
                }
            }
        }
        self.input.clear();
        self.scroll = 0;
    }
}

pub fn run_tui(mut app: App) -> io::Result<()> {
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

            // split into 3 vertical chunks: input, diagram grid, footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // input box
                    Constraint::Min(5),     // diagrams area
                    Constraint::Length(1),  // footer/info bar
                ])
                .split(area);

            // 1) Input box
            let input = Paragraph::new(app.input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Chord(s)"));
            f.render_widget(input, chunks[0]);

            // 2) Diagrams (unchanged)
            let max_width = chunks[1].width as usize;
            let rows = combine_diagrams_grid(&app.diagrams, max_width, 2);
            // clamp app.scroll …
            let text = rows.join("\n");
            let diagrams = Paragraph::new(text)
                .scroll((app.scroll, 0))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Diagrams")
                        .border_style(Style::default().add_modifier(Modifier::BOLD)),
                );
            f.render_widget(diagrams, chunks[1]);

            // 3) Footer / info bar
            let footer_text = "Enter: lookup  |  ↑/↓: scroll  |  Esc/Ctrl-C: quit";
            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input & scrolling
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key {
                    // Exit on Esc or Ctrl+C
                    KeyEvent { code: KeyCode::Esc, .. }
                    | KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } =>
                    {
                        break
                    }
                    // Normal typing
                    KeyEvent { code: KeyCode::Char(c), .. } => {
                        app.input.push(c);
                    }
                    KeyEvent { code: KeyCode::Backspace, .. } => {
                        app.input.pop();
                    }
                    // Enter = lookup
                    KeyEvent { code: KeyCode::Enter, .. } => {
                        app.lookup();
                    }
                    // Scroll up/down
                    KeyEvent { code: KeyCode::Up, .. } => {
                        if app.scroll > 0 {
                            app.scroll -= 1;
                        }
                    }
                    KeyEvent { code: KeyCode::Down, .. } => {
                        app.scroll = app.scroll.saturating_add(1);
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

/// Arrange diagrams into rows that wrap at `max_width`, spacing them by `spacing` columns,
/// and padding each line to the display‐width of its block.
fn combine_diagrams_grid(
    diagrams: &[String],
    max_width: usize,
    spacing: usize,
) -> Vec<String> {
    // 1) Split into lines and compute each block’s display‐width & height
    let blocks: Vec<Vec<String>> = diagrams
        .iter()
        .map(|d| d.lines().map(str::to_string).collect())
        .collect();

    let widths: Vec<usize> = blocks
        .iter()
        .map(|lines| {
            lines
                .iter()
                .map(|l| UnicodeWidthStr::width(l.as_str()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    let heights: Vec<usize> = blocks.iter().map(|lines| lines.len()).collect();

    // 2) Pack block indices into rows
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut used_w = 0;

    for (i, &w) in widths.iter().enumerate() {
        let needed = if cur.is_empty() { w } else { used_w + spacing + w };
        if needed > max_width && !cur.is_empty() {
            rows.push(cur);
            cur = vec![i];
            used_w = w;
        } else {
            if cur.is_empty() {
                used_w = w;
            } else {
                used_w = used_w + spacing + w;
            }
            cur.push(i);
        }
    }
    if !cur.is_empty() {
        rows.push(cur);
    }

    // 3) Build each output line
    let mut out: Vec<String> = Vec::new();

    for row in rows {
        // how tall is this row?
        let row_h = row.iter().map(|&i| heights[i]).max().unwrap_or(0);

        for line_idx in 0..row_h {
            let mut line = String::new();

            for (j, &block_i) in row.iter().enumerate() {
                let block = &blocks[block_i];
                // get the text for this line, or "" if the block has fewer lines
                let cell = block.get(line_idx).map(String::as_str).unwrap_or("");
                let disp = UnicodeWidthStr::width(cell);
                let pad = widths[block_i].saturating_sub(disp);

                line.push_str(cell);
                // pad with spaces to the block’s full width
                line.push_str(&" ".repeat(pad));

                // spacing between blocks
                if j + 1 < row.len() {
                    line.push_str(&" ".repeat(spacing));
                }
            }

            out.push(line);
        }

        // blank separator row between block‐rows
        out.push(String::new());
    }

    out
}
