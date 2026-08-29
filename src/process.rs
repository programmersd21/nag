use std::ffi::OsString;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::NagError;
use crate::render::{LiveRenderer, SpinnerStyle};
use crate::signals::{map_signal_to_exit_code, restore_signal_handlers, setup_signal_handlers};
use crate::terminal::{is_stderr_tty, set_terminal_title};
use crate::theme::ColorSupport;

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub is_success: bool,
    pub elapsed: Duration,
}

pub fn execute_command(
    command_args: &[OsString],
    display_name: &str,
    live_timer_enabled: bool,
    spinner_style: SpinnerStyle,
    title_updates_enabled: bool,
    color: ColorSupport,
) -> Result<ProcessResult, NagError> {
    if command_args.is_empty() {
        return Err(NagError::NoCommandSpecified);
    }

    let program = &command_args[0];
    let args = &command_args[1..];

    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    // Create a new process group for the child so signal forwarding reaches descendants
    cmd.process_group(0);

    let start_instant = Instant::now();

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let cmd_str = program.to_string_lossy().to_string();
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(NagError::CommandNotFound {
                    command: cmd_str,
                    source: e,
                });
            } else {
                return Err(NagError::CommandSpawnFailed {
                    command: cmd_str,
                    source: e,
                });
            }
        }
    };

    let child_pid = child.id() as i32;
    setup_signal_handlers(child_pid);

    let is_tty = is_stderr_tty();
    let should_render_live = is_tty && live_timer_enabled && spinner_style != SpinnerStyle::None;
    let should_update_title = is_tty && title_updates_enabled;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    let display_name_owned = display_name.to_string();

    let ui_thread = if should_render_live || should_update_title {
        Some(thread::spawn(move || {
            let mut renderer = LiveRenderer::new(spinner_style, color, display_name_owned.clone());
            let mut last_title_sec: u64 = u64::MAX;

            while !stop_flag_clone.load(Ordering::Relaxed) {
                let elapsed = start_instant.elapsed();
                let secs = elapsed.as_secs();

                if should_render_live {
                    renderer.tick(elapsed);
                }

                if should_update_title && secs != last_title_sec {
                    last_title_sec = secs;
                    let clock_str = crate::duration::format_timer_clock(elapsed);
                    let title_text = format!("⏱ {} {}", clock_str, display_name_owned);
                    set_terminal_title(&title_text);
                }

                thread::sleep(Duration::from_millis(100));
            }

            if should_render_live {
                renderer.clear();
            }
        }))
    } else {
        None
    };

    // Wait for the child process to finish
    let status_res = child.wait();
    let end_instant = Instant::now();
    let elapsed = end_instant.duration_since(start_instant);

    stop_flag.store(true, Ordering::Relaxed);
    if let Some(t) = ui_thread {
        let _ = t.join();
    }

    restore_signal_handlers();

    let status = status_res.map_err(|e| NagError::CommandSpawnFailed {
        command: program.to_string_lossy().to_string(),
        source: e,
    })?;

    let exit_code = if let Some(code) = status.code() {
        Some(code)
    } else if let Some(sig) = status.signal() {
        Some(map_signal_to_exit_code(sig))
    } else {
        Some(1)
    };

    let is_success = status.success();

    Ok(ProcessResult {
        exit_code,
        is_success,
        elapsed,
    })
}
