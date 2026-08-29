use std::env;
use std::io::{self, IsTerminal};

// vibrant ultra-modern truecolor catppuccin-inspired palette
// accent / lavender:  #cba6f7  rgb(203, 166, 247)
// sky / cyan:         #89dceb  rgb(137, 220, 235)
// mint / teal:        #94e2d5  rgb(148, 226, 213)
// peach / yellow:     #f9e2af  rgb(249, 226, 175)
// pink / flamingo:    #f5c2e7  rgb(245, 194, 231)
// green / sage:       #a6e3a1  rgb(166, 227, 161)
// red / rose:         #f38ba8  rgb(243, 139, 168)
// text / bright:      #cdd6f4  rgb(205, 214, 244)
// muted / subtext1:   #a6adc8  rgb(166, 173, 200)
// dim / overlay1:     #6c7086  rgb(108, 112, 134)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub const COLOR_DIM: Rgb = Rgb(108, 112, 134);
pub const COLOR_MUTED: Rgb = Rgb(166, 173, 200);
pub const COLOR_BRIGHT: Rgb = Rgb(205, 214, 244);
pub const COLOR_SUCCESS: Rgb = Rgb(166, 227, 161);
pub const COLOR_ERROR: Rgb = Rgb(243, 139, 168);
pub const COLOR_ACCENT: Rgb = Rgb(203, 166, 247);
pub const COLOR_SKY: Rgb = Rgb(137, 220, 235);
pub const COLOR_MINT: Rgb = Rgb(148, 226, 213);
pub const COLOR_PEACH: Rgb = Rgb(249, 226, 175);
pub const COLOR_PINK: Rgb = Rgb(245, 194, 231);

pub const GLYPH_SUCCESS: &str = "✓";
pub const GLYPH_ERROR: &str = "✗";

pub const SPINNER_DOTS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
pub const SPINNER_LINE: &[&str] = &["-", "\\", "|", "/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorSupport {
    pub enabled: bool,
}

impl ColorSupport {
    pub fn detect(cli_no_color: bool) -> Self {
        if cli_no_color {
            return Self { enabled: false };
        }
        if env::var_os("NO_COLOR").is_some() {
            return Self { enabled: false };
        }
        if env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
            return Self { enabled: false };
        }
        // color is valid on either a tty stdout or stderr (e.g. --help piped check)
        let is_term = io::stderr().is_terminal() || io::stdout().is_terminal();
        if !is_term {
            return Self { enabled: false };
        }
        let term = env::var("TERM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        let supported = !term.is_empty()
            || !colorterm.is_empty()
            || env::var_os("ITERM_SESSION_ID").is_some()
            || env::var_os("VSCODE_INJECTION").is_some()
            || env::var_os("ALACRITTY_LOG").is_some()
            || env::var_os("KITTY_PID").is_some();
        Self { enabled: supported }
    }

    pub fn paint(&self, rgb: Rgb, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        format!("\x1b[38;2;{};{};{}m{}\x1b[0m", rgb.0, rgb.1, rgb.2, text)
    }

    pub fn bold(&self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        format!("\x1b[1m{}\x1b[0m", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.paint(COLOR_DIM, text)
    }

    pub fn muted(&self, text: &str) -> String {
        self.paint(COLOR_MUTED, text)
    }

    pub fn bright(&self, text: &str) -> String {
        self.paint(COLOR_BRIGHT, text)
    }

    pub fn success(&self, text: &str) -> String {
        self.paint(COLOR_SUCCESS, text)
    }

    pub fn error(&self, text: &str) -> String {
        self.paint(COLOR_ERROR, text)
    }

    pub fn accent(&self, text: &str) -> String {
        self.paint(COLOR_ACCENT, text)
    }

    pub fn sky(&self, text: &str) -> String {
        self.paint(COLOR_SKY, text)
    }

    pub fn mint(&self, text: &str) -> String {
        self.paint(COLOR_MINT, text)
    }

    pub fn peach(&self, text: &str) -> String {
        self.paint(COLOR_PEACH, text)
    }

    pub fn pink(&self, text: &str) -> String {
        self.paint(COLOR_PINK, text)
    }
}
