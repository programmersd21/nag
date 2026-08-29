# contributing

thanks for considering a contribution to nag.

## before you open a pr

- open an issue first for anything non-trivial — feature requests, design changes, or behaviour shifts
- bug fixes and documentation corrections can go straight to a pr

## development setup

```
git clone https://github.com/programmersd21/nag
cd nag
cargo build
cargo test --all-targets --all-features
```

requires rust stable (≥ 1.80). no other system dependencies needed for the build itself; desktop notifications on linux require a running dbus session.

## making changes

- `cargo fmt --all` before committing — the ci enforces formatting
- `cargo clippy --all-targets --all-features -- -D warnings` must pass clean
- add or update tests in `tests/integration_tests.rs` for any behaviour change
- keep commit messages lowercase and concise

## submitting

- target `main`
- one logical change per pr
- if the pr fixes a bug, reference the issue number

## code style

- lowercase comments and strings throughout the codebase
- no windows-specific code — this tool is unix-only by design
- zero stdout interference is a hard invariant — nag must never write to stdout during command execution

## license

by contributing you agree that your changes will be licensed under the [mit license](LICENSE).
