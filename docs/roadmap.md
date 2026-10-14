# Roadmap

Each version is usable on its own. Don't start the next until the current one is the thing I actually use.

## v0.1: replaces the prototype

- TOML spaces: `dir`, `env`, tabs, single-pane tabs, `run`.
- Space lookup: project `.gogo.toml`, user dir, `GOGO_PATH`.
- `gogo <backend> <space>`, `open`, `list`, `plan` / `--dry-run`, `check`, `version`, `help`.
- herdr backend. Open or focus. Attach when outside. Starts the herdr server if it isn't running.
- `gogo herdr stop`: close every workspace and stop every herdr session. `gogo herdr stop <space>` closes one space (added after 0.1.1).
- Exit codes and `--json` as in `cli.md`, from day one. Changing them later breaks scripts.
- Golden plan tests.
- Install via Homebrew (`conorbr/tap`), a `curl | sh` script, prebuilt binaries on each release, or `cargo install gogo-spaces`.
- **Done when:** `gogo herdr blog` replaces the dotfiles script and nothing feels worse.
- **Then:** repo goes public with `LICENSE-MIT` and `LICENSE-APACHE`, `gogo-spaces` is published to crates.io, launch post on breen.ie.

## v0.2: splits and save

- Splits and sizes. Pane `label`, shown on the pane border.
- `save` for herdr: tabs, splits, pane dirs, and commands from a running workspace into a space file. Needs splits first.
- **Done when:** `gogo save` on a workspace gogo opened writes back the same file, and a hand-built workspace saves and reopens as it was.

## v0.3: managing spaces

- `--fresh`, `show`, `new`, `edit`, `doctor`, `backends`. (`close` became `stop <space>`.)
- Shell completions, including space names.
- Maybe: the `gogo <space>` shorthand, if v0.1 use calls for it.
- Fallback space: a `default` space that opens in any directory without a `.gogo.toml` (`gogo herdr .`). A project's own space always wins.

## v0.4: agents

- `agent`, `agent_name`, `agent_args`, `prompt`.
- `ready` waits and timeouts.
- `before` hooks.
- `vars` and `--set`.
- Worktrees: `gogo herdr blog --worktree <branch>` creates a git worktree (`herdr worktree create`) and opens the space in it. One space, many branches, an agent on each.
- Branch matching: a space can name a `branch`, and the most specific match wins (repo and branch, then repo, then the fallback).

## v0.5: tmux

- tmux backend with fallbacks.
- Backend contract tests, run against both.
- `save` for tmux (best effort).

## v1.0

- `extends`.
- Docs site or a thorough README.
- Stable config format and JSON output, versioned.
- herdr plugin: a keybinding opens a fuzzy picker of spaces in a popup and runs `gogo herdr <space>`. Ships beside gogo, not inside it.

## Later

- External backends (`gogo-backend-<name>`, plan as JSON).
- Zellij, WezTerm, or kitty backends if anyone wants them.
- herdr remote machines (`--machine`).
