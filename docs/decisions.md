# Decisions

Newest first. Each one says what, why, and what it rules out.

## Made

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

- **Name.** "gogo" collides with other projects. Check GitHub, Homebrew, and package registries before publishing.
- **`gogo <space>` with no backend.** Allow it with a default backend, or always require one? Allowing it adds one more rule to resolving the first word.
- **Agents on tmux.** Is running the command and sending keys good enough, or should agent spaces refuse to open in tmux?
- **Public or private** until v0.1 works.
- **Licence.** MIT unless there's a reason otherwise.
