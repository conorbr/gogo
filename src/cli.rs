use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::backend;

/// Open a configured workspace in your terminal multiplexer.
///
/// The everyday form is `gogo <backend> <space>`, e.g. `gogo herdr blog`,
/// which is `gogo open blog --backend herdr`. `gogo <backend>` lists spaces,
/// marking the open ones. `gogo <backend> stop` closes every workspace and
/// stops the backend; `gogo <backend> stop <space>` closes one space's
/// workspace and leaves the backend running.
#[derive(Debug, Parser)]
#[command(name = "gogo", version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Backend to use. Default: GOGO_BACKEND, then config, then detected
    #[arg(short, long, global = true, value_name = "NAME")]
    pub backend: Option<String>,

    /// Machine-readable output on stdout
    #[arg(long, global = true)]
    pub json: bool,

    /// Errors only
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show each backend call
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Use this config file
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Plain output (gogo has no colour yet; accepted for scripts)
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Open a space, or focus it if it's open
    Open {
        space: String,
        /// Print the plan, change nothing
        #[arg(short = 'n', long)]
        dry_run: bool,
        #[command(flatten)]
        placement: Placement,
    },
    /// Print the plan for a space without running it
    Plan {
        space: String,
        #[command(flatten)]
        placement: Placement,
    },
    /// List spaces and where each is defined
    List {
        /// Only spaces open in the backend
        #[arg(long)]
        open: bool,
    },
    /// Close every workspace and stop every session in the backend, or close
    /// one space's workspace
    Stop {
        /// Close only this space's workspace, and leave the backend running
        space: Option<String>,
        /// List what would be stopped, change nothing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Validate space files. Every space if none is named
    Check { space: Option<String> },
    /// Print the version
    Version,
}

#[derive(Debug, Clone, Copy, clap::Args)]
pub struct Placement {
    /// Build it but don't attach
    #[arg(long)]
    pub no_attach: bool,
    /// Build it but leave focus where it is
    #[arg(long)]
    pub no_focus: bool,
}

const COMMANDS: &[&str] = &["open", "plan", "list", "stop", "check", "version", "help"];
/// Commands that read as `gogo <backend> <command>`. A space with one of these
/// names opens with `gogo open <space> --backend <backend>` instead.
const BACKEND_COMMANDS: &[&str] = &["stop"];
const TAKES_VALUE: &[&str] = &["-b", "--backend", "--config"];

/// Rewrites `gogo <backend> <space> ...` to `gogo open <space> --backend
/// <backend> ...`, `gogo <backend> stop [space]` to `gogo stop --backend
/// <backend> [space]`,
/// and `gogo <backend>` to `gogo list --backend <backend>`.
/// Anything else passes through for clap to parse.
pub fn expand_shorthand(args: Vec<String>) -> Vec<String> {
    let mut positionals = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            break;
        }
        if TAKES_VALUE.contains(&arg) {
            i += 2;
            continue;
        }
        if !arg.starts_with('-') {
            positionals.push(i);
        }
        i += 1;
    }
    let Some(&first) = positionals.first() else {
        return args;
    };
    let word = args[first].clone();
    if COMMANDS.contains(&word.as_str()) || !backend::is_backend_name(&word) {
        return args;
    }
    let mut out = args;
    if let Some(&second) = positionals.get(1) {
        if BACKEND_COMMANDS.contains(&out[second].as_str()) {
            out[first] = out.remove(second);
            out.insert(first + 1, "--backend".to_owned());
            out.insert(first + 2, word);
            return out;
        }
    }
    out[first] = if positionals.len() > 1 { "open" } else { "list" }.to_owned();
    out.insert(first + 1, "--backend".to_owned());
    out.insert(first + 2, word);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(line: &str) -> String {
        let args = std::iter::once("gogo").chain(line.split_whitespace()).map(String::from).collect();
        expand_shorthand(args)[1..].join(" ")
    }

    #[test]
    fn backend_and_space_open() {
        assert_eq!(expand("herdr blog"), "open --backend herdr blog");
        assert_eq!(expand("herdr blog --dry-run"), "open --backend herdr blog --dry-run");
        assert_eq!(expand("--json herdr blog"), "--json open --backend herdr blog");
        assert_eq!(expand("--config x.toml herdr blog"), "--config x.toml open --backend herdr blog");
    }

    #[test]
    fn backend_alone_lists() {
        assert_eq!(expand("herdr"), "list --backend herdr");
        assert_eq!(expand("herdr --open"), "list --backend herdr --open");
    }

    #[test]
    fn backend_and_command_runs_the_command() {
        assert_eq!(expand("herdr stop"), "stop --backend herdr");
        assert_eq!(expand("herdr stop -n"), "stop --backend herdr -n");
        assert_eq!(expand("--json herdr stop"), "--json stop --backend herdr");
        assert_eq!(expand("herdr blog stop"), "open --backend herdr blog stop");
    }

    #[test]
    fn backend_stop_space_closes_one() {
        assert_eq!(expand("herdr stop blog"), "stop --backend herdr blog");
        assert_eq!(expand("herdr stop blog -n"), "stop --backend herdr blog -n");
        let parse = |line: &[&str]| Cli::try_parse_from(expand_shorthand(line.iter().map(|s| s.to_string()).collect()));
        let cli = parse(&["gogo", "herdr", "stop", "blog"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Stop { space: Some(ref s), dry_run: false }) if s == "blog"));
        let cli = parse(&["gogo", "herdr", "stop"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Stop { space: None, .. })));
    }

    #[test]
    fn planned_backends_are_rewritten_too() {
        assert_eq!(expand("tmux blog"), "open --backend tmux blog");
    }

    #[test]
    fn everything_else_passes_through() {
        for line in ["", "list", "open blog", "blog", "check", "help herdr", "--backend herdr list", "stop"] {
            assert_eq!(expand(line), line);
        }
    }

    #[test]
    fn parses_after_expansion() {
        let args = expand_shorthand(["gogo", "herdr", "blog", "-n"].map(String::from).to_vec());
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.backend.as_deref(), Some("herdr"));
        assert!(matches!(cli.command, Some(Command::Open { ref space, dry_run: true, .. }) if space == "blog"));
    }
}
