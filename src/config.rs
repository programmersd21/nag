use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cli::Cli;
use crate::duration::parse_duration;
use crate::render::SpinnerStyle;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ConfigFile {
    pub notify: Option<NotifyConfig>,
    pub display: Option<DisplayConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NotifyConfig {
    pub desktop: Option<bool>,
    pub bell: Option<bool>,
    pub title: Option<bool>,
    pub webhook_url: Option<String>,
    pub min_duration_secs: Option<u64>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DisplayConfig {
    pub live_timer: Option<bool>,
    pub spinner_style: Option<String>,
    pub show_exit_code_on_success: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub notify_desktop: bool,
    pub notify_bell: bool,
    pub notify_title: bool,
    pub webhook_url: Option<String>,
    pub min_duration: Duration,
    pub live_timer: bool,
    pub spinner_style: SpinnerStyle,
    pub show_exit_code_on_success: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub custom_title: Option<String>,
}

impl ResolvedConfig {
    pub fn resolve(cli: &Cli) -> (Self, Option<String>) {
        let mut warning = None;

        let config_file = if let Some(custom_path) = &cli.config {
            match load_config_from_path(custom_path) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    warning = Some(format!(
                        "failed to load config at {}: {}",
                        custom_path.display(),
                        e
                    ));
                    None
                }
            }
        } else if let Some(default_path) = default_config_path() {
            if default_path.exists() {
                match load_config_from_path(&default_path) {
                    Ok(cfg) => Some(cfg),
                    Err(e) => {
                        warning = Some(format!(
                            "failed to load config at {}: {}",
                            default_path.display(),
                            e
                        ));
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let file_notify = config_file
            .as_ref()
            .and_then(|c| c.notify.clone())
            .unwrap_or_default();
        let file_display = config_file
            .as_ref()
            .and_then(|c| c.display.clone())
            .unwrap_or_default();

        let notify_desktop = if cli.no_desktop {
            false
        } else {
            file_notify.desktop.unwrap_or(true)
        };

        let notify_bell = if cli.no_bell {
            false
        } else {
            file_notify.bell.unwrap_or(true)
        };

        let notify_title = if cli.no_title {
            false
        } else {
            file_notify.title.unwrap_or(true)
        };

        let webhook_url = if cli.no_webhook {
            None
        } else if let Some(url) = &cli.webhook {
            if url.trim().is_empty() {
                None
            } else {
                Some(url.clone())
            }
        } else if let Some(url) = &file_notify.webhook_url {
            if url.trim().is_empty() {
                None
            } else {
                Some(url.clone())
            }
        } else if let Ok(url) = env::var("NAG_WEBHOOK_URL") {
            if url.trim().is_empty() {
                None
            } else {
                Some(url)
            }
        } else {
            None
        };

        let min_duration = if let Some(dur_str) = &cli.min_duration {
            match parse_duration(dur_str) {
                Ok(d) => d,
                Err(_) => Duration::ZERO,
            }
        } else if let Some(secs) = file_notify.min_duration_secs {
            Duration::from_secs(secs)
        } else {
            // default: notify for every command regardless of duration
            Duration::ZERO
        };

        let live_timer = file_display.live_timer.unwrap_or(true);
        let spinner_style = if !live_timer {
            SpinnerStyle::None
        } else if let Some(style_str) = &file_display.spinner_style {
            SpinnerStyle::parse(style_str).unwrap_or(SpinnerStyle::Dots)
        } else {
            SpinnerStyle::Dots
        };

        let show_exit_code_on_success = file_display.show_exit_code_on_success.unwrap_or(false);

        let resolved = Self {
            notify_desktop,
            notify_bell,
            notify_title,
            webhook_url,
            min_duration,
            live_timer,
            spinner_style,
            show_exit_code_on_success,
            quiet: cli.quiet,
            verbose: cli.verbose,
            custom_title: cli.title.clone(),
        };

        (resolved, warning)
    }
}

fn load_config_from_path(path: &Path) -> Result<ConfigFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed: ConfigFile = toml::from_str(&content)?;
    Ok(parsed)
}

fn default_config_path() -> Option<PathBuf> {
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        if !xdg_config.trim().is_empty() {
            return Some(PathBuf::from(xdg_config).join("nag/config.toml"));
        }
    }
    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home).join(".config/nag/config.toml"));
        }
    }
    None
}
