# Decisions

Newest first. Each one says what, why, and what it rules out.

## Made

### 2026-10-10: Name: command `gogo`, package `gogo-spaces`

`gogo` on crates.io is taken (a terminal URL opener), and `chainreactors/gogo` on GitHub ships a binary called `gogo`. Homebrew has no `gogo`. The command stays `gogo`; the crate is `gogo-spaces`, which installs a `gogo` binary. Homebrew formula `gogo` while it's free.
**Why:** no cost to muscle memory, and the clash is with two niche tools.
**Rules out:** `cargo install gogo`. Users with the other tools installed get whichever `gogo` is first on `PATH`.

### 2026-10-10: Backend required in v0.1

Only `gogo <backend> <space>` and `gogo open <space> [--backend]`. No `gogo <space>` shorthand yet.
**Why:** simpler first-word rules. Adding the shorthand later is safe; removing it would break habits.
**Rules out (for now):** nothing for users. Space names stay unrestricted until the shorthand exists.

### 2026-10-10: Agents on tmux run, prompts skipped

An `agent` pane on tmux runs the agent's command as a plain `run`. `prompt` is skipped with one warning. `[tmux] send_prompts = true` opts in to best-effort typed prompts after `ready`.
**Why:** a prompt typed into a pane that isn't ready can land anywhere. Skipping is predictable; the space still opens.
**Rules out:** refusing agent spaces on tmux, which would break "spaces belong to no backend".

### 2026-10-10: Private until v0.1

The repo stays private until `gogo herdr blog` runs on the Rust build. Then it goes public, the crate is published, and the launch gets a post.
**Why:** first impression is a working tool.
**Rules out:** building in public before v0.1. No crate published before there's code.

### 2026-10-10: MIT OR Apache-2.0

Dual licence, the Rust convention. `LICENSE-MIT` and `LICENSE-APACHE` added when the repo goes public.
**Why:** fits the ecosystem, and Apache-2.0's patent grant matters to some companies.

### 2026-10-10: Rust

**Why:** a single static binary with no runtime, strong typing for the config and plan models, and good crates for the job (`clap` for the CLI and completions, `serde` and `toml` for config, `serde_json` for plans and `--json`).
**Rules out:** Go, which was the other option.

### 2026-10-10: Spaces belong to no backend

A space describes what you want. The backend decides how. `gogo herdr blog` and `gogo tmux blog` open the same space.
**Why:** adding tmux, or anything else, then needs no change to anyone's spaces.
**Rules out:** backend-specific space files as the main format. Backend-only options live in `[herdr]` and `[tmux]` tables.

### 2026-10-10: Backend first on the command line

`gogo herdr blog`, as used daily in the prototype. Also available as `gogo open blog --backend herdr`.
**Why:** it reads as "go, herdr, blog" and matches muscle memory.
**Rules out:** backends and commands sharing a name.

### 2026-10-10: The plan is the contract

Config becomes a plan (plain data), and backends only ever see plans.
**Why:** `--dry-run` for free, golden tests without a terminal, and a language-agnostic protocol for external backends later.

### 2026-10-10: Open or focus, never patch

v1 doesn't diff a running workspace against its file. `--fresh` closes and reopens.
**Why:** reconciling live terminals is hard and rarely wanted. Re-running must never duplicate.

### 2026-10-10: TOML

**Why:** herdr's own config is TOML, it's readable, and tabs and panes map well onto arrays of tables.
**Rules out:** YAML (tmuxinator, tmuxp) and KDL (Zellij).

## Open

- **The `gogo <space>` shorthand,** after v0.1. If added: space names can't match a command or backend, and `gogo check` enforces it.
