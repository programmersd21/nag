mod cli;
mod command_display;
mod config;
mod duration;
mod error;
mod notifications;
mod process;
mod render;
mod signals;
mod terminal;
mod theme;

use clap::Parser;
use std::process::ExitCode;

use crate::cli::{print_custom_help, Cli};
use crate::command_display::format_command_display;
use crate::config::ResolvedConfig;
use crate::error::NagError;
use crate::notifications::{NotificationDispatcher, NotificationEvent};
use crate::process::execute_command;
use crate::render::render_final_summary;
use crate::terminal::is_stderr_tty;
use crate::theme::ColorSupport;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let color = ColorSupport::detect(cli.no_color);

    if cli.help {
        print_custom_help(color);
        return ExitCode::from(0);
    }

    if cli.command.is_empty() {
        print_custom_help(color);
        return ExitCode::from(1);
    }

    if let Some(dur_str) = &cli.min_duration {
        if let Err(reason) = duration::parse_duration(dur_str) {
            eprintln!(
                "{}",
                NagError::InvalidDuration {
                    input: dur_str.clone(),
                    reason,
                }
            );
            return ExitCode::from(1);
        }
    }

    let (config, config_warning) = ResolvedConfig::resolve(&cli);
    if let Some(warn) = config_warning {
        if config.verbose {
            eprintln!("nag: {}", warn);
        }
    }

    let display_name = if let Some(ref custom_title) = config.custom_title {
        custom_title.clone()
    } else {
        format_command_display(&cli.command)
    };

    let is_tty = is_stderr_tty();

    let result = match execute_command(
        &cli.command,
        &display_name,
        config.live_timer,
        config.spinner_style,
        config.notify_title,
        color,
    ) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
    };

    // Print final summary unless --quiet
    if !config.quiet {
        render_final_summary(
            &display_name,
            result.elapsed,
            result.exit_code,
            result.is_success,
            config.show_exit_code_on_success,
            color,
            is_tty,
        );
    }

    // Dispatch notifications
    let exit_code_val = result
        .exit_code
        .unwrap_or(if result.is_success { 0 } else { 1 });
    let event = NotificationEvent {
        display_name: &display_name,
        exit_code: exit_code_val,
        is_success: result.is_success,
        elapsed: result.elapsed,
    };

    let dispatcher = NotificationDispatcher::new(&config);
    dispatcher.dispatch(&event);

    ExitCode::from((exit_code_val & 0xFF) as u8)
}
