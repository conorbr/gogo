use std::env;
use std::path::{Path, PathBuf};

pub fn home() -> Option<PathBuf> {
    env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from)
}

/// `$XDG_CONFIG_HOME/gogo`, else `~/.config/gogo`. Deliberately not the
/// platform config dir: macOS users expect `~/.config` for CLI tools.
pub fn config_dir() -> Option<PathBuf> {
    match env::var_os("XDG_CONFIG_HOME").filter(|x| !x.is_empty()) {
        Some(xdg) => Some(PathBuf::from(xdg).join("gogo")),
        None => home().map(|h| h.join(".config").join("gogo")),
    }
}

/// Expands `~`, `$VAR`, and `${VAR}`, then resolves a relative result
/// against `base`. Unset variables are an error, not an empty string.
pub fn expand(raw: &str, base: &Path) -> Result<PathBuf, String> {
    let text = expand_vars(raw)?;
    let path = if text == "~" {
        home().ok_or("~ used but HOME is not set")?
    } else if let Some(rest) = text.strip_prefix("~/") {
        home().ok_or("~ used but HOME is not set")?.join(rest)
    } else {
        PathBuf::from(text)
    };
    Ok(if path.is_absolute() { path } else { base.join(path) })
}

fn expand_vars(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        let name: String = if chars.peek() == Some(&'{') {
            chars.next();
            let name: String = chars.by_ref().take_while(|&c| c != '}').collect();
            if name.is_empty() {
                return Err(format!("empty ${{}} in {raw:?}"));
            }
            name
        } else {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                out.push('$');
                continue;
            }
            name
        };
        let value = env::var(&name).map_err(|_| format!("${name} is not set (in {raw:?})"))?;
        out.push_str(&value);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_paths_pass_through() {
        assert_eq!(expand("/srv/app", Path::new("/base")).unwrap(), PathBuf::from("/srv/app"));
    }

    #[test]
    fn relative_paths_resolve_against_base() {
        assert_eq!(expand("app", Path::new("/base")).unwrap(), PathBuf::from("/base/app"));
    }

    #[test]
    fn tilde_expands_to_home() {
        let home = home().unwrap();
        assert_eq!(expand("~/x", Path::new("/base")).unwrap(), home.join("x"));
        assert_eq!(expand("~", Path::new("/base")).unwrap(), home);
    }

    #[test]
    fn variables_expand() {
        let home = home().unwrap();
        let home = home.to_str().unwrap();
        assert_eq!(expand("$HOME/a", Path::new("/")).unwrap(), PathBuf::from(format!("{home}/a")));
        assert_eq!(expand("${HOME}a", Path::new("/")).unwrap(), PathBuf::from(format!("{home}a")));
    }

    #[test]
    fn unset_variables_are_errors() {
        let err = expand("$GOGO_TEST_SURELY_UNSET/x", Path::new("/")).unwrap_err();
        assert!(err.contains("GOGO_TEST_SURELY_UNSET"), "{err}");
    }

    #[test]
    fn lone_dollar_is_literal() {
        assert_eq!(expand("/a$", Path::new("/")).unwrap(), PathBuf::from("/a$"));
    }
}
