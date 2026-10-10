# Roadmap

Each version is usable on its own. Don't start the next until the current one is the thing I actually use.

## v0.1: replaces the prototype

- TOML spaces: `dir`, `env`, tabs, single-pane tabs, `run`.
- Space lookup: project `.gogo.toml`, user dir, `GOGO_PATH`.
- `gogo <backend> <space>`, `open`, `list`, `plan` / `--dry-run`, `check`, `version`, `help`.
- herdr backend. Open or focus. Attach when outside.
- Exit codes and `--json` as in `cli.md`, from day one. Changing them later breaks scripts.
- Golden plan tests.
- **Done when:** `gogo herdr blog` replaces the dotfiles script and nothing feels worse.

## v0.2: shaping spaces

- Splits and sizes.
- `close`, `--fresh`, `show`, `new`, `edit`, `doctor`, `backends`.
- Shell completions, including space names.
- `save` for herdr.

## v0.3: agents

- `agent`, `agent_name`, `agent_args`, `prompt`.
- `ready` waits and timeouts.
- `before` hooks.
- `vars` and `--set`.

## v0.4: tmux

- tmux backend with fallbacks.
- Backend contract tests, run against both.
- `save` for tmux (best effort).

## v1.0

- `extends`.
- Docs site or a thorough README. Install via Homebrew and a script.
- Stable config format and JSON output, versioned.

## Later

- External backends (`gogo-backend-<name>`, plan as JSON).
- Zellij, WezTerm, or kitty backends if anyone wants them.
- herdr remote machines (`--machine`).
