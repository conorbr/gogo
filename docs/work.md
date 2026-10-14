# Work log

## 2026-10-14: `gogo herdr stop <space>`

- `gogo stop <space>` (`gogo <backend> stop <space>`, `-n` to check only): closes that space's workspace in the current session and leaves herdr running. Not open, or herdr not running: says so, exit 0. Unknown space: exit 3, same message as `open`. The space file only has to exist, not be valid. `--json`: `space`, `backend`, `dry_run`, `closed`.
- Backend trait: `close(space, dry_run)`. herdr: `find` by label, then `workspace close`; if gogo runs inside that workspace, the close waits for `finish_stop` and runs in a `setsid` child, like `stop`'s own workspace.
- Tests: 42 unit tests (shorthand and parse for `stop <space>`, the fake backend's close). Clippy clean. The repo isn't rustfmt-clean at default settings (87 diffs before this change), so new code follows the surrounding style.
- Tested against real herdr 0.9.3 with a throwaway space on `GOGO_PATH`: open, `stop -n`, `--json -n`, `stop`, `stop` again (not open), unknown space (exit 3), and run from a pane inside the workspace being closed (prints, then the workspace closes). Other workspaces untouched.

## 2026-10-10: `gogo herdr stop`

- `gogo stop` (`gogo <backend> stop`, `-n` to list only): closes every workspace in every running herdr session, then stops each session. `--json` lists sessions and the workspaces closed.
- Workspaces close first because herdr restores them after a restart; checked by stopping and restarting a test session.
- Inside herdr, gogo's own workspace and session go last, in a `setsid` child, after the output prints. A process group alone was killed with the workspace; checked both.
- Tested against throwaway `herdr --session gogotest server` sessions, with a `herdr` wrapper hiding the real session: dry run, `--json -v`, nothing running, and run from a pane inside the session being stopped. New dependency: `libc`, for `setsid`.
- `open` now starts the herdr server if it isn't running (after a `stop`, say): `herdr server` in its own Unix session, then waits up to 5s for `herdr status server` to say running. `list` still only checks, so it never starts a server. Tested from a stopped server: opened `college`, server left running with PPID 1. herdr also creates its default `~` workspace on start.

## 2026-10-10: going public

- Added `LICENSE-MIT` and `LICENSE-APACHE` (Apache text from apache.org). Crate metadata: repository, keywords, categories; `publish = false` removed.
- README: install from GitHub with `cargo install --git`, licence section.
- Public on GitHub at github.com/conorbr/gogo. Published `gogo-spaces` 0.1.0 to crates.io the same day. README installs with `cargo install gogo-spaces`.
- Homebrew tap at github.com/conorbr/homebrew-tap: `brew install conorbr/tap/gogo`. First version built from source and pulled in Homebrew's rust (and llvm), which is too much to ask of users.
- Release workflow (`.github/workflows/release.yml`): on a `v*` tag, or by hand for an existing tag, builds macOS arm64/x86_64 and static Linux musl arm64/x86_64 tarballs with `.sha256` files and attaches them to the GitHub Release. Ran it for v0.1.0: all four built, checksums verified, both macOS binaries run here, Linux ones are static.
- The tap formula now installs those binaries: no dependencies, installs in about a second. Style, strict audit, install, and `brew test` pass.
- README install split into macOS, Linux, and With Rust: Homebrew or a direct download (arch picked by `uname -m`) for each OS, plus checksums. Tested the macOS snippet into a scratch home; Linux URLs resolve for x86_64, aarch64, and arm64. Release checklist added to the README.
- Replaced the per-OS download snippets with `scripts/install.sh` (`curl ... | sh`): detects OS and CPU, finds the latest release via the /releases/latest redirect (no API), verifies the .sha256, writes one file to `~/.local/bin`. `GOGO_VERSION`, `GOGO_INSTALL_DIR`. Tested under dash and macOS sh: latest, pinned, PATH notes, missing version, corrupted download (aborts, nothing written). `.github/workflows/install.yml` runs it on Ubuntu x86_64 and arm64, Alpine (busybox sh), and macOS. Release steps in the tap's README. homebrew-core later, once the repo is old and notable enough for `brew audit --new`.

## 2026-10-10: v0.1 build

- Rust crate `gogo-spaces`, binary `gogo`. `publish = false` until the repo goes public.
- Built: TOML spaces (`dir`, `env`, tabs, `run`, `focus`), lookup (project `.gogo.toml`, user dir, `GOGO_PATH`, `spaces_path`), global config, `gogo <backend> <space>` shorthand, `open`, `plan`/`--dry-run`, `list` (with `*` for open), `check`, `version`, `--json`, exit codes, `-v` backend calls, herdr backend.
- Layout: `space` (parse, validate, resolve) → `plan` (steps, plain data) → `backend::execute` → `backend::herdr`. The executor stops before `attach` so results print first.
- Keys from later versions (`agent`, `prompt`, `split`, ...) fail with "planned for vX", not just "unknown field".
- Tests: 39 unit tests, golden plans in `tests/golden/` (inside and outside, text and JSON), a fake backend for the executor. Clippy clean.
- Tested against real herdr 0.9.3 inside herdr: create (env merge, per-tab dirs, run, focus tab), re-run focuses, `--open` marks it, `--json` output. Errors and exit codes checked by hand.
- Attaching from outside herdr: tested by hand in a real terminal, works.
- **Not yet tested:** the server-not-running error.
- **Gap against `cli.md`:** an unknown first word gets clap's suggestion only, not "did you mean `gogo herdr <space>`".
- **v0.1 "done when" met:** installed to `~/.local/bin` (`cargo install --root ~/.local`), the blog space is TOML in `~/dotfiles/gogo/spaces` (linked to `~/.config/gogo/spaces`), and the bash prototype is gone. `gogo herdr blog` runs the Rust build.

## 2026-10-10

- Started the repo as a design space. No code.
- Prototype first, in `~/dotfiles`: `gogo herdr blog` builds a three-tab herdr workspace (Claude, Editor, Server) from a line-format space file, or focuses it if open. Tested by building throwaway workspaces and closing them.
- Wrote the brief, CLI surface, space format, architecture (config to plan to backend), herdr and tmux mappings, roadmap, and decisions.
- Key call: spaces belong to no backend. The backend is chosen at open time.
- Language: Rust.
- Decided the rest: command `gogo`, crate `gogo-spaces`; backend required in v0.1; agents on tmux run without prompts (opt-in `send_prompts`); private until v0.1; MIT OR Apache-2.0.
