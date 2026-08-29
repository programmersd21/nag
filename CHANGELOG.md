# changelog

all notable changes to this project will be documented in this file.

format: [keep a changelog](https://keepachangelog.com/en/1.1.0/) —
versioning: [semantic versioning](https://semver.org/spec/v2.0.0.html)

## [0.1.0] — 2026-08-29

### added

- transparent command execution — zero stdout alteration, full stdio inheritance, no buffering
- live in-place spinner with palette-cycling animated glyph (dots / line / none)
- right-aligned elapsed timer updating in place during execution
- final summary line with thin rule separator, always anchored after command output
- desktop notifications via dbus/zbus on linux and nsuser on macos
- svg notification icons compiled into the binary — no runtime asset search
- terminal bell on completion
- osc 0 terminal title updates (live clock while running, completion state on finish)
- slack and discord webhook auto-detection with structured payloads
- generic json webhook support
- `--min-duration` threshold — suppress notifications for commands faster than the given duration (default: 0, always notify)
- exact unix exit code fidelity — exit codes 0–255 are forwarded unchanged
- process group signal forwarding (sigint, sigterm, sighup, sigquit reach all child processes)
- optional config at `~/.config/nag/config.toml` with precedence: cli > env > config > defaults
- `NAG_WEBHOOK_URL` environment variable support
- automatic monochrome fallback when stderr is not a tty
- 14 integration tests covering exit codes, signal handling, stdout purity, webhooks, and config loading
