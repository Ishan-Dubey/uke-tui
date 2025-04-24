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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::chords::Chord;

pub struct App {
    input: String,
    chords: Vec<Chord>,
    diagrams: Vec<String>,
    scroll: u16,       // scroll for diagrams
    help_shown: bool,  // whether help modal is visible
    help_scroll: u16,  // scroll for help modal
}


impl App {
    pub fn new(chords: Vec<Chord>) -> Self {
        Self {
            input: String::new(),
            chords,
            diagrams: vec!["Type comma separated chords and press Enter.".into()],
            scroll: 0,
            help_shown: false,
            help_scroll: 0,
        }
    }

    fn lookup(&mut self) {
        // same two-pass global range logic as before, but skip if help is shown
        self.help_shown = false;
        self.help_scroll = 0;
        let raw = self.input.trim();
        self.diagrams.clear();
        if raw.is_empty() {
            self.diagrams.push(
                "Please enter one or more chords, separated by commas".into()
            );
        } else {
            // collect matches / not-founds
            let mut selected = Vec::new();
            for entry in raw.split(',') {
                let key = entry.trim().to_string();
                if key.is_empty() { continue; }
                match self.chords.iter().find(|c| c.matches_name(&key)) {
                    Some(ch) => selected.push((key, ch)),
                    None     => self.diagrams.push(format!("Chord not found: {}", key)),
                }
            }
            if !selected.is_empty() {
                let mut gmin = u8::MAX;
                let mut gmax = 0u8;
                let mut has_open = false;
                for (_, chord) in &selected {
                    if chord.frets.iter().any(|&f| f == Some(0)) {
                        has_open = true;
                    }
                    if let Some((mn, mx)) = chord.fret_bounds() {
                        gmin = gmin.min(mn);
                        gmax = gmax.max(mx);
                    }
                }
                let start = if has_open || gmin < 2 { 1 } else { gmin };
                let end   = std::cmp::max(gmax, start + 4);
                for (key, chord) in selected {
                    let mut d = chord.render_range(start, end);
                    if let Some(pos) = d.find('\n') {
                        let rest = &d[pos..];
                        d = format!("Chord: {}\n{}", key, rest);
                    }
                    self.diagrams.push(d);
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
    let mut term = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        // 1) Draw
        term.draw(|f| {

            let area = f.area();

            if app.help_shown {
                // Build help text
                let lines: Vec<String> = vec![
                    "Help — Keybindings".into(),
                    "".into(),
                    "Enter   : lookup chords".into(),
                    "↑ / ↓   : scroll diagrams".into(),
                    "?       : show/hide this help".into(),
                    "Esc/C-c : quit help or exit".into(),
                    "".into(),
                    "Usage: [Note][Accidental][Type], where".into(),
                    "Note = C, D, E, F, G, A, B".into(),
                    "Accidental = None, #, b".into(),
                    "Type = None (default = maj), m, 7, maj7, m7, dim7, m7b5, 9, maj9, m9, 6, m6, add9, madd9, sus2, sus4, 7sus2, 7sus4, 7+5, 7b5, mM7, 6/9, aug, dim, add11, madd11".into(),
                    // "Supported chords:".into(),
                    "".into(),
                    "Example: C, Ebm, G#m7sus4".into(),
                ];
                // Single long line of all chord names:
                // let names = app
                //     .chords
                //     .iter()
                //     .map(|c| c.name.clone())
                //     .collect::<Vec<_>>()
                //     .join(", ");
                // lines.push(names);

                let help_text = lines.join("\n");

                // Centered help box
                let block_area = {
                    let w = area.width.saturating_sub(10);
                    let h = area.height.saturating_sub(6);
                    let x = area.x + (area.width - w) / 2;
                    let y = area.y + (area.height - h) / 2;
                    Rect::new(x, y, w, h)
                };

                // Render clear background + help
                f.render_widget(Clear, block_area);
                let help_para = Paragraph::new(help_text)
                    .wrap(Wrap { trim: true })
                    .scroll((app.help_scroll, 0))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Help ")
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                    .alignment(Alignment::Left);
                f.render_widget(help_para, block_area);
            } else {
                // Main UI split
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // Input box
                let input = Paragraph::new(app.input.as_str())
                    .block(Block::default().borders(Borders::ALL).title("Chord(s)"));
                f.render_widget(input, chunks[0]);

                // Blinking cursor at end of input
                let x = chunks[0].x + 1 + UnicodeWidthStr::width(app.input.as_str()) as u16;
                let y = chunks[0].y + 1;
                f.set_cursor_position((x, y));

                // Diagrams grid
                // let max_w = chunks[1].width as usize;
                // let rows = combine_diagrams_grid(&app.diagrams, max_w, 2);
                // let text = rows.join("\n");
                // let diags = Paragraph::new(text)
                //     .scroll((app.scroll, 0))
                //     .block(
                //         Block::default()
                //             .borders(Borders::ALL)
                //             .title("Diagrams")
                //             .border_style(Style::default().add_modifier(Modifier::BOLD)),
                //     );
                // f.render_widget(diags, chunks[1]);
                let area = chunks[1];
                let text_block = if app.diagrams.len() == 1
                    && app.diagrams[0].starts_with("Type comma separated")
                {
                    // INITIAL LOGO + PROMPT
                    let box_width = area.width as usize;
                
                    // 1. ASCII art
                    let lines = vec![
                        "     @@@@@@@@                                                        ".to_string(),
                        "   @@@      @@@@         @@       uke-tui                            ".to_string(),
                        "  @@@          @@@@@  @@@@@@@@    ishan                              ".to_string(),
                        " @@               @@@@@   @@@@@   https://github.com/ishan-dubey     ".to_string(),
                        " @@          ##              @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@        ".to_string(),
                        "@@           #────,**,────────────────────────────────X   X@@@       ".to_string(),
                        "@@           #───(,,,,)──────────────────────────────────/   @@@     ".to_string(),
                        " @@          #──((,,,,))─────────────────────────────────\\     @@@   ".to_string(),
                        " @@          #───(,,,,)───────────────────────────────X   X      @@@ ".to_string(),
                        "  @@         ##   ****        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@".to_string(),
                        "   @@            @@@@       @@@                                      ".to_string(),
                        "   @@@@@    @@@@@@  @@@@@@@@@                                        ".to_string(),
                        "      @@@@@@@@                                                       ".to_string(),
                        // Prompt
                        "".to_string(),
                        "Type a chord and press Enter".to_string(),
                        "".to_string(),
                    ];

                
                    // 3. Center each line horizontally
                    lines
                        .into_iter()
                        .map(|line| {
                            let w = UnicodeWidthStr::width(line.as_str());
                            if box_width > w {
                                let left = (box_width - w) / 2;
                                " ".repeat(left) + &line
                            } else {
                                line
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    // NORMAL GRID
                    let max_w = area.width as usize;
                    let rows = combine_diagrams_grid(&app.diagrams, max_w, 2);
                    rows.join("\n")
                };
                
                // Render it
                let diags = Paragraph::new(text_block)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Diagrams")
                            .border_style(Style::default().add_modifier(Modifier::BOLD)),
                    );
                f.render_widget(diags, area);

                // Footer
                let footer = Paragraph::new("Enter:lookup  ↑/↓:scroll  ?:help  Esc/C-c:quit")
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                f.render_widget(footer, chunks[2]);
            }
        })?;

        // 2) Show or hide terminal cursor
        if app.help_shown {
            term.hide_cursor()?;
        } else {
            term.show_cursor()?;
        }

        // 3) Input / scrolling events
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.help_shown {
                    match key {
                        KeyEvent { code: KeyCode::Esc, .. }
                        | KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } =>
                        {
                            app.help_shown = false;
                        }
                        KeyEvent { code: KeyCode::Up, .. } => {
                            if app.help_scroll > 0 {
                                app.help_scroll -= 1;
                            }
                        }
                        KeyEvent { code: KeyCode::Down, .. } => {
                            app.help_scroll = app.help_scroll.saturating_add(1);
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        KeyEvent { code: KeyCode::Char('?'), .. } => {
                            app.help_shown = true;
                            app.help_scroll = 0;
                        }
                        KeyEvent { code: KeyCode::Esc, .. }
                        | KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } =>
                        {
                            break;
                        }
                        KeyEvent { code: KeyCode::Char(c), .. } => {
                            app.input.push(c);
                        }
                        KeyEvent { code: KeyCode::Backspace, .. } => {
                            app.input.pop();
                        }
                        KeyEvent { code: KeyCode::Enter, .. } => {
                            app.lookup();
                        }
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
        }

        // 4) Throttle loop
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }


    // restore
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
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
