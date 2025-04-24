use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chord {
    pub name: String,
    pub frets: [Option<u8>; 4], // [G, C, E, A]
}

impl Chord {
    pub fn from_string(name: &str, frets_str: &str) -> Option<Self> {
        let parsed: Vec<Option<u8>> = frets_str
            .split_whitespace()
            .map(|s| {
                if s.eq_ignore_ascii_case("X") {
                    None
                } else {
                    s.parse::<u8>().ok()
                }
            })
            .collect();

        if parsed.len() != 4 {
            return None;
        }

        Some(Self {
            name: name.to_string(),
            frets: [parsed[0], parsed[1], parsed[2], parsed[3]],
        })
    }

    pub fn render(&self) -> String {
        let strings = ["G", "C", "E", "A"];
        let mut output = String::new();

        output.push_str(&format!("Chord: {}\n", self.name));
        output.push_str("     0   1   2   3   4\n");
        output.push_str("   ---------------------\n");

        for (i, string) in (0..4).rev().enumerate() {
            let string_name = strings[3 - i];
            let fret_val = self.frets[3 - i];

            match fret_val {
                Some(0) => output.push_str(&format!("{:>2} O |", string_name)),
                None => output.push_str(&format!("{:>2} X |", string_name)),
                _ => output.push_str(&format!("{:>2}   |", string_name)),
            }

            for fret in 1..=4 {
                match fret_val {
                    Some(f) if f == fret => output.push_str(" ● "),
                    _ => output.push_str(" - "),
                }
            }
            output.push('\n');
        }

        output
    }
}