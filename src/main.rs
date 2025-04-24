mod chords;
mod tui;

use chords::Chord;
use std::process;

fn main() {
    // Load all chords from file
    let chords = Chord::load_from_file("chords.txt");

    // Launch TUI app
    let app = tui::App::new(chords);
    if let Err(e) = tui::run_tui(app) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
