# Work log

## 2026-10-10: going public

- Added `LICENSE-MIT` and `LICENSE-APACHE` (Apache text from apache.org). Crate metadata: repository, keywords, categories; `publish = false` removed.
- README: install from GitHub with `cargo install --git`, licence section.
- Public on GitHub at github.com/conorbr/gogo. Published `gogo-spaces` 0.1.0 to crates.io the same day. README installs with `cargo install gogo-spaces`.
- Homebrew tap at github.com/conorbr/homebrew-tap: `brew install conorbr/tap/gogo`. First version built from source and pulled in Homebrew's rust (and llvm), which is too much to ask of users.
- Release workflow (`.github/workflows/release.yml`): on a `v*` tag, or by hand for an existing tag, builds macOS arm64/x86_64 and static Linux musl arm64/x86_64 tarballs with `.sha256` files and attaches them to the GitHub Release. Ran it for v0.1.0: all four built, checksums verified, both macOS binaries run here, Linux ones are static.
- The tap formula now installs those binaries: no dependencies, installs in about a second. Style, strict audit, install, and `brew test` pass. Release steps in the tap's README. homebrew-core later, once the repo is old and notable enough for `brew audit --new`.

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
