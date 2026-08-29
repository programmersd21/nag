use crate::duration::format_duration;
use std::io::Write;
use std::time::Duration;

// SVG icons compiled into the binary — zero runtime asset search.
const ICON_SUCCESS: &[u8] = include_bytes!("../../assets/icons/nag-success.svg");
const ICON_ERROR: &[u8] = include_bytes!("../../assets/icons/nag-error.svg");

/// Write a bundled icon to a temp file and return its path.
/// Uses std only — no tempfile crate dependency in production code.
fn write_icon_temp(data: &[u8], name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::temp_dir().join(name);
    let mut f = std::fs::File::create(&path).ok()?;
    f.write_all(data).ok()?;
    Some(path)
}

pub fn dispatch_desktop(
    display_name: &str,
    is_success: bool,
    elapsed: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (summary, icon_data, icon_name) = if is_success {
        ("done", ICON_SUCCESS, "nag-success.svg")
    } else {
        ("failed", ICON_ERROR, "nag-error.svg")
    };

    let body = format!("{}\n{}", display_name, format_duration(elapsed));

    let mut notification = notify_rust::Notification::new();
    notification.summary(summary).body(&body).appname("nag");

    // Write the bundled icon to /tmp and point notify-rust at it.
    // If writing fails we still show the notification — just without an icon.
    if let Some(icon_path) = write_icon_temp(icon_data, icon_name) {
        notification.icon(&icon_path.to_string_lossy());
    }

    notification.show()?;
    Ok(())
}
