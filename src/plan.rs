use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;

use crate::space::Space;

/// One thing a backend does. Tabs are named by their space name, never by
/// backend IDs; the backend keeps that mapping. See docs/architecture.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum Step {
    /// Make the workspace. Its first tab comes with it.
    CreateWorkspace {
        name: String,
        first_tab: String,
        dir: String,
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
        focus: bool,
    },
    CreateTab {
        name: String,
        dir: String,
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
    },
    Run { tab: String, command: String },
    FocusWorkspace { name: String },
    FocusTab { tab: String },
    /// Hand the terminal to the backend. Always last.
    Attach,
}

/// What gogo will do: one list if the workspace is already open, another
/// if it isn't. Plain data, so `--dry-run` and `--json` just print it.
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub space: String,
    pub backend: String,
    pub if_open: Vec<Step>,
    pub otherwise: Vec<Step>,
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// gogo is running inside the backend already.
    pub inside: bool,
    /// Attach when outside (`--no-attach` and `[backend] attach` turn it off).
    pub attach: bool,
    /// Move focus to the space (`--no-focus` turns it off).
    pub focus: bool,
}

pub fn plan(space: &Space, backend: &str, options: Options) -> Plan {
    let attach = options.attach && !options.inside;
    // Attaching shows whatever is focused, so attaching implies focusing.
    let focus = options.focus || attach;

    let mut if_open = Vec::new();
    if focus {
        if_open.push(Step::FocusWorkspace { name: space.name.clone() });
    }

    let mut otherwise = Vec::new();
    for (i, tab) in space.tabs.iter().enumerate() {
        let dir = tab.dir.to_string_lossy().into_owned();
        otherwise.push(if i == 0 {
            Step::CreateWorkspace {
                name: space.name.clone(),
                first_tab: tab.name.clone(),
                dir,
                env: tab.env.clone(),
                focus,
            }
        } else {
            Step::CreateTab { name: tab.name.clone(), dir, env: tab.env.clone() }
        });
        if let Some(command) = &tab.run {
            otherwise.push(Step::Run { tab: tab.name.clone(), command: command.clone() });
        }
    }
    let focused = space.tabs.iter().position(|t| t.focus).unwrap_or(0);
    if focus && focused != 0 {
        otherwise.push(Step::FocusTab { tab: space.tabs[focused].name.clone() });
    }

    if attach {
        if_open.push(Step::Attach);
        otherwise.push(Step::Attach);
    }

    Plan { space: space.name.clone(), backend: backend.to_owned(), if_open, otherwise }
}

impl Plan {
    /// Human-readable, one step per line. What `--dry-run` prints.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "plan for {} in {}", self.space, self.backend);
        let _ = writeln!(out, "if it's open:");
        if self.if_open.is_empty() {
            let _ = writeln!(out, "  nothing");
        }
        for step in &self.if_open {
            let _ = writeln!(out, "  {}", describe(step));
        }
        let _ = writeln!(out, "otherwise:");
        for step in &self.otherwise {
            let _ = writeln!(out, "  {}", describe(step));
        }
        out
    }
}

fn describe(step: &Step) -> String {
    match step {
        Step::CreateWorkspace { name, first_tab, dir, env, focus } => format!(
            "create workspace {name} with tab {first_tab} in {dir}{}{}",
            env_suffix(env),
            if *focus { ", focused" } else { "" }
        ),
        Step::CreateTab { name, dir, env } => format!("create tab {name} in {dir}{}", env_suffix(env)),
        Step::Run { tab, command } => format!("run in {tab}: {command}"),
        Step::FocusWorkspace { name } => format!("focus workspace {name}"),
        Step::FocusTab { tab } => format!("focus tab {tab}"),
        Step::Attach => "attach".to_owned(),
    }
}

fn env_suffix(env: &BTreeMap<String, String>) -> String {
    if env.is_empty() {
        return String::new();
    }
    let pairs: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!(" with {}", pairs.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::{parse, Source};
    use std::path::Path;

    fn blog() -> Space {
        parse(
            "dir = \"/srv/blog\"\n\
             [[tab]]\nname = \"Claude\"\nrun = \"claude\"\n\
             [[tab]]\nname = \"Shell\"\n",
            Path::new("/spaces/blog.toml"),
            Source::User,
            Path::new("/"),
        )
        .unwrap()
    }

    #[test]
    fn inside_never_attaches() {
        let p = plan(&blog(), "herdr", Options { inside: true, attach: true, focus: true });
        assert!(!p.otherwise.contains(&Step::Attach));
        assert_eq!(p.if_open, [Step::FocusWorkspace { name: "blog".into() }]);
    }

    #[test]
    fn outside_attaches_and_so_focuses() {
        let p = plan(&blog(), "herdr", Options { inside: false, attach: true, focus: false });
        assert_eq!(p.otherwise.last(), Some(&Step::Attach));
        assert_eq!(p.if_open, [Step::FocusWorkspace { name: "blog".into() }, Step::Attach]);
    }

    #[test]
    fn no_focus_inside_does_nothing_when_open() {
        let p = plan(&blog(), "herdr", Options { inside: true, attach: true, focus: false });
        assert!(p.if_open.is_empty());
        assert!(matches!(p.otherwise[0], Step::CreateWorkspace { focus: false, .. }));
    }

    #[test]
    fn a_tab_without_run_gets_no_run_step() {
        let p = plan(&blog(), "herdr", Options { inside: true, attach: false, focus: true });
        let runs = p.otherwise.iter().filter(|s| matches!(s, Step::Run { .. })).count();
        assert_eq!(runs, 1);
    }
}
