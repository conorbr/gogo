use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::paths;

/// `~/.config/gogo/config.toml`. Every key is optional. See docs/config.md.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub default_backend: Option<String>,
    #[serde(default)]
    pub spaces_path: Vec<String>,
    pub editor: Option<String>,
    pub herdr: Option<BackendConfig>,
    pub tmux: Option<BackendConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendConfig {
    pub attach: Option<bool>,
    pub send_prompts: Option<bool>,
}

impl Config {
    /// `--config`, else `GOGO_CONFIG`, else the default path. Only the
    /// default path may be missing.
    pub fn load(explicit: Option<&Path>) -> Result<Config> {
        let named = explicit
            .map(Path::to_path_buf)
            .or_else(|| env::var_os("GOGO_CONFIG").filter(|v| !v.is_empty()).map(PathBuf::from));
        let (path, required) = match named {
            Some(path) => (path, true),
            None => match paths::config_dir() {
                Some(dir) => (dir.join("config.toml"), false),
                None => return Ok(Config::default()),
            },
        };
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)
                .map_err(|e| Error::failed(format!("{}: {}", path.display(), e.message()))),
            Err(e) if e.kind() == io::ErrorKind::NotFound && !required => Ok(Config::default()),
            Err(e) => Err(Error::failed(format!("{}: {e}", path.display()))),
        }
    }

    pub fn backend(&self, name: &str) -> Option<&BackendConfig> {
        match name {
            "herdr" => self.herdr.as_ref(),
            "tmux" => self.tmux.as_ref(),
            _ => None,
        }
    }

    pub fn attach(&self, backend: &str) -> bool {
        self.backend(backend).and_then(|b| b.attach).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_documented_key() {
        let config: Config = toml::from_str(
            r#"
            default_backend = "herdr"
            spaces_path = ["~/dotfiles/gogo/spaces"]
            editor = "fresh"

            [herdr]
            attach = false

            [tmux]
            attach = true
            send_prompts = false
            "#,
        )
        .unwrap();
        assert_eq!(config.default_backend.as_deref(), Some("herdr"));
        assert!(!config.attach("herdr"));
        assert!(config.attach("tmux"));
        assert!(config.attach("other"));
    }

    #[test]
    fn rejects_unknown_keys() {
        assert!(toml::from_str::<Config>("defualt_backend = \"herdr\"").is_err());
    }

    #[test]
    fn missing_explicit_file_is_an_error() {
        let err = Config::load(Some(Path::new("/surely/missing/gogo.toml"))).unwrap_err();
        assert_eq!(err.kind, crate::error::Kind::Failed);
    }
}
