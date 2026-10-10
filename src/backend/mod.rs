use serde::Serialize;

use crate::error::{Error, Result};
use crate::plan::{Plan, Step};

pub mod herdr;

/// A terminal multiplexer gogo can drive. See docs/architecture.md.
pub trait Backend {
    fn name(&self) -> &'static str;
    /// Installed, reachable, and running. Fails with exit code 4 if not.
    fn available(&mut self) -> Result<()>;
    /// Like `available`, but starts the backend first if it's installed and
    /// not running.
    fn start(&mut self) -> Result<()> {
        self.available()
    }
    /// gogo is running inside this backend right now.
    fn inside(&self) -> bool;
    /// The open workspace for this space, if there is one. Remembers it for
    /// later steps.
    fn find(&mut self, space: &str) -> Result<Option<String>>;
    /// Names of every open workspace.
    fn open_names(&mut self) -> Result<Vec<String>>;
    /// Runs one step. `Step::Attach` is never passed here; see `attach`.
    fn apply(&mut self, step: &Step) -> Result<()>;
    /// Closes every workspace and stops every session, or only lists them
    /// when `dry_run`. Anything that would end gogo itself waits for
    /// `finish_stop`.
    fn stop(&mut self, dry_run: bool) -> Result<Vec<Stopped>>;
    /// Ends what `stop` left: the workspace and session gogo runs in.
    fn finish_stop(&mut self) -> Result<()>;
    /// Hands the terminal to the backend's client. Only returns on failure.
    fn attach(&mut self) -> Error;
}

/// Backends that exist.
pub const NAMES: &[&str] = &["herdr"];
/// Backends that are planned. Recognised on the command line so they fail
/// with a useful message.
pub const PLANNED: &[(&str, &str)] = &[("tmux", "v0.5")];

pub fn is_backend_name(name: &str) -> bool {
    NAMES.contains(&name) || PLANNED.iter().any(|(n, _)| *n == name)
}

pub fn get(name: &str, verbose: bool) -> Result<Box<dyn Backend>> {
    match name {
        "herdr" => Ok(Box::new(herdr::Herdr::new(verbose))),
        _ => match PLANNED.iter().find(|(n, _)| *n == name) {
            Some((_, version)) => Err(Error::not_found(format!("{name} support is planned for {version}"))),
            None => Err(Error::not_found(format!(
                "no backend called {name:?}. Backends: {}",
                NAMES.join(", ")
            ))),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Created,
    Focused,
    /// Open already, and nothing was asked of it.
    Unchanged,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub action: Action,
    /// The plan ends in `Attach`; the caller attaches after reporting.
    pub attach: bool,
}

/// A session `stop` stopped, and the workspaces it closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stopped {
    pub session: String,
    pub workspaces: Vec<String>,
}

/// Runs a plan up to, not including, `Attach`.
pub fn execute(plan: &Plan, backend: &mut dyn Backend) -> Result<Outcome> {
    backend.start()?;
    let (steps, action) = match backend.find(&plan.space)? {
        Some(_) if plan.if_open.iter().any(|s| *s != Step::Attach) => (&plan.if_open, Action::Focused),
        Some(_) => (&plan.if_open, Action::Unchanged),
        None => (&plan.otherwise, Action::Created),
    };
    let mut attach = false;
    for step in steps {
        if *step == Step::Attach {
            attach = true;
        } else {
            backend.apply(step)?;
        }
    }
    Ok(Outcome { action, attach })
}

#[cfg(test)]
pub mod fake {
    use super::*;

    /// Records steps instead of running them.
    #[derive(Default)]
    pub struct Fake {
        pub open: Vec<String>,
        pub inside: bool,
        pub applied: Vec<Step>,
        pub fail_on: Option<usize>,
    }

    impl Backend for Fake {
        fn name(&self) -> &'static str {
            "fake"
        }
        fn available(&mut self) -> Result<()> {
            Ok(())
        }
        fn inside(&self) -> bool {
            self.inside
        }
        fn find(&mut self, space: &str) -> Result<Option<String>> {
            Ok(self.open.iter().find(|n| *n == space).map(|_| "id".into()))
        }
        fn open_names(&mut self) -> Result<Vec<String>> {
            Ok(self.open.clone())
        }
        fn apply(&mut self, step: &Step) -> Result<()> {
            assert_ne!(*step, Step::Attach, "attach must not reach apply");
            if self.fail_on == Some(self.applied.len()) {
                return Err(Error::failed("boom"));
            }
            self.applied.push(step.clone());
            Ok(())
        }
        fn stop(&mut self, dry_run: bool) -> Result<Vec<Stopped>> {
            let workspaces = if dry_run { self.open.clone() } else { std::mem::take(&mut self.open) };
            Ok(vec![Stopped { session: "default".into(), workspaces }])
        }
        fn finish_stop(&mut self) -> Result<()> {
            Ok(())
        }
        fn attach(&mut self) -> Error {
            Error::failed("fake can't attach")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::Fake;
    use super::*;
    use crate::plan::{plan, Options};
    use crate::space::{parse, Source};
    use std::path::Path;

    fn blog_plan(options: Options) -> Plan {
        let space = parse(
            "dir = \"/srv\"\n[[tab]]\nname = \"A\"\nrun = \"x\"\n[[tab]]\nname = \"B\"\n",
            Path::new("/s/blog.toml"),
            Source::User,
            Path::new("/"),
        )
        .unwrap();
        plan(&space, "fake", options)
    }

    #[test]
    fn creates_when_closed() {
        let mut fake = Fake::default();
        let out = execute(&blog_plan(Options { inside: false, attach: true, focus: true }), &mut fake).unwrap();
        assert_eq!(out.action, Action::Created);
        assert!(out.attach);
        assert_eq!(fake.applied.len(), 3);
    }

    #[test]
    fn focuses_when_open() {
        let mut fake = Fake { open: vec!["blog".into()], ..Default::default() };
        let out = execute(&blog_plan(Options { inside: true, attach: true, focus: true }), &mut fake).unwrap();
        assert_eq!(out.action, Action::Focused);
        assert!(!out.attach);
        assert_eq!(fake.applied, [Step::FocusWorkspace { name: "blog".into() }]);
    }

    #[test]
    fn unchanged_when_open_and_nothing_asked() {
        let mut fake = Fake { open: vec!["blog".into()], ..Default::default() };
        let out = execute(&blog_plan(Options { inside: true, attach: true, focus: false }), &mut fake).unwrap();
        assert_eq!(out.action, Action::Unchanged);
        assert!(fake.applied.is_empty());
    }

    #[test]
    fn stops_at_the_first_failure() {
        let mut fake = Fake { fail_on: Some(1), ..Default::default() };
        let err = execute(&blog_plan(Options { inside: true, attach: false, focus: true }), &mut fake).unwrap_err();
        assert_eq!(err.message, "boom");
        assert_eq!(fake.applied.len(), 1);
    }

    #[test]
    fn planned_backends_say_when() {
        let err = get("tmux", false).err().unwrap();
        assert_eq!(err.kind, crate::error::Kind::NotFound);
        assert!(err.message.contains("v0.5"));
        assert!(is_backend_name("tmux") && is_backend_name("herdr") && !is_backend_name("blog"));
    }
}
