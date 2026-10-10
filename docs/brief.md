# Brief

## Problem

Every project has a working setup: an agent in one tab, an editor in another, a dev server in a third. Rebuilding it by hand every morning is slow and slightly different each time. Shell scripts that rebuild it work for one project and one multiplexer, then get copied and drift.

## What gogo is

A small CLI that opens a **space** (a named, declarative workspace) in a **backend** (a terminal multiplexer).

```bash
gogo herdr blog
```

One space file. Any supported backend. Open it, or focus it if it's already open.

## Who it's for

Developers who live in the terminal and work with coding agents. First user: me, with herdr. Second: anyone with a tmux habit.

## Goals

1. **One command to the working state.** Directory, tabs, panes, processes, agents.
2. **Spaces are backend-agnostic.** Write it once, open it in herdr or tmux.
3. **Agents are first-class.** Start a named agent, wait until it's ready, give it an opening prompt. Where a backend can't do that natively, degrade to running the command.
4. **Safe to re-run.** Open or focus, never duplicate.
5. **Predictable.** `--dry-run` shows exactly what will happen. `--json` for scripts.
6. **Easy to extend.** A new backend implements one small interface. Nothing else changes.

## Non-goals

- **Not a multiplexer.** gogo drives one; it never draws a terminal.
- **Not a process manager.** No restarts, health checks, or logs. Use the backend or a real supervisor.
- **No live sync.** v1 opens or focuses. It does not diff a running workspace against the file and patch it.
- **No GUI.**

## Prior art

| Tool | Backend | Notes |
|---|---|---|
| tmuxinator | tmux | YAML projects. Ruby. The reference for this space |
| tmuxp | tmux | YAML/JSON sessions, can freeze a session to a file. Python |
| Zellij layouts | Zellij | Built in, KDL. One backend only |
| Dotfiles scripts | any | What everyone actually does. Drift |

**What's different:** backend-agnostic spaces, agents as a first-class pane type, and `save` that captures a running workspace into a file.

## Risks

- **herdr adds native layouts.** Plausible. gogo still earns its place through backend-agnostic spaces, agent startup, and tmux support. Check herdr's roadmap before v0.2.
- **Backend CLIs change.** herdr is young. Keep each backend thin and pin tested versions.
- **The name.** "gogo" is used elsewhere (an old Go web framework, others). Check GitHub, Homebrew, and package registries before publishing.
