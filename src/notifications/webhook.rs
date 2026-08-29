use serde::Serialize;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct GenericWebhookPayload {
    pub command: String,
    pub exit_code: i32,
    pub duration_secs: u64,
    pub success: bool,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlackPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    pub color: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscordPayload {
    pub embeds: Vec<DiscordEmbed>,
}

pub fn get_hostname() -> String {
    if let Ok(host) = env::var("HOSTNAME") {
        if !host.trim().is_empty() {
            return host;
        }
    }
    if let Ok(host) = env::var("HOST") {
        if !host.trim().is_empty() {
            return host;
        }
    }
    // Try reading /etc/hostname on Unix
    if let Ok(content) = std::fs::read_to_string("/etc/hostname") {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "unknown".to_string()
}

pub fn dispatch_webhook(
    url: &str,
    display_name: &str,
    exit_code: i32,
    duration_secs: u64,
    success: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hostname = get_hostname();

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let url_lower = url.to_ascii_lowercase();

    let response = if url_lower.contains("hooks.slack.com") {
        let status_emoji = if success { "✓" } else { "✗" };
        let status_text = if success { "succeeded" } else { "failed" };
        let text = format!(
            "{} *{}* {} on `{}` (exit {}, duration: {}s)",
            status_emoji, display_name, status_text, hostname, exit_code, duration_secs
        );
        let payload = SlackPayload { text };
        client.post(url).json(&payload).send()?
    } else if url_lower.contains("discord.com/api/webhooks")
        || url_lower.contains("discordapp.com/api/webhooks")
    {
        let title = if success {
            format!("✓ Done: {}", display_name)
        } else {
            format!("✗ Failed: {}", display_name)
        };
        let desc = format!(
            "Host: `{}`\nExit code: `{}`\nDuration: `{}s`",
            hostname, exit_code, duration_secs
        );
        let color = if success { 0x9BC9A8 } else { 0xE3949E };
        let payload = DiscordPayload {
            embeds: vec![DiscordEmbed {
                title,
                description: desc,
                color,
            }],
        };
        client.post(url).json(&payload).send()?
    } else {
        let payload = GenericWebhookPayload {
            command: display_name.to_string(),
            exit_code,
            duration_secs,
            success,
            hostname,
        };
        client.post(url).json(&payload).send()?
    };

    if !response.status().is_success() {
        return Err(format!("webhook responded with status {}", response.status()).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hostname() {
        let host = get_hostname();
        assert!(!host.is_empty());
    }
}
