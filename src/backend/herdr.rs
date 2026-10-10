//! The herdr backend, over the herdr CLI's JSON output. Tested against herdr
//! 0.9.3. See docs/backends/herdr.md.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

use serde_json::Value;

use super::Backend;
use crate::error::{Error, Result};
use crate::plan::Step;

pub struct Herdr {
    verbose: bool,
    workspace: Option<String>,
    /// Tab name to (tab ID, root pane ID), for tabs this run created.
    tabs: HashMap<String, (String, String)>,
}

impl Herdr {
    pub fn new(verbose: bool) -> Self {
        Herdr { verbose, workspace: None, tabs: HashMap::new() }
    }

    fn call(&self, args: &[&str]) -> Result<Value> {
        if self.verbose {
            eprintln!("herdr {}", args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "));
        }
        let output = Command::new("herdr").args(args).output().map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::unavailable("herdr isn't installed (not found on PATH)")
            } else {
                Error::failed(format!("couldn't run herdr: {e}"))
            }
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() { stdout } else { stderr };
            return Err(Error::failed(format!("herdr {} failed: {}", args.join(" "), detail.trim())));
        }
        Ok(serde_json::from_slice(&output.stdout).unwrap_or(Value::Null))
    }

    fn workspace(&self) -> Result<&str> {
        self.workspace.as_deref().ok_or_else(|| Error::failed("no herdr workspace yet"))
    }

    fn tab(&self, name: &str) -> Result<&(String, String)> {
        self.tabs.get(name).ok_or_else(|| Error::failed(format!("no herdr tab for {name:?} yet")))
    }

    fn workspaces(&self) -> Result<Vec<(String, String)>> {
        let list = self.call(&["workspace", "list"])?;
        let items = list.pointer("/result/workspaces").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(items
            .iter()
            .filter_map(|w| Some((str_at(w, "/workspace_id")?, str_at(w, "/label")?)))
            .collect())
    }
}

impl Backend for Herdr {
    fn name(&self) -> &'static str {
        "herdr"
    }

    fn available(&mut self) -> Result<()> {
        let output = Command::new("herdr").args(["status", "server"]).output().map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::unavailable("herdr isn't installed (not found on PATH)")
            } else {
                Error::unavailable(format!("couldn't run herdr: {e}"))
            }
        })?;
        let text = String::from_utf8_lossy(&output.stdout);
        if output.status.success() && text.lines().any(|l| l.trim() == "status: running") {
            Ok(())
        } else {
            Err(Error::unavailable("the herdr server isn't running. Run `herdr` once to start it"))
        }
    }

    fn inside(&self) -> bool {
        env::var("HERDR_ENV").is_ok_and(|v| v == "1")
    }

    fn find(&mut self, space: &str) -> Result<Option<String>> {
        let id = self.workspaces()?.into_iter().find(|(_, label)| label == space).map(|(id, _)| id);
        self.workspace = id.clone();
        Ok(id)
    }

    fn open_names(&mut self) -> Result<Vec<String>> {
        Ok(self.workspaces()?.into_iter().map(|(_, label)| label).collect())
    }

    fn apply(&mut self, step: &Step) -> Result<()> {
        match step {
            Step::CreateWorkspace { name, first_tab, dir, env, focus } => {
                let mut args = vec!["workspace", "create", "--cwd", dir, "--label", name];
                args.push(if *focus { "--focus" } else { "--no-focus" });
                let pairs = env_args(env);
                args.extend(pairs.iter().flat_map(|p| ["--env", p.as_str()]));
                let created = self.call(&args)?;
                let workspace = need(&created, "/result/workspace/workspace_id")?;
                let tab = need(&created, "/result/tab/tab_id")?;
                let pane = need(&created, "/result/root_pane/pane_id")?;
                self.call(&["tab", "rename", &tab, first_tab])?;
                self.workspace = Some(workspace);
                self.tabs.insert(first_tab.clone(), (tab, pane));
            }
            Step::CreateTab { name, dir, env } => {
                let workspace = self.workspace()?.to_owned();
                let mut args = vec!["tab", "create", "--workspace", &workspace, "--cwd", dir, "--label", name, "--no-focus"];
                let pairs = env_args(env);
                args.extend(pairs.iter().flat_map(|p| ["--env", p.as_str()]));
                let created = self.call(&args)?;
                let tab = need(&created, "/result/tab/tab_id")?;
                let pane = need(&created, "/result/root_pane/pane_id")?;
                self.tabs.insert(name.clone(), (tab, pane));
            }
            Step::Run { tab, command } => {
                let pane = self.tab(tab)?.1.clone();
                self.call(&["pane", "run", &pane, command])?;
            }
            Step::FocusWorkspace { .. } => {
                let workspace = self.workspace()?.to_owned();
                self.call(&["workspace", "focus", &workspace])?;
            }
            Step::FocusTab { tab } => {
                let id = self.tab(tab)?.0.clone();
                self.call(&["tab", "focus", &id])?;
            }
            Step::Attach => return Err(Error::failed("attach is not a step to apply")),
        }
        Ok(())
    }

    fn attach(&mut self) -> Error {
        if self.verbose {
            eprintln!("herdr");
        }
        let e = Command::new("herdr").exec();
        Error::failed(format!("couldn't attach to herdr: {e}"))
    }
}

fn str_at(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(Value::as_str).map(str::to_owned)
}

fn need(value: &Value, pointer: &str) -> Result<String> {
    str_at(value, pointer).ok_or_else(|| {
        Error::failed(format!("herdr's reply had no {pointer}. Is this herdr version supported? (tested: 0.9.3)"))
    })
}

fn env_args(env: &BTreeMap<String, String>) -> Vec<String> {
    env.iter().map(|(k, v)| format!("{k}={v}")).collect()
}

fn quote(arg: &str) -> String {
    if !arg.is_empty() && arg.chars().all(|c| c.is_ascii_alphanumeric() || "-_./:=@%+,".contains(c)) {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_only_when_needed() {
        assert_eq!(quote("w1:p3"), "w1:p3");
        assert_eq!(quote("npm run dev"), "'npm run dev'");
        assert_eq!(quote("it's"), r"'it'\''s'");
        assert_eq!(quote(""), "''");
    }

    #[test]
    fn reads_ids_from_replies() {
        let reply: Value = serde_json::from_str(
            r#"{"result":{"workspace":{"workspace_id":"w2"},"tab":{"tab_id":"w2:t1"},"root_pane":{"pane_id":"w2:p1"}}}"#,
        )
        .unwrap();
        assert_eq!(need(&reply, "/result/root_pane/pane_id").unwrap(), "w2:p1");
        assert!(need(&reply, "/result/nope").unwrap_err().message.contains("0.9.3"));
    }
}
