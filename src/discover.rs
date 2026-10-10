use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::space::{self, Source, Space};

/// Where to look for spaces, in precedence order. See docs/config.md.
#[derive(Debug, Clone)]
pub struct Search {
    pub cwd: PathBuf,
    pub home: Option<PathBuf>,
    /// `~/.config/gogo/spaces`.
    pub user: Option<PathBuf>,
    /// `GOGO_PATH`, then `spaces_path`.
    pub extra: Vec<PathBuf>,
}

/// A space file found on disk, loaded or not.
#[derive(Debug)]
pub struct Found {
    pub name: String,
    pub file: PathBuf,
    pub source: Source,
    pub space: Result<Space>,
    /// An earlier file has the same name, so this one is never used.
    pub shadowed: bool,
}

impl Search {
    /// Every space file, in precedence order, including shadowed ones.
    pub fn all(&self) -> Vec<Found> {
        let mut files: Vec<(PathBuf, Source)> = Vec::new();
        if let Some(project) = self.project_file() {
            files.push((project, Source::Project));
        }
        if let Some(user) = &self.user {
            files.extend(toml_files(user).into_iter().map(|f| (f, Source::User)));
        }
        for dir in &self.extra {
            files.extend(toml_files(dir).into_iter().map(|f| (f, Source::Path)));
        }

        let mut found: Vec<Found> = Vec::new();
        for (file, source) in files {
            if found.iter().any(|f| f.file == file) {
                continue;
            }
            let space = space::load(&file, source, &self.cwd);
            let name = match &space {
                Ok(s) => s.name.clone(),
                Err(_) => space::declared_name(&file).unwrap_or_else(|| space::default_name(&file, source)),
            };
            let shadowed = found.iter().any(|f| f.name == name);
            found.push(Found { name, file, source, space, shadowed });
        }
        found
    }

    /// The space a name resolves to: the first file with that name.
    pub fn find(&self, name: &str) -> Result<Space> {
        let all = self.all();
        match all.into_iter().find(|f| f.name == name) {
            Some(found) => found.space,
            None => Err(Error::not_found(format!(
                "no space called {name:?}. `gogo list` shows the ones there are"
            ))),
        }
    }

    /// `.gogo.toml` in `cwd` or the nearest parent, stopping at `$HOME`.
    /// Outside `$HOME`, only `cwd` itself is checked.
    fn project_file(&self) -> Option<PathBuf> {
        let under_home = self.home.as_ref().is_some_and(|h| self.cwd.starts_with(h));
        for dir in self.cwd.ancestors() {
            let file = dir.join(".gogo.toml");
            if file.is_file() {
                return Some(file);
            }
            if !under_home || Some(dir) == self.home.as_deref() {
                break;
            }
        }
        None
    }
}

fn toml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml") && p.is_file())
        .collect();
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const TAB: &str = "[[tab]]\nname = \"A\"\n";

    struct Fixture {
        root: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Fixture { root: TempDir::new().unwrap() }
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.root.path().join(rel)
        }

        fn write(&self, rel: &str, text: &str) {
            let path = self.path(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }

        fn search(&self, cwd: &str) -> Search {
            let cwd = self.path(cwd);
            fs::create_dir_all(&cwd).unwrap();
            Search {
                cwd,
                home: Some(self.path("home")),
                user: Some(self.path("home/.config/gogo/spaces")),
                extra: vec![self.path("dotfiles/spaces")],
            }
        }
    }

    #[test]
    fn finds_user_and_extra_spaces() {
        let fx = Fixture::new();
        fx.write("home/.config/gogo/spaces/blog.toml", TAB);
        fx.write("dotfiles/spaces/college.toml", TAB);
        fx.write("dotfiles/spaces/notes.txt", "not a space");
        let search = fx.search("home");
        let names: Vec<_> = search.all().into_iter().map(|f| (f.name, f.source)).collect();
        assert_eq!(names, [("blog".into(), Source::User), ("college".into(), Source::Path)]);
        assert_eq!(search.find("college").unwrap().name, "college");
    }

    #[test]
    fn project_space_wins_and_shadows() {
        let fx = Fixture::new();
        fx.write("home/code/blog/.gogo.toml", TAB);
        fx.write("home/.config/gogo/spaces/blog.toml", "dir = \"/elsewhere\"\n[[tab]]\nname = \"A\"\n");
        let search = fx.search("home/code/blog/src/deep");
        let all = search.all();
        assert_eq!(all[0].source, Source::Project);
        assert!(!all[0].shadowed);
        assert!(all[1].shadowed);
        assert_eq!(search.find("blog").unwrap().dir, fx.path("home/code/blog"));
    }

    #[test]
    fn project_search_stops_at_home() {
        let fx = Fixture::new();
        fx.write(".gogo.toml", TAB);
        let search = fx.search("home/code");
        assert!(search.all().is_empty());
    }

    #[test]
    fn broken_files_are_listed_under_their_name() {
        let fx = Fixture::new();
        fx.write("home/.config/gogo/spaces/broken.toml", "name = \"fixme\"\nnonsense = 1\n");
        let all = fx.search("home").all();
        assert_eq!(all[0].name, "fixme");
        assert!(all[0].space.is_err());
    }

    #[test]
    fn unknown_names_are_not_found() {
        let fx = Fixture::new();
        let err = fx.search("home").find("nope").unwrap_err();
        assert_eq!(err.kind, crate::error::Kind::NotFound);
    }
}
