use std::io::{self, IsTerminal, Write};

pub fn is_stderr_tty() -> bool {
    io::stderr().is_terminal()
}

pub fn get_terminal_dimensions() -> (usize, usize) {
    if let Some((terminal_size::Width(w), terminal_size::Height(h))) =
        terminal_size::terminal_size()
    {
        (w as usize, h as usize)
    } else {
        (80, 24)
    }
}

pub fn set_terminal_title(title: &str) {
    if !is_stderr_tty() {
        return;
    }
    // OSC 0 ; <title> BEL
    let _ = write!(io::stderr(), "\x1b]0;{}\x07", title);
    let _ = io::stderr().flush();
}

pub fn ring_bell() {
    if !is_stderr_tty() {
        return;
    }
    let _ = write!(io::stderr(), "\x07");
    let _ = io::stderr().flush();
}
