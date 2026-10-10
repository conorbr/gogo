use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::backend::{self, Action};
use crate::cli::{Cli, Command, Placement};
use crate::config::Config;
use crate::discover::{Found, Search};
use crate::error::{Error, Kind, Result};
use crate::paths;
use crate::plan::{self, Options, Plan};
use crate::space::{self, Source, Space};

/// The JSON output version. See "Output" in docs/cli.md.
const JSON_VERSION: u32 = 1;

struct Ctx {
    json: bool,
    quiet: bool,
    verbose: bool,
    flag_backend: Option<String>,
    config: Config,
    search: Search,
}

pub fn run(cli: Cli) -> Result<()> {
    let config = Config::load(cli.config.as_deref())?;
    let search = search(&config)?;
    let ctx = Ctx {
        json: cli.json,
        quiet: cli.quiet,
        verbose: cli.verbose,
        flag_backend: cli.backend,
        config,
        search,
    };
    match cli.command {
        None => list(&ctx, false),
        Some(Command::Open { space, dry_run, placement }) => open(&ctx, &space, placement, dry_run),
        Some(Command::Plan { space, placement }) => open(&ctx, &space, placement, true),
        Some(Command::List { open }) => list(&ctx, open),
        Some(Command::Check { space }) => check(&ctx, space.as_deref()),
        Some(Command::Version) => version(&ctx),
    }
}

fn search(config: &Config) -> Result<Search> {
    let cwd = env::current_dir().map_err(|e| Error::failed(format!("can't read the current directory: {e}")))?;
    let mut extra = Vec::new();
    let gogo_path = env::var("GOGO_PATH").unwrap_or_default();
    let configured = gogo_path.split(':').filter(|p| !p.is_empty()).map(str::to_owned);
    for raw in configured.chain(config.spaces_path.iter().cloned()) {
        extra.push(paths::expand(&raw, &cwd).map_err(|e| Error::failed(format!("spaces path {raw:?}: {e}")))?);
    }
    Ok(Search {
        cwd,
        home: paths::home(),
        user: paths::config_dir().map(|d| d.join("spaces")),
        extra,
    })
}

/// `--backend`, `GOGO_BACKEND`, config, then detected from the environment.
fn choose_backend(ctx: &Ctx) -> Result<String> {
    let chosen = ctx
        .flag_backend
        .clone()
        .or_else(|| env::var("GOGO_BACKEND").ok().filter(|b| !b.is_empty()))
        .or_else(|| ctx.config.default_backend.clone())
        .or_else(|| {
            if env::var("HERDR_ENV").is_ok_and(|v| v == "1") {
                Some("herdr".to_owned())
            } else if env::var_os("TMUX").is_some_and(|v| !v.is_empty()) {
                Some("tmux".to_owned())
            } else {
                None
            }
        });
    chosen.ok_or_else(|| Error::usage("name a backend, e.g. `gogo herdr <space>` or `--backend herdr`"))
}

fn open(ctx: &Ctx, name: &str, placement: Placement, dry_run: bool) -> Result<()> {
    let space = ctx.search.find(name)?;
    let missing = space::missing_dirs(&space);
    let backend_name = choose_backend(ctx)?;
    let mut backend = backend::get(&backend_name, ctx.verbose)?;
    let options = Options {
        inside: backend.inside(),
        attach: !placement.no_attach && ctx.config.attach(&backend_name),
        focus: !placement.no_focus,
    };
    let plan = plan::plan(&space, &backend_name, options);

    if dry_run {
        for dir in &missing {
            warn(ctx, &format!("{}: dir doesn't exist: {}", space.name, tilde(dir)));
        }
        return print_plan(ctx, &plan);
    }
    if let Some(dir) = missing.first() {
        return Err(Error::failed(format!("{}: dir doesn't exist: {}", space.name, tilde(dir))));
    }

    let outcome = backend::execute(&plan, backend.as_mut())?;
    if ctx.json {
        emit(json!({
            "gogo": JSON_VERSION,
            "command": "open",
            "space": space.name,
            "backend": backend_name,
            "action": outcome.action,
            "attach": outcome.attach,
        }));
    } else if !ctx.quiet {
        let line = match outcome.action {
            Action::Created => format!("opened {} in {backend_name}", space.name),
            Action::Focused => format!("focused {} in {backend_name}", space.name),
            Action::Unchanged => format!("{} is already open in {backend_name}", space.name),
        };
        println!("{line}");
    }
    if outcome.attach {
        let _ = std::io::stdout().flush();
        return Err(backend.attach());
    }
    Ok(())
}

fn print_plan(ctx: &Ctx, plan: &Plan) -> Result<()> {
    if ctx.json {
        emit(json!({ "gogo": JSON_VERSION, "command": "plan", "plan": plan }));
    } else {
        print!("{}", plan.render());
    }
    Ok(())
}

fn list(ctx: &Ctx, only_open: bool) -> Result<()> {
    let found = ctx.search.all();

    // Mark open spaces when a backend is named, or required by --open.
    let named = ctx.flag_backend.is_some() || env::var("GOGO_BACKEND").is_ok_and(|b| !b.is_empty());
    let open: Option<(String, Vec<String>)> = if only_open || named {
        let name = choose_backend(ctx)?;
        let names = backend::get(&name, ctx.verbose).and_then(|mut b| {
            b.available()?;
            b.open_names()
        });
        match names {
            Ok(names) => Some((name, names)),
            Err(e) if only_open => return Err(e),
            Err(e) => {
                warn(ctx, &format!("can't see what's open: {e}"));
                None
            }
        }
    } else {
        None
    };
    let is_open = |f: &Found| open.as_ref().map(|(_, names)| !f.shadowed && names.contains(&f.name));
    let shown: Vec<&Found> = found.iter().filter(|f| !only_open || is_open(f) == Some(true)).collect();

    if ctx.json {
        let spaces: Vec<_> = shown
            .iter()
            .map(|f| {
                let mut entry = json!({
                    "name": f.name,
                    "file": f.file,
                    "source": f.source,
                    "valid": f.space.is_ok(),
                    "shadowed": f.shadowed,
                });
                if let Some(open) = is_open(f) {
                    entry["open"] = json!(open);
                }
                if let Err(e) = &f.space {
                    entry["error"] = json!(e.message);
                }
                entry
            })
            .collect();
        let mut out = json!({ "gogo": JSON_VERSION, "command": "list", "spaces": spaces });
        if let Some((name, _)) = &open {
            out["backend"] = json!(name);
        }
        emit(out);
        return Ok(());
    }

    if shown.is_empty() {
        if !ctx.quiet {
            let hint = match paths::config_dir() {
                Some(dir) => format!(". Add one at {}", tilde(&dir.join("spaces").join("<name>.toml"))),
                None => String::new(),
            };
            eprintln!("{}{hint}", if only_open { "no spaces are open" } else { "no spaces yet" });
        }
        return Ok(());
    }
    let width = shown.iter().map(|f| f.name.chars().count()).max().unwrap_or(0);
    for f in shown {
        let mark = if is_open(f) == Some(true) { "*" } else { " " };
        let mut notes = Vec::new();
        if f.shadowed {
            notes.push("shadowed".to_owned());
        }
        if f.space.is_err() {
            notes.push("invalid, see `gogo check`".to_owned());
        }
        let notes = if notes.is_empty() { String::new() } else { format!("  ({})", notes.join(", ")) };
        println!("{mark} {:<width$}  {:<7}  {}{notes}", f.name, source_label(f.source), tilde(&f.file));
    }
    Ok(())
}

fn check(ctx: &Ctx, name: Option<&str>) -> Result<()> {
    let found = ctx.search.all();
    let targets: Vec<&Found> = found.iter().filter(|f| name.is_none_or(|n| f.name == n)).collect();
    if let Some(name) = name {
        if targets.is_empty() {
            return Err(Error::not_found(format!("no space called {name:?}")));
        }
    }

    let mut results = Vec::new();
    for f in &targets {
        let problems: Vec<String> = match &f.space {
            Err(e) => vec![e.message.clone()],
            Ok(space) => missing_dir_problems(space),
        };
        let shadowed_by = if f.shadowed {
            found.iter().find(|other| other.name == f.name && !other.shadowed).map(|o| o.file.clone())
        } else {
            None
        };
        results.push((f, problems, shadowed_by));
    }
    let invalid = results.iter().filter(|(_, problems, _)| !problems.is_empty()).count();

    if ctx.json {
        let spaces: Vec<_> = results
            .iter()
            .map(|(f, problems, shadowed_by)| {
                json!({
                    "name": f.name,
                    "file": f.file,
                    "valid": problems.is_empty(),
                    "problems": problems,
                    "shadowed_by": shadowed_by,
                })
            })
            .collect();
        emit(json!({ "gogo": JSON_VERSION, "command": "check", "spaces": spaces }));
    } else {
        for (f, problems, shadowed_by) in &results {
            if problems.is_empty() {
                if !ctx.quiet {
                    println!("ok       {}  {}", f.name, tilde(&f.file));
                }
            } else {
                for problem in problems {
                    println!("invalid  {}  {problem}", f.name);
                }
            }
            if let Some(by) = shadowed_by {
                warn(ctx, &format!("{} in {} is shadowed by {}", f.name, tilde(&f.file), tilde(by)));
            }
        }
        if results.is_empty() && !ctx.quiet {
            eprintln!("no spaces to check");
        }
    }

    if invalid > 0 {
        if !ctx.json {
            eprintln!("gogo: {invalid} of {} spaces invalid", results.len());
        }
        // Already reported; an empty message tells main not to repeat it.
        return Err(Error::new(Kind::InvalidSpace, ""));
    }
    Ok(())
}

fn missing_dir_problems(space: &Space) -> Vec<String> {
    space::missing_dirs(space).iter().map(|d| format!("dir doesn't exist: {}", tilde(d))).collect()
}

fn version(ctx: &Ctx) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    if ctx.json {
        emit(json!({ "gogo": JSON_VERSION, "command": "version", "version": version }));
    } else {
        println!("gogo {version}");
    }
    Ok(())
}

fn source_label(source: Source) -> &'static str {
    match source {
        Source::Project => "project",
        Source::User => "user",
        Source::Path => "path",
    }
}

fn warn(ctx: &Ctx, message: &str) {
    if !ctx.quiet {
        eprintln!("gogo: warning: {message}");
    }
}

fn emit(value: serde_json::Value) {
    println!("{value}");
}

/// Shows paths under `$HOME` as `~/...`.
fn tilde(path: &Path) -> String {
    if let Some(home) = paths::home() {
        if let Ok(rest) = path.strip_prefix(&home) {
            return PathBuf::from("~").join(rest).display().to_string();
        }
    }
    path.display().to_string()
}
