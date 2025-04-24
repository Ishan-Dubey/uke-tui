use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chord {
    /// The “official” chord name as written in chords.txt, e.g. "C#dim"
    pub name: String,
    /// Fret definitions (Some(n) = fret n, None = muted)
    pub frets: [Option<u8>; 4],
    /// Full alternate names, e.g. ["Dbdim"] for a C#dim chord
    alias_names: Vec<String>,
}

impl Chord {
    /// Parse a line like `C#dim = 0 1 0 4`
    pub fn from_string(full_name: &str, frets_str: &str) -> Option<Self> {
        let name = full_name.trim().to_string();

        // Parse exactly four tokens into Option<u8>
        let parts: Vec<&str> = frets_str.trim().split_whitespace().collect();
        if parts.len() != 4 {
            return None;
        }
        let mut frets = [None; 4];
        for (i, tok) in parts.into_iter().enumerate() {
            frets[i] = if tok.eq_ignore_ascii_case("X") {
                None
            } else {
                tok.parse::<u8>().ok()
            };
        }

        // Extract root & quality (e.g. "C#" + "dim")
        let (root, quality) = Self::split_name(&name)?;

        // Build full alias names: [alias_root + quality]
        let alias_roots = Self::alias_roots(&root);
        let alias_names = alias_roots
            .into_iter()
            .map(|r| format!("{}{}", r, quality))
            .collect();

        Some(Chord { name, frets, alias_names })
    }

    /// Load all chords from a simple `chords.txt` (skips empty/“#” lines)
    pub fn load_from_file(path: &str) -> Vec<Self> {
        let file = File::open(path).expect("Could not open chord file");
        let reader = BufReader::new(file);
        reader
            .lines()
            .filter_map(|l| l.ok())
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let (name, frets) = line.split_once('=')?;
                Self::from_string(name, frets)
            })
            .collect()
    }

    /// Does this chord match the user’s input (case-insensitive)?
    pub fn matches_name(&self, input: &str) -> bool {
        if self.name.eq_ignore_ascii_case(input) {
            true
        } else {
            self.alias_names
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(input))
        }
    }

    /// Very simple horizontal 0–4 fretboard (open = O, mute = X, note = ●)
    pub fn render(&self) -> String {
        let strings = ["G", "C", "E", "A"];
        let mut out = String::new();
        out.push_str(&format!("Chord: {}\n", self.name));
        out.push_str("     0   1   2   3   4\n");
        out.push_str("   ---------------------\n");

        // Loop from A(3) → G(0)
        for &idx in &[3, 2, 1, 0] {
            let s = strings[idx];
            let f = self.frets[idx];
            // prefix: open, muted, or blank
            if f == Some(0) {
                out.push_str(&format!("{:>2} O |", s));
            } else if f.is_none() {
                out.push_str(&format!("{:>2} X |", s));
            } else {
                out.push_str(&format!("{:>2}   |", s));
            }
            // frets 1–4
            for fret in 1..=4 {
                if f == Some(fret) {
                    out.push_str(" ● ");
                } else {
                    out.push_str(" - ");
                }
            }
            out.push('\n');
        }

        out
    }

    // ──────────────── private helpers ────────────────

    /// Split "C#dim" → ("C#", "dim")
    fn split_name(name: &str) -> Option<(String, String)> {
        // Try the 2-char roots first, then single letters
        let roots = [
            "A#", "Bb", "C#", "Db", "D#", "Eb", "F#", "Gb", "G#", "Ab",
            "A", "B", "C", "D", "E", "F", "G",
        ];
        for &r in &roots {
            if name.starts_with(r) {
                let qual = name[r.len()..].to_string();
                return Some((r.to_string(), qual));
            }
        }
        None
    }

    /// For a given root, list its enharmonic equivalents
    fn alias_roots(root: &str) -> Vec<&'static str> {
        match root {
            "C#" => vec!["Db"],
            "Db" => vec!["C#"],
            "D#" => vec!["Eb"],
            "Eb" => vec!["D#"],
            "F#" => vec!["Gb"],
            "Gb" => vec!["F#"],
            "G#" => vec!["Ab"],
            "Ab" => vec!["G#"],
            "A#" => vec!["Bb"],
            "Bb" => vec!["A#"],
            _ => vec![],
        }
    }
}
