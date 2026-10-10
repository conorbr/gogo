# Work log

## 2026-10-10: v0.1 build

- Rust crate `gogo-spaces`, binary `gogo`. `publish = false` until the repo goes public.
- Built: TOML spaces (`dir`, `env`, tabs, `run`, `focus`), lookup (project `.gogo.toml`, user dir, `GOGO_PATH`, `spaces_path`), global config, `gogo <backend> <space>` shorthand, `open`, `plan`/`--dry-run`, `list` (with `*` for open), `check`, `version`, `--json`, exit codes, `-v` backend calls, herdr backend.
- Layout: `space` (parse, validate, resolve) → `plan` (steps, plain data) → `backend::execute` → `backend::herdr`. The executor stops before `attach` so results print first.
- Keys from later versions (`agent`, `prompt`, `split`, ...) fail with "planned for vX", not just "unknown field".
- Tests: 39 unit tests, golden plans in `tests/golden/` (inside and outside, text and JSON), a fake backend for the executor. Clippy clean.
- Tested against real herdr 0.9.3 inside herdr: create (env merge, per-tab dirs, run, focus tab), re-run focuses, `--open` marks it, `--json` output. Errors and exit codes checked by hand.
- **Not yet tested:** attaching from outside herdr (needs a real terminal), and the server-not-running error.
- **Gap against `cli.md`:** an unknown first word gets clap's suggestion only, not "did you mean `gogo herdr <space>`".
- **Left for "done when":** install the binary, move the blog space to TOML, and switch dotfiles from the prototype.

## 2026-10-10

- Started the repo as a design space. No code.
- Prototype first, in `~/dotfiles`: `gogo herdr blog` builds a three-tab herdr workspace (Claude, Editor, Server) from a line-format space file, or focuses it if open. Tested by building throwaway workspaces and closing them.
- Wrote the brief, CLI surface, space format, architecture (config to plan to backend), herdr and tmux mappings, roadmap, and decisions.
- Key call: spaces belong to no backend. The backend is chosen at open time.
- Language: Rust.
- Decided the rest: command `gogo`, crate `gogo-spaces`; backend required in v0.1; agents on tmux run without prompts (opt-in `send_prompts`); private until v0.1; MIT OR Apache-2.0.
