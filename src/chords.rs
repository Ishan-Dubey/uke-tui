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

        // Extract root & type (e.g. "C#" + "dim")
        let (root, quality) = Self::split_name(&name)?;

        // Build full alias names: [alias_root + type]
        let alias_roots = Self::alias_roots(&root);
        let alias_names = alias_roots
            .into_iter()
            .map(|r| format!("{}{}", r, quality))
            .collect();

        Some(Chord { name, frets, alias_names })
    }

    /// Load all chords from `chords.txt` (skips empty/“#” lines)
    pub fn load_from_file(path: &str) -> Vec<Self> {
        let file = File::open(path).expect("Could not open chord file.");
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
    
    /// Inspect this chord’s frets and return (min_fret, max_fret), ignoring 0/Open and X/None.
    pub fn fret_bounds(&self) -> Option<(u8, u8)> {
        let used: Vec<u8> = self
            .frets
            .iter()
            .filter_map(|&f| match f {
                Some(0) | None => None,
                Some(x) => Some(x),
            })
            .collect();
        if used.is_empty() {
            None
        } else {
            let min = *used.iter().min().unwrap();
            let max = *used.iter().max().unwrap();
            Some((min, max))
        }
    }

    /// Check of this chord matches the user’s input (case-insensitive).
    pub fn matches_name(&self, input: &str) -> bool {
        if self.name.eq_ignore_ascii_case(input) {
            true
        } else {
            self.alias_names
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(input))
        }
    }

    /// Render this chord over exactly start..=end frets (all rows use the same window).
    pub fn render_range(&self, start_fret: u8, end_fret: u8) -> String {
        let strings = ["G","C","E","A"];
        let mut out = String::new();

        // Title
        out.push_str(&format!("Chord: {}\n", self.name));

        // Header indent + fret numbers
        let prefix = "   ";          // 3 spaces
        out.push_str(prefix);
        for f in start_fret..=end_fret {
            out.push_str(&format!("{:>3}", f));
        }
        out.push('\n');

        // Divider line
        let total_width = prefix.len() + ((end_fret - start_fret + 1) as usize)*3;
        out.push_str(&"-".repeat(total_width));
        out.push('\n');

        // Each string row (A E C G)
        for &i in &[3,2,1,0] {
            let s = strings[i];
            let fv = self.frets[i];
            let ind = match fv {
                Some(0) => 'O',
                None    => 'X',
                _       => ' ',
            };
            // e.g. "G O| "
            out.push_str(&format!("{} {}| ", s, ind));

            // Cells
            for f in start_fret..=end_fret {
                if fv == Some(f) {
                    out.push_str("●  ");
                } else {
                    out.push_str("-  ");
                }
            }
            out.push('\n');
        }

        out
    }
    
// ──────────────── Helper functions ────────────────

    /// Split a chord into note + type. "C#dim" → ("C#", "dim")
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

    /// For a given root, list its equivalents
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
