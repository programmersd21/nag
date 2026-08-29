use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::theme::ColorSupport;

pub const NAG_FIGLET: &str = r#" _ _  __ _ __ _ 
| ' \/ _` / _` |
|_||_\__,_\__, |
          |___/ "#;

pub fn print_custom_help(color: ColorSupport) {
    if !color.enabled {
        println!(
            r#"{NAG_FIGLET}

a tiny transparent command wrapper that tells you when a command finishes.

usage:
  nag [options] <command> [args...]
  nag [options] -- <command> [args...]

examples:
  nag npm run build
  nag -- cargo test --release
  nag --title "deploy site" ./deploy.sh
  nag --min-duration 10s -- python3 train.py
  nag --webhook https://hooks.slack.com/... ./build.sh
  nag -q make

options:
      --no-desktop             disable desktop notification upon completion
      --no-bell                disable terminal bell upon completion
      --no-title               disable terminal title updates
      --no-webhook             disable webhook notification
      --webhook <url>          webhook url for completion notification
  -q, --quiet                  suppress final summary line on stderr
      --min-duration <dur>     minimum duration before notifying (e.g. 10s, 500ms, 1m) [default: 0]
      --title <label>          override display label for the command
  -v, --verbose                print internal diagnostic messages to stderr
      --no-color               disable ansi colors and styling
  -h, --help                   print help
  -V, --version                print version"#
        );
        return;
    }

    // logo lines — four clearly distinct colors
    let l1 = color.accent(" _ _  __ _ __ _ "); // lavender  #cba6f7
    let l2 = color.sky("| ' \\/ _` / _` |"); // cyan      #89dceb
    let l3 = color.pink("|_||_\\__,_\\__, |"); // flamingo  #f5c2e7
    let l4 = color.mint("          |___/ "); // teal      #94e2d5

    let badge = format!(
        "{}{}{} {}",
        color.dim("["),
        color.peach("v0.1.0"),
        color.dim("]"),
        color.dim("·")
    );
    let tag = color.dim("notify when done");

    let desc =
        color.muted("a tiny transparent command wrapper that tells you when a command finishes.");

    // section dots — three hues clearly distinct from each other AND from
    // the `nag` keyword text (accent/lavender #cba6f7):
    // sky     = cyan/blue    #89dceb  hue≈193°
    // peach   = yellow       #f9e2af  hue≈ 43°
    // success = green        #a6e3a1  hue≈135°
    let h_usage = format!("{} {}", color.sky("●"), color.bold(&color.accent("usage")));
    let h_examples = format!(
        "{} {}",
        color.peach("●"),
        color.bold(&color.accent("examples"))
    );
    let h_options = format!(
        "{} {}",
        color.success("●"),
        color.bold(&color.accent("options"))
    );

    let bullet = color.dim("›");
    let nag = color.accent("nag");
    let dash = color.dim("--");

    let flag = |s: &str| color.sky(s);
    let val = |s: &str| color.peach(s);
    let muted_text = |s: &str| color.muted(s);
    let cmd = |s: &str| color.bright(s);

    // pre-build example fragments so no format! inside the println args
    let ex_title_flag = flag("--title");
    let ex_title_val = val("\"deploy site\"");
    let ex_min_flag = flag("--min-duration");
    let ex_min_val = val("10s");
    let ex_web_flag = flag("--webhook");
    let ex_web_val = val("https://hooks.slack.com/...");

    // options table: (raw_flag_str, colored_flag, description)
    let rows: [(&str, String, String); 12] = [
        (
            "--no-desktop",
            flag("--no-desktop"),
            muted_text("disable desktop notification upon completion"),
        ),
        (
            "--no-bell",
            flag("--no-bell"),
            muted_text("disable terminal bell upon completion"),
        ),
        (
            "--no-title",
            flag("--no-title"),
            muted_text("disable terminal title updates"),
        ),
        (
            "--no-webhook",
            flag("--no-webhook"),
            muted_text("disable webhook notification"),
        ),
        (
            "--webhook <url>",
            format!("{} {}", flag("--webhook"), val("<url>")),
            muted_text("webhook url for completion notification"),
        ),
        (
            "-q, --quiet",
            format!("{}, {}", flag("-q"), flag("--quiet")),
            muted_text("suppress final summary line on stderr"),
        ),
        (
            "--min-duration <dur>",
            format!("{} {}", flag("--min-duration"), val("<dur>")),
            muted_text("minimum duration before notifying [default: 0]"),
        ),
        (
            "--title <label>",
            format!("{} {}", flag("--title"), val("<label>")),
            muted_text("override display label for the command"),
        ),
        (
            "-v, --verbose",
            format!("{}, {}", flag("-v"), flag("--verbose")),
            muted_text("print internal diagnostic messages to stderr"),
        ),
        (
            "--no-color",
            flag("--no-color"),
            muted_text("disable ansi colors and styling"),
        ),
        (
            "-h, --help",
            format!("{}, {}", flag("-h"), flag("--help")),
            muted_text("print help"),
        ),
        (
            "-V, --version",
            format!("{}, {}", flag("-V"), flag("--version")),
            muted_text("print version"),
        ),
    ];

    let mut opts = String::new();
    for (raw, colored_flag, desc_str) in rows {
        let pad = " ".repeat(24usize.saturating_sub(raw.chars().count()));
        opts.push_str(&format!(
            "  {} {}{}{}\n",
            bullet, colored_flag, pad, desc_str
        ));
    }

    println!(
        r#"
{l1}
{l2}
{l3}
{l4}  {badge} {tag}

{desc}

{h_usage}
  {bullet} {nag} {} {}
  {bullet} {nag} {} {dash} {}

{h_examples}
  {bullet} {nag} {}
  {bullet} {nag} {dash} {}
  {bullet} {nag} {ex_title_flag} {ex_title_val} {}
  {bullet} {nag} {ex_min_flag} {ex_min_val} {dash} {}
  {bullet} {nag} {ex_web_flag} {ex_web_val} {}
  {bullet} {nag} {}

{h_options}
{opts}"#,
        color.dim("[options]"),
        val("<command> [args...]"),
        color.dim("[options]"),
        val("<command> [args...]"),
        cmd("npm run build"),
        cmd("cargo test --release"),
        cmd("./deploy.sh"),
        cmd("python3 train.py"),
        cmd("./build.sh"),
        cmd("-q make"),
        opts = opts.trim_end()
    );
}

#[derive(Parser, Debug)]
#[command(
    name = "nag",
    about = "a tiny transparent command wrapper that tells you when a command finishes.",
    version,
    disable_help_flag = true
)]
pub struct Cli {
    /// print help
    #[arg(short, long)]
    pub help: bool,

    /// disable desktop notification upon completion
    #[arg(long)]
    pub no_desktop: bool,

    /// disable terminal bell upon completion
    #[arg(long)]
    pub no_bell: bool,

    /// disable terminal title updates
    #[arg(long)]
    pub no_title: bool,

    /// disable webhook notification
    #[arg(long)]
    pub no_webhook: bool,

    /// webhook url for completion notification
    #[arg(long, value_name = "url")]
    pub webhook: Option<String>,

    /// suppress final summary line on stderr
    #[arg(short, long)]
    pub quiet: bool,

    /// minimum duration threshold before sending notifications (e.g. 5s, 500ms, 1m)
    #[arg(long, value_name = "dur")]
    pub min_duration: Option<String>,

    /// override display label for the command
    #[arg(long, value_name = "label")]
    pub title: Option<String>,

    /// print internal diagnostic messages to stderr
    #[arg(short, long)]
    pub verbose: bool,

    /// disable ansi colors and styling
    #[arg(long)]
    pub no_color: bool,

    /// explicit path to configuration file
    #[arg(long, value_name = "path", hide = true)]
    pub config: Option<PathBuf>,

    /// the command and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<OsString>,
}
