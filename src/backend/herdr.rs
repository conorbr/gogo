//! The herdr backend, over the herdr CLI's JSON output. Tested against herdr
//! 0.9.3. See docs/backends/herdr.md.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::{Backend, Stopped};
use crate::error::{Error, Result};
use crate::plan::Step;

pub struct Herdr {
    verbose: bool,
    workspace: Option<String>,
    /// Tab name to (tab ID, root pane ID), for tabs this run created.
    tabs: HashMap<String, (String, String)>,
    /// The session gogo runs in, as (name, socket, gogo's workspace ID), left
    /// for `finish_stop`.
    stop_last: Option<(String, String, Option<String>)>,
    /// gogo's own workspace, as (socket, workspace ID), when `close` was asked
    /// to close it. Left for `finish_stop`.
    close_last: Option<(Option<String>, String)>,
}

impl Herdr {
    pub fn new(verbose: bool) -> Self {
        Herdr { verbose, workspace: None, tabs: HashMap::new(), stop_last: None, close_last: None }
    }

    fn call(&self, args: &[&str]) -> Result<Value> {
        self.call_at(None, args)
    }

    /// Runs a herdr command against the session listening on `socket`, or the
    /// inherited one.
    fn call_at(&self, socket: Option<&str>, args: &[&str]) -> Result<Value> {
        if self.verbose {
            let prefix = socket.map(|s| format!("HERDR_SOCKET_PATH={} ", quote(s))).unwrap_or_default();
            eprintln!("{prefix}herdr {}", args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "));
        }
        let mut command = Command::new("herdr");
        command.args(args);
        if let Some(socket) = socket {
            command.env("HERDR_SOCKET_PATH", socket);
        }
        let output = command.output().map_err(|e| {
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
        self.workspaces_at(None)
    }

    /// (ID, label) of each workspace in a session.
    fn workspaces_at(&self, socket: Option<&str>) -> Result<Vec<(String, String)>> {
        let list = self.call_at(socket, &["workspace", "list"])?;
        let items = list.pointer("/result/workspaces").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(items
            .iter()
            .filter_map(|w| Some((str_at(w, "/workspace_id")?, str_at(w, "/label")?)))
            .collect())
    }

    /// (name, socket) of each running session, with the one gogo runs in last.
    fn sessions(&self) -> Result<Vec<(String, String)>> {
        let list = self.call(&["session", "list", "--json"])?;
        let items = list.pointer("/sessions").and_then(Value::as_array).cloned().unwrap_or_default();
        let mut sessions: Vec<(String, String)> = items
            .iter()
            .filter(|s| s.pointer("/running").and_then(Value::as_bool) == Some(true))
            .filter_map(|s| Some((str_at(s, "/name")?, str_at(s, "/socket_path")?)))
            .collect();
        let own = self.own_socket();
        sessions.sort_by_key(|(_, socket)| own.as_deref() == Some(socket.as_str()));
        Ok(sessions)
    }

    /// Whether the herdr server is up. Fails only if herdr can't be run.
    fn running(&self) -> Result<bool> {
        let output = Command::new("herdr").args(["status", "server"]).output().map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::unavailable("herdr isn't installed (not found on PATH)")
            } else {
                Error::unavailable(format!("couldn't run herdr: {e}"))
            }
        })?;
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(output.status.success() && text.lines().any(|l| l.trim() == "status: running"))
    }

    /// The socket of the session gogo is running in, if it is.
    fn own_socket(&self) -> Option<String> {
        if !self.inside() {
            return None;
        }
        env::var("HERDR_SOCKET_PATH").ok().filter(|s| !s.is_empty())
    }
}

impl Backend for Herdr {
    fn name(&self) -> &'static str {
        "herdr"
    }

    fn available(&mut self) -> Result<()> {
        if self.running()? {
            Ok(())
        } else {
            Err(Error::unavailable("the herdr server isn't running. Run `herdr` once to start it"))
        }
    }

    fn start(&mut self) -> Result<()> {
        if self.running()? {
            return Ok(());
        }
        if self.verbose {
            eprintln!("herdr server");
        }
        let mut command = Command::new("herdr");
        command.arg("server").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        detach(&mut command);
        command.spawn().map_err(|e| Error::unavailable(format!("couldn't start the herdr server: {e}")))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(100));
            if self.running()? {
                return Ok(());
            }
        }
        Err(Error::unavailable("started the herdr server, but it didn't come up within 5s"))
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

    fn stop(&mut self, dry_run: bool) -> Result<Vec<Stopped>> {
        let own_socket = self.own_socket();
        let own_workspace = env::var("HERDR_WORKSPACE_ID").ok().filter(|_| own_socket.is_some());
        let mut stopped = Vec::new();
        for (name, socket) in self.sessions()? {
            let mine = own_socket.as_deref() == Some(socket.as_str());
            let workspaces = self.workspaces_at(Some(&socket))?;
            if !dry_run {
                // Closed first: herdr restores a stopped session's workspaces
                // when it next starts. gogo's own workspace and session go
                // last, in `finish_stop`, since closing them ends gogo.
                for (id, _) in &workspaces {
                    if !(mine && own_workspace.as_deref() == Some(id.as_str())) {
                        self.call_at(Some(&socket), &["workspace", "close", id])?;
                    }
                }
                if mine {
                    self.stop_last = Some((name.clone(), socket.clone(), own_workspace.clone()));
                } else {
                    self.call(&["session", "stop", &name, "--json"])?;
                }
            }
            stopped.push(Stopped { session: name, workspaces: workspaces.into_iter().map(|(_, label)| label).collect() });
        }
        Ok(stopped)
    }

    fn close(&mut self, space: &str, dry_run: bool) -> Result<bool> {
        if !self.running()? {
            return Ok(false);
        }
        let Some(id) = self.find(space)? else {
            return Ok(false);
        };
        if dry_run {
            return Ok(true);
        }
        let own = self.inside() && env::var("HERDR_WORKSPACE_ID").is_ok_and(|w| w == id);
        if own {
            // Closing it ends gogo, so it waits for `finish_stop`.
            self.close_last = Some((self.own_socket(), id));
        } else {
            self.call(&["workspace", "close", &id])?;
        }
        Ok(true)
    }

    fn finish_stop(&mut self) -> Result<()> {
        if let Some((socket, workspace)) = self.close_last.take() {
            // Handed to a process in its own Unix session, as below.
            if self.verbose {
                let prefix = socket.as_deref().map(|s| format!("HERDR_SOCKET_PATH={} ", quote(s))).unwrap_or_default();
                eprintln!("{prefix}herdr workspace close {}", quote(&workspace));
            }
            let mut command = Command::new("herdr");
            command
                .args(["workspace", "close", &workspace])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            if let Some(socket) = &socket {
                command.env("HERDR_SOCKET_PATH", socket);
            }
            detach(&mut command);
            command
                .spawn()
                .map_err(|e| Error::failed(format!("couldn't close herdr workspace {workspace}: {e}")))?;
        }
        let Some((name, socket, workspace)) = self.stop_last.take() else {
            return Ok(());
        };
        // Handed to a process in its own Unix session: closing a workspace
        // kills every process in its panes' sessions, gogo included.
        let script = r#"[ -n "$2" ] && herdr workspace close "$2" >/dev/null; herdr session stop "$1" --json >/dev/null"#;
        let workspace = workspace.unwrap_or_default();
        if self.verbose {
            eprintln!("HERDR_SOCKET_PATH={} herdr workspace close {}", quote(&socket), quote(&workspace));
            eprintln!("herdr session stop {} --json", quote(&name));
        }
        let mut command = Command::new("sh");
        command
            .args(["-c", script, "sh", &name, &workspace])
            .env("HERDR_SOCKET_PATH", &socket)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        detach(&mut command);
        command
            .spawn()
            .map_err(|e| Error::failed(format!("couldn't stop herdr session {name}: {e}")))?;
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

/// Runs `command` in its own Unix session, so it outlives gogo and anything
/// that kills gogo's.
fn detach(command: &mut Command) {
    // SAFETY: setsid is async-signal-safe and touches no Rust state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
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
