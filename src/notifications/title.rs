use crate::terminal::set_terminal_title;

pub fn dispatch_completion_title(display_name: &str, is_success: bool) {
    let title_text = if is_success {
        format!("✓ done: {}", display_name)
    } else {
        format!("✗ failed: {}", display_name)
    };
    set_terminal_title(&title_text);
}
