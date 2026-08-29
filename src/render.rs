use std::io::{self, Write};
use std::time::Duration;

use crate::command_display::truncate_command_display;
use crate::duration::{format_duration, format_timer_clock};
use crate::terminal::get_terminal_dimensions;
use crate::theme::{ColorSupport, Rgb, GLYPH_ERROR, GLYPH_SUCCESS, SPINNER_DOTS, SPINNER_LINE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    Dots,
    Line,
    None,
}

impl SpinnerStyle {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dots" => Some(Self::Dots),
            "line" => Some(Self::Line),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            Self::Dots => SPINNER_DOTS,
            Self::Line => SPINNER_LINE,
            Self::None => &[""],
        }
    }
}

const SPIN_PALETTE: &[Rgb] = &[
    Rgb(203, 166, 247), // lavender
    Rgb(137, 220, 235), // sky
    Rgb(148, 226, 213), // mint
    Rgb(245, 194, 231), // pink
    Rgb(249, 226, 175), // peach
];

pub struct LiveRenderer {
    spinner_style: SpinnerStyle,
    color: ColorSupport,
    display_name: String,
    frame_index: usize,
    active: bool,
}

impl LiveRenderer {
    pub fn new(spinner_style: SpinnerStyle, color: ColorSupport, display_name: String) -> Self {
        Self {
            spinner_style,
            color,
            display_name,
            frame_index: 0,
            active: false,
        }
    }

    pub fn tick(&mut self, elapsed: Duration) {
        if self.spinner_style == SpinnerStyle::None {
            return;
        }

        let frames = self.spinner_style.frames();
        let frame = frames[self.frame_index % frames.len()];
        self.frame_index += 1;

        let spin_rgb = SPIN_PALETTE[self.frame_index % SPIN_PALETTE.len()];
        let glyph = self.color.paint(spin_rgb, frame);

        let time_str = format_timer_clock(elapsed);
        let time = self.color.dim(&time_str);

        let (term_w, term_h) = get_terminal_dimensions();

        // Need at least 2 rows: 1 for content, 1 for spinner.
        if term_h < 2 {
            return;
        }

        // Layout: "  ⠙  <command>                  0:14"
        //   prefix = 5 visible chars ("  X  "), trailing = time_w
        let prefix_w = 5;
        let time_w = time_str.chars().count();
        let avail = term_w.saturating_sub(prefix_w + time_w + 2);
        let truncated = truncate_command_display(&self.display_name, avail);
        let cmd_w = truncated.chars().count();
        let cmd = self.color.bright(&truncated);
        let total = prefix_w + cmd_w + time_w;
        let pad = " ".repeat(term_w.saturating_sub(total).max(2));

        let line = format!("  {}  {}{}{}", glyph, cmd, pad, time);

        let mut stderr = io::stderr().lock();

        if !self.active {
            // Set DECSTBM scroll region to rows 1..(term_h-1).
            // Row term_h is now EXCLUDED from scrolling — it is the
            // dedicated spinner row. Child stderr scrolls only within the
            // region above, so it can never overwrite the spinner row.
            let _ = write!(stderr, "\x1b[1;{}r", term_h - 1);
            self.active = true;
        }

        // Save cursor → move to dedicated spinner row → erase → draw → restore.
        // \x1b7 / \x1b8 (VT100 save/restore) have broader terminal support
        // than \x1b[s / \x1b[u (DECSC/DECRC).
        let _ = write!(stderr, "\x1b7\x1b[{};1H\r\x1b[2K{}\x1b8", term_h, line);
        let _ = stderr.flush();
    }

    pub fn clear(&mut self) {
        if self.active {
            let (_, term_h) = get_terminal_dimensions();
            let mut stderr = io::stderr().lock();
            // Save → erase spinner row → reset scroll region to full terminal
            // (\x1b[r with no args = full window) → restore cursor.
            let _ = write!(stderr, "\x1b7\x1b[{};1H\r\x1b[2K\x1b[r\x1b8", term_h);
            let _ = stderr.flush();
            self.active = false;
        }
    }
}

// ─── final summary ─────────────────────────────────────────────────────────────
//
//  success:
//    ─────────────────────────────────────────────────────────────────
//      ✓  cargo build --release                              2m 14s
//
//  failure:
//    ─────────────────────────────────────────────────────────────────
//      ✗  cargo build --release              exit 1  ·  0:03

pub fn render_final_summary(
    display_name: &str,
    elapsed: Duration,
    exit_code: Option<i32>,
    is_success: bool,
    show_exit_code_on_success: bool,
    color: ColorSupport,
    is_tty: bool,
) {
    let (term_w, _) = get_terminal_dimensions();
    let term_width = if is_tty { term_w } else { 80 };
    let dur_str = format_duration(elapsed);
    let dot = color.dim("·");

    let summary = if is_success {
        let glyph = color.success(GLYPH_SUCCESS);
        let dur_c = color.mint(&dur_str);

        let (right_plain, right_c) = if show_exit_code_on_success {
            (
                format!("exit 0  ·  {}", dur_str),
                format!("{}  {}  {}", color.success("exit 0"), dot, dur_c),
            )
        } else {
            (dur_str.clone(), dur_c)
        };

        let right_w = right_plain.chars().count();
        let prefix_w = 5; // "  ✓  "
        let avail = term_width.saturating_sub(prefix_w + right_w + 2);
        let truncated = truncate_command_display(display_name, avail);
        let cmd_w = truncated.chars().count();
        let pad = " ".repeat(term_width.saturating_sub(prefix_w + cmd_w + right_w).max(2));
        format!(
            "  {}  {}{}{}",
            glyph,
            color.bright(&truncated),
            pad,
            right_c
        )
    } else {
        let glyph = color.error(GLYPH_ERROR);
        let dur_c = color.peach(&dur_str);

        let code_str = match exit_code {
            Some(c) => format!("exit {}", c),
            None => "signal".to_string(),
        };
        let right_plain = format!("{}  ·  {}", code_str, dur_str);
        let right_c = format!("{}  {}  {}", color.error(&code_str), dot, dur_c);

        let right_w = right_plain.chars().count();
        let prefix_w = 5;
        let avail = term_width.saturating_sub(prefix_w + right_w + 2);
        let truncated = truncate_command_display(display_name, avail);
        let cmd_w = truncated.chars().count();
        let pad = " ".repeat(term_width.saturating_sub(prefix_w + cmd_w + right_w).max(2));
        format!("  {}  {}{}{}", glyph, color.error(&truncated), pad, right_c)
    };

    let mut stderr = io::stderr().lock();

    if is_tty {
        let rule = color.dim(&"─".repeat(term_width));
        // Single write: rule + result, no blank line between them.
        let _ = write!(stderr, "{}\n{}\n", rule, summary);
    } else {
        let _ = writeln!(stderr, "{}", summary);
    }

    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_style_parse() {
        assert_eq!(SpinnerStyle::parse("dots"), Some(SpinnerStyle::Dots));
        assert_eq!(SpinnerStyle::parse("line"), Some(SpinnerStyle::Line));
        assert_eq!(SpinnerStyle::parse("none"), Some(SpinnerStyle::None));
        assert_eq!(SpinnerStyle::parse("invalid"), None);
    }
}
