mod chords;
use chords::Chord;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ukulele_chords <CHORD_NAME>");
        std::process::exit(1);
    }

    // Join all args apart from the binary name, e.g. ["Db", "dim"] → "Dbdim"
    let input = args[1..].join("");

    // Load your chords.txt
    let chord_db = Chord::load_from_file("chords.txt");

    // Find a match by name or alias
    if let Some(chord) = chord_db.iter().find(|c| c.matches_name(&input)) {
        println!("{}", chord.render());
    } else {
        println!("Chord not found: {}", input);
    }
}
