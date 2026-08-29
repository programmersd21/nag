use std::process::Command;
use tempfile::tempdir;

fn nag_binary() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nag"));
    cmd.env_remove("NO_COLOR");
    cmd
}

#[test]
fn test_stdout_purity() {
    let output = nag_binary()
        .args(["printf", "hello world"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello world");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("printf 'hello world'"));
}

#[test]
fn test_stderr_preservation() {
    let output = nag_binary()
        .args(["sh", "-c", "printf 'err_out' >&2"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("err_out"));
    assert!(stderr.contains("sh -c 'printf '\\''err_out'\\'' >&2'"));
}

#[test]
fn test_exit_code_zero() {
    let output = nag_binary()
        .args(["true"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_nonzero() {
    let output = nag_binary()
        .args(["false"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exit 1"));
}

#[test]
fn test_exit_code_custom() {
    let output = nag_binary()
        .args(["sh", "-c", "exit 42"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(42));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exit 42"));
}

#[test]
fn test_quiet_flag() {
    let output = nag_binary()
        .args(["--quiet", "true"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn test_double_dash_delimiter() {
    let output = nag_binary()
        .args(["--", "echo", "--some-flag", "val"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "--some-flag val"
    );
}

#[test]
fn test_custom_title() {
    let output = nag_binary()
        .args(["--title", "custom task", "--", "true"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("custom task"));
}

#[test]
fn test_min_duration_suppression() {
    let output = nag_binary()
        .args(["--min-duration", "10s", "true"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
}

#[test]
fn test_invalid_duration_error() {
    let output = nag_binary()
        .args(["--min-duration", "invalid_dur", "true"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid duration"));
}

#[test]
fn test_config_file_loading() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[notify]
desktop = false
bell = false

[display]
live_timer = false
show_exit_code_on_success = true
"#,
    )
    .unwrap();

    let output = nag_binary()
        .args(["--config", config_path.to_str().unwrap(), "true"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exit 0"));
}

#[test]
fn test_command_not_found() {
    let output = nag_binary()
        .args(["nonexistent_command_xyz_12345"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nag: could not start `nonexistent_command_xyz_12345`"));
}

#[test]
fn test_signal_handling_and_exit_code() {
    // Spawning a subshell that kills itself with SIGTERM (15) -> shell exit 143 (128 + 15)
    let output = nag_binary()
        .args(["sh", "-c", "kill -15 $$"])
        .output()
        .expect("failed to run nag");

    assert_eq!(output.status.code(), Some(143));
}

#[test]
fn test_webhook_dispatch_mock() {
    let server = httptest::Server::run();
    server.expect(
        httptest::Expectation::matching(httptest::matchers::request::method_path(
            "POST", "/webhook",
        ))
        .respond_with(httptest::responders::status_code(200)),
    );

    let url = server.url("/webhook");

    let output = nag_binary()
        .args(["--webhook", &url.to_string(), "true"])
        .output()
        .expect("failed to run nag");

    assert!(output.status.success());
}
