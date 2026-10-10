# Config

## Space files

TOML. One space per file. The file name is the space name unless `name` says otherwise.

```toml
# ~/.config/gogo/spaces/blog.toml
dir = "~/Developer/breen"

[[tab]]
name = "Claude"
agent = "claude"
prompt = "Read docs/work.md and tell me where we left off."
focus = true

[[tab]]
name = "Editor"
run = "fresh ."

[[tab]]
name = "Server"
run = "npm run dev"
ready = "Local:"
```

### Space keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | file name | Space name. Also the workspace label (herdr) or session name (tmux) |
| `dir` | path | current dir | Where every tab starts. `~` and env vars expand |
| `env` | table | none | Environment for every pane |
| `extends` | string | none | Another space to start from. Its tabs come first, then these |
| `before` | string or list | none | Commands run once, in `dir`, before anything opens. A failure stops the open |
| `vars` | table | none | Variables for `${var}` in strings. Overridable with `--set` |
| `[herdr]`, `[tmux]` | table | none | Backend-only options. Ignored by other backends |

### Tab keys

A tab is one pane, or several. A tab with `run` or `agent` and no `[[tab.pane]]` is a single pane.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | required | Tab label |
| `dir` | path | space `dir` | Override for this tab |
| `env` | table | none | Merged over the space's |
| `focus` | bool | first tab | The tab you land on. At most one |
| `run`, `agent`, `prompt`, `ready` | | | As pane keys, for a single-pane tab |

### Pane keys

```toml
[[tab]]
name = "Dev"

  [[tab.pane]]
  run = "npm run dev"
  ready = "Local:"

  [[tab.pane]]
  split = "right"
  size = "40%"
  run = "npm test -- --watch"
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `run` | string | none | Command typed into the pane's shell. The shell stays when it exits |
| `agent` | string | none | A coding agent kind (`claude`, `codex`, ...). Started as an agent, not a plain command, where the backend supports it |
| `agent_name` | string | `<space>-<tab>` | Name to register the agent under |
| `agent_args` | list | none | Extra arguments for the agent |
| `prompt` | string | none | Sent to the agent once it's ready. Needs `agent` |
| `ready` | string | none | Wait for this text in the pane's output before opening the next pane or tab |
| `ready_timeout` | duration | `30s` | Give up waiting after this |
| `split` | `right` or `down` | first pane: none | Split from the previous pane in this tab |
| `size` | percent | even | Size of the new pane |
| `label` | string | none | Name shown on the pane border, where the backend has one |
| `dir`, `env` | | | As for tabs |
| `focus` | bool | first pane | The pane focused within the tab |

`run` and `agent` are exclusive. A pane with neither is a plain shell.

### Validation

`gogo check` fails on: unknown keys, `run` and `agent` together, `prompt` without `agent`, two focused tabs, `split` on a first pane, a `dir` that doesn't exist, an `extends` cycle. Unknown keys are errors, not warnings, so typos don't silently do nothing.

## Where spaces live

Searched in this order. The first match by name wins.

1. **Project:** `.gogo.toml` in the current directory or any parent up to `$HOME`. One space. Name from `name`, else the directory name.
2. **User:** `~/.config/gogo/spaces/*.toml` (respects `XDG_CONFIG_HOME`).
3. **Extra:** each directory in `GOGO_PATH`, then `spaces_path` in config. Good for a dotfiles repo.

`gogo list` shows where each space came from. `gogo check` warns when one name is defined in more than one place.

## Global config

`~/.config/gogo/config.toml`. Every key is optional.

```toml
default_backend = "herdr"
spaces_path = ["~/dotfiles/gogo/spaces"]
editor = "fresh"

[herdr]
attach = true

[tmux]
attach = true
send_prompts = false   # true: type agent prompts after `ready`, best effort
```

## From the dotfiles prototype

The prototype's line format maps directly:

| Prototype | TOML |
|---|---|
| `dir ~/Developer/breen` | `dir = "~/Developer/breen"` |
| `tab Claude claude` | `[[tab]]` `name = "Claude"` `agent = "claude"` |
| `tab Editor fresh .` | `[[tab]]` `name = "Editor"` `run = "fresh ."` |
| `tab Server npm run dev` | `[[tab]]` `name = "Server"` `run = "npm run dev"` |

`gogo save blog` run inside the current workspace should produce the same file.
