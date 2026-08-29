use std::ffi::OsStr;

pub fn format_command_display<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut parts = Vec::new();
    for arg in args {
        let s = arg.as_ref().to_string_lossy();
        parts.push(quote_argument(&s));
    }
    parts.join(" ")
}

pub fn quote_argument(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }

    let needs_quotes = s.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '|' | '&'
                    | ';'
                    | '<'
                    | '>'
                    | '('
                    | ')'
                    | '$'
                    | '`'
                    | '\\'
                    | '"'
                    | '\''
                    | '*'
                    | '?'
                    | '['
                    | ']'
                    | '#'
                    | '~'
                    | '='
                    | '%'
            )
    });

    if !needs_quotes {
        return s.to_string();
    }

    // Use single quotes if no single quote is in the string
    if !s.contains('\'') {
        return format!("'{}'", s);
    }

    // Otherwise escape single quotes or use double quotes
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

pub fn truncate_command_display(s: &str, max_width: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_width {
        return s.to_string();
    }

    if max_width <= 3 {
        return "…".chars().take(max_width).collect();
    }

    // Keep prefix and suffix around an ellipsis
    let available = max_width.saturating_sub(1); // 1 char for '…'
    let prefix_len = (available * 2) / 3;
    let suffix_len = available.saturating_sub(prefix_len);

    let prefix: String = s.chars().take(prefix_len).collect();
    let suffix: String = s
        .chars()
        .skip(char_count.saturating_sub(suffix_len))
        .collect();

    format!("{}…{}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_display() {
        assert_eq!(
            format_command_display(["cargo", "test", "--release"]),
            "cargo test --release"
        );
        assert_eq!(
            format_command_display(["printf", "hello world"]),
            "printf 'hello world'"
        );
        assert_eq!(
            format_command_display(["echo", "hello 'world'"]),
            "echo 'hello '\\''world'\\'''"
        );
        assert_eq!(format_command_display([""]), "''");
        assert_eq!(
            format_command_display(["sh", "-c", "echo $HOME"]),
            "sh -c 'echo $HOME'"
        );
    }

    #[test]
    fn test_truncate_command_display() {
        let cmd = "cargo test --release --package very-long-package-name-with-lots-of-stuff";
        let truncated = truncate_command_display(cmd, 30);
        assert_eq!(truncated.chars().count(), 30);
        assert!(truncated.contains('…'));
        assert_eq!(truncate_command_display("hello", 10), "hello");
    }
}
