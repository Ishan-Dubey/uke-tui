# uke-tui

A terminal-based ukulele chord viewer written in Rust, powered by [Ratatui](https://crates.io/crates/ratatui).  
Type a chord (or comma-separated list of chords) to see clean ASCII fretboard diagrams, complete with dynamic fret ranges, a built-in help overlay, and a little ukulele ASCII art splash screen.

---

## 🎸 Features

- **Single-chord lookup**: `C`, `Am7`, `F#dim`, etc.  
- **Multi-chord mode**: `C, Am, F, G` → displays all diagrams in a wrapped grid.  
- **Dynamic fret range**: Auto-zoom to the lowest/highest fret used (with a minimum 5-fret window).  
- **Muted/open strings**: `X` for muted, `O` for open.  
- **Help overlay**: `?` to list keybindings and usage guide.  
- **Cross-platform**: works on Linux, macOS, Windows in any ANSI terminal.

---

## 📺 Demo

<!-- Replace with an actual recording or screenshots! -->
![Demo GIF placeholder](docs/demo.gif)

---

## 🚀 Installation

1. Clone this repo:
    ```bash
    git clone https://github.com/your-username/uke-tui.git
    cd uke-tui
    ```

2. Ensure you have Rust and Cargo installed.

2. Build:
    ```bash
    cargo build --release
    ```