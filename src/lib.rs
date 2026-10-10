//! gogo: open a configured workspace in your terminal multiplexer.
//! Design: docs/architecture.md.

use clap::Parser;

pub mod backend;
pub mod cli;
mod commands;
pub mod config;
pub mod discover;
pub mod error;
pub mod paths;
pub mod plan;
pub mod space;

/// Runs gogo with these arguments (`args[0]` is the program name) and
/// returns the exit code.
pub fn main(args: Vec<String>) -> i32 {
    let cli = match cli::Cli::try_parse_from(cli::expand_shorthand(args)) {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            return e.exit_code();
        }
    };
    let json = cli.json;
    match commands::run(cli) {
        Ok(()) => 0,
        Err(e) => {
            if !e.message.is_empty() {
                if json {
                    let out = serde_json::json!({
                        "gogo": 1,
                        "error": { "kind": e.kind.as_str(), "code": e.kind.code(), "message": e.message },
                    });
                    println!("{out}");
                } else {
                    eprintln!("gogo: {e}");
                }
            }
            e.kind.code()
        }
    }
}
