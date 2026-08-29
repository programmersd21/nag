pub mod bell;
pub mod desktop;
pub mod title;
pub mod webhook;

use crate::config::ResolvedConfig;
use std::time::Duration;

pub struct NotificationEvent<'a> {
    pub display_name: &'a str,
    pub exit_code: i32,
    pub is_success: bool,
    pub elapsed: Duration,
}

pub struct NotificationDispatcher<'a> {
    config: &'a ResolvedConfig,
}

impl<'a> NotificationDispatcher<'a> {
    pub fn new(config: &'a ResolvedConfig) -> Self {
        Self { config }
    }

    pub fn dispatch(&self, event: &NotificationEvent) {
        // Minimum duration threshold check for notifications:
        // If elapsed < min_duration:
        // - do NOT send desktop notification
        // - do NOT ring bell
        // - do NOT send webhook
        // - do NOT emit completion notification title flash
        if event.elapsed < self.config.min_duration {
            return;
        }

        // 1. Desktop Notification
        if self.config.notify_desktop {
            if let Err(e) =
                desktop::dispatch_desktop(event.display_name, event.is_success, event.elapsed)
            {
                if self.config.verbose {
                    eprintln!("nag: desktop notification error: {}", e);
                }
            }
        }

        // 2. Bell
        if self.config.notify_bell {
            bell::dispatch_bell();
        }

        // 3. Terminal Title completion state
        if self.config.notify_title {
            title::dispatch_completion_title(event.display_name, event.is_success);
        }

        // 4. Webhook
        if let Some(ref url) = self.config.webhook_url {
            if let Err(e) = webhook::dispatch_webhook(
                url,
                event.display_name,
                event.exit_code,
                event.elapsed.as_secs(),
                event.is_success,
            ) {
                if self.config.verbose {
                    eprintln!("nag: webhook dispatch error: {}", e);
                }
            }
        }
    }
}
