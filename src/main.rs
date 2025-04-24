mod chords;

use chords::Chord;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ukulele_chords <CHORD_NAME>");
        return;
    }

    let input = args[1].to_uppercase();

    // Hardcoded chord database (you can expand this)
    let chord_db = vec![
        Chord::from_string("C", "0 0 0 3").unwrap(),
        Chord::from_string("G", "0 2 3 2").unwrap(),
        Chord::from_string("Am", "2 0 0 0").unwrap(),
        Chord::from_string("F", "2 0 1 0").unwrap(),
    ];

    // Find and display the chord
    match chord_db.iter().find(|c| c.name.to_uppercase() == input) {
        Some(chord) => println!("{}", chord.render()),
        None => println!("Chord not found: {}", input),
    }
}
