use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::paths;

/// Where a space file was found. See "Where spaces live" in docs/config.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// `.gogo.toml` in the current directory or a parent.
    Project,
    /// `~/.config/gogo/spaces/*.toml`.
    User,
    /// A directory from `GOGO_PATH` or `spaces_path`.
    Path,
}

/// A space after defaults are applied and paths expanded.
#[derive(Debug, Clone)]
pub struct Space {
    pub name: String,
    pub file: PathBuf,
    pub source: Source,
    pub dir: PathBuf,
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub name: String,
    pub dir: PathBuf,
    /// The space's env with the tab's merged over it.
    pub env: BTreeMap<String, String>,
    pub focus: bool,
    pub run: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpaceFile {
    name: Option<String>,
    dir: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default, rename = "tab")]
    tabs: Vec<TabFile>,
    // Backend-only options. Accepted so files stay valid; unused in v0.1.
    #[allow(dead_code)]
    herdr: Option<toml::Table>,
    #[allow(dead_code)]
    tmux: Option<toml::Table>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TabFile {
    name: String,
    dir: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    focus: bool,
    run: Option<String>,
}

/// Keys from docs/config.md that later versions add. Named in the error so a
/// space written from the docs fails with a reason, not just "unknown field".
const PLANNED_KEYS: &[(&str, &str)] = &[
    ("agent", "v0.3"),
    ("agent_name", "v0.3"),
    ("agent_args", "v0.3"),
    ("prompt", "v0.3"),
    ("ready", "v0.3"),
    ("ready_timeout", "v0.3"),
    ("before", "v0.3"),
    ("vars", "v0.3"),
    ("pane", "v0.2"),
    ("split", "v0.2"),
    ("size", "v0.2"),
    ("extends", "v1.0"),
];

/// The name a space file gives itself, without validating the rest.
/// Lets `list` show a broken file under the right name.
pub fn declared_name(file: &Path) -> Option<String> {
    let text = fs::read_to_string(file).ok()?;
    let table: toml::Table = toml::from_str(&text).ok()?;
    table.get("name")?.as_str().map(str::to_owned)
}

/// The name a file gets when it doesn't set `name`: the directory name for
/// a project's `.gogo.toml`, the file stem otherwise.
pub fn default_name(file: &Path, source: Source) -> String {
    let named = match source {
        Source::Project => file.parent().and_then(Path::file_name),
        Source::User | Source::Path => file.file_stem(),
    };
    named.map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Reads and resolves a space file. `cwd` is the default `dir` for spaces
/// outside a project.
pub fn load(file: &Path, source: Source, cwd: &Path) -> Result<Space> {
    let text = fs::read_to_string(file).map_err(|e| Error::failed(format!("{}: {e}", file.display())))?;
    parse(&text, file, source, cwd)
}

pub fn parse(text: &str, file: &Path, source: Source, cwd: &Path) -> Result<Space> {
    let invalid = |message: String| Error::invalid(format!("{}: {message}", file.display()));
    let raw: SpaceFile = toml::from_str(text).map_err(|e| invalid(explain(e.message())))?;

    let name = raw.name.unwrap_or_else(|| default_name(file, source));
    if name.trim().is_empty() {
        return Err(invalid("the space has no name".into()));
    }
    if name.chars().any(char::is_control) {
        return Err(invalid(format!("space name {name:?} has control characters")));
    }

    let base = file.parent().unwrap_or(Path::new("/"));
    let expand = |raw: &str| paths::expand(raw, base).map_err(invalid);
    let dir = match (&raw.dir, source) {
        (Some(dir), _) => expand(dir)?,
        (None, Source::Project) => base.to_path_buf(),
        (None, _) => cwd.to_path_buf(),
    };

    if raw.tabs.is_empty() {
        return Err(invalid("a space needs at least one [[tab]]".into()));
    }
    let mut seen = HashSet::new();
    let mut tabs = Vec::with_capacity(raw.tabs.len());
    for tab in raw.tabs {
        if tab.name.trim().is_empty() {
            return Err(invalid("a tab has an empty name".into()));
        }
        if !seen.insert(tab.name.clone()) {
            return Err(invalid(format!("two tabs are called {:?}", tab.name)));
        }
        if let Some(run) = &tab.run {
            if run.trim().is_empty() {
                return Err(invalid(format!("tab {:?} has an empty run", tab.name)));
            }
        }
        let mut env = raw.env.clone();
        env.extend(tab.env);
        tabs.push(Tab {
            dir: match &tab.dir {
                Some(d) => expand(d)?,
                None => dir.clone(),
            },
            name: tab.name,
            env,
            focus: tab.focus,
            run: tab.run,
        });
    }

    let focused = tabs.iter().filter(|t| t.focus).count();
    if focused > 1 {
        return Err(invalid("more than one tab has focus = true".into()));
    }
    if focused == 0 {
        tabs[0].focus = true;
    }

    Ok(Space { name, file: file.to_path_buf(), source, dir, tabs })
}

/// Directories the space points at that don't exist.
pub fn missing_dirs(space: &Space) -> Vec<PathBuf> {
    let mut missing: Vec<PathBuf> = Vec::new();
    let dirs = std::iter::once(&space.dir).chain(space.tabs.iter().map(|t| &t.dir));
    for dir in dirs {
        if !dir.is_dir() && !missing.contains(dir) {
            missing.push(dir.clone());
        }
    }
    missing
}

fn explain(message: &str) -> String {
    let message = message.trim().to_owned();
    for (key, version) in PLANNED_KEYS {
        if message.contains(&format!("unknown field `{key}`")) {
            return format!("{message}\n`{key}` is planned for {version}, not supported yet");
        }
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(text: &str) -> Result<Space> {
        parse(text, Path::new("/spaces/blog.toml"), Source::User, Path::new("/cwd"))
    }

    #[test]
    fn single_tab_space_with_defaults() {
        let s = space("[[tab]]\nname = \"Shell\"\n").unwrap();
        assert_eq!(s.name, "blog");
        assert_eq!(s.dir, PathBuf::from("/cwd"));
        assert_eq!(s.tabs[0].dir, PathBuf::from("/cwd"));
        assert!(s.tabs[0].focus);
        assert!(s.tabs[0].run.is_none());
    }

    #[test]
    fn name_key_overrides_the_file_name() {
        let s = space("name = \"site\"\n[[tab]]\nname = \"A\"\n").unwrap();
        assert_eq!(s.name, "site");
    }

    #[test]
    fn project_space_defaults_to_its_directory() {
        let s = parse(
            "[[tab]]\nname = \"A\"\n",
            Path::new("/code/breen/.gogo.toml"),
            Source::Project,
            Path::new("/code/breen/app"),
        )
        .unwrap();
        assert_eq!(s.name, "breen");
        assert_eq!(s.dir, PathBuf::from("/code/breen"));
    }

    #[test]
    fn relative_dirs_resolve_against_the_file() {
        let s = space("dir = \"site\"\n[[tab]]\nname = \"A\"\ndir = \"docs\"\n").unwrap();
        assert_eq!(s.dir, PathBuf::from("/spaces/site"));
        assert_eq!(s.tabs[0].dir, PathBuf::from("/spaces/docs"));
    }

    #[test]
    fn tab_env_merges_over_space_env() {
        let s = space(
            "[env]\nA = \"1\"\nB = \"1\"\n[[tab]]\nname = \"T\"\n[tab.env]\nB = \"2\"\nC = \"3\"\n",
        )
        .unwrap();
        let env: Vec<_> = s.tabs[0].env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(env, ["A=1", "B=2", "C=3"]);
    }

    #[test]
    fn explicit_focus_moves_off_the_first_tab() {
        let s = space("[[tab]]\nname = \"A\"\n[[tab]]\nname = \"B\"\nfocus = true\n").unwrap();
        assert!(!s.tabs[0].focus);
        assert!(s.tabs[1].focus);
    }

    #[test]
    fn rejects_bad_spaces() {
        let cases = [
            ("dir = \"/x\"\n", "at least one [[tab]]"),
            ("[[tab]]\nname = \"A\"\n[[tab]]\nname = \"A\"\n", "two tabs"),
            ("[[tab]]\nname = \"A\"\nfocus = true\n[[tab]]\nname = \"B\"\nfocus = true\n", "more than one tab"),
            ("[[tab]]\nname = \"\"\n", "empty name"),
            ("[[tab]]\nname = \"A\"\nrun = \" \"\n", "empty run"),
            ("[[tab]]\nname = \"A\"\nrnu = \"x\"\n", "unknown field `rnu`"),
            ("name = \"\"\n[[tab]]\nname = \"A\"\n", "no name"),
        ];
        for (text, expected) in cases {
            let err = space(text).unwrap_err();
            assert_eq!(err.kind, crate::error::Kind::InvalidSpace);
            assert!(err.message.contains(expected), "{text:?}: {}", err.message);
            assert!(err.message.starts_with("/spaces/blog.toml: "), "{}", err.message);
        }
    }

    #[test]
    fn planned_keys_say_when_they_arrive() {
        let err = space("[[tab]]\nname = \"A\"\nagent = \"claude\"\n").unwrap_err();
        assert!(err.message.contains("planned for v0.3"), "{}", err.message);
    }

    #[test]
    fn missing_dirs_are_listed_once() {
        let s = space("dir = \"/surely/missing\"\n[[tab]]\nname = \"A\"\n[[tab]]\nname = \"B\"\n").unwrap();
        assert_eq!(missing_dirs(&s), [PathBuf::from("/surely/missing")]);
    }
}
