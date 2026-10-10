# Architecture

```
space file ──load──▶ Space ──resolve──▶ Space (resolved) ──plan──▶ Plan ──run──▶ Backend
   TOML              parsed              extends, vars,            steps          herdr, tmux
                                         defaults, paths
```

Each arrow is a pure function except the last. Everything up to the plan is testable without a terminal.

## Layers

| Layer | Job | Knows about backends? |
|---|---|---|
| **cli** | Parse args, pick the command, format output, set exit codes | Only by name |
| **config** | Find, load, validate, and resolve space files and global config | No |
| **plan** | Turn a resolved space into ordered steps, given a backend's capabilities | Capabilities only |
| **backend** | Run steps against one multiplexer. Find existing workspaces. Capture for `save` | Yes, one each |

The core never imports a specific backend. Backends register themselves by name.

## The plan

A list of steps. Plain data, serialisable to JSON. `--dry-run` prints it; `--json` emits it.

```
find_workspace   name=blog                      → exists? focus and stop
create_workspace name=blog dir=~/Developer/breen
before           run="git pull"
create_tab       name=Claude                    (the workspace's first tab)
start_agent      kind=claude name=blog-claude
prompt_agent     name=blog-claude text="Read docs/work.md..."
create_tab       name=Editor
run              pane=Editor.0 cmd="fresh ."
create_tab       name=Server
run              pane=Server.0 cmd="npm run dev"
wait_output      pane=Server.0 text="Local:" timeout=30s
focus            tab=Claude
attach           (only outside the backend)
```

Steps refer to tabs and panes by **plan names** (`Server.0`), not backend IDs. The backend keeps the mapping from plan names to its real IDs as it goes. The plan stays stable and readable; IDs never leak into it.

## Backend interface

What a backend implements. Names are illustrative, not a language API.

| Method | Purpose |
|---|---|
| `name()` | `herdr`, `tmux` |
| `available()` | Installed, server reachable, version supported. Feeds `doctor` and exit code 4 |
| `capabilities()` | What it supports natively (below) |
| `find(space)` | Is a workspace for this space open? By label or session name |
| `apply(step)` | Run one step. Returns the IDs it created |
| `inside()` | Is gogo running inside this backend right now? |
| `attach(workspace)` | Hand the terminal to the backend's client |
| `capture(workspace)` | For `save`: tabs, panes, splits, commands, dirs, agents |
| `close(workspace)` | For `close` and `--fresh` |

## Capabilities

The planner asks before it plans. Missing native support means a fallback step, or an error if there isn't one.

| Capability | herdr | tmux | Fallback |
|---|---|---|---|
| Tabs | yes | yes (windows) | none needed |
| Splits | yes | yes | none needed |
| Split sizes | check | yes | even split, with a warning |
| Wait for output | yes (`pane wait-output`) | no | poll `capture-pane` |
| Native agents | yes (`agent start`) | no | `run` the agent's command |
| Agent prompt | yes (`agent prompt --wait`) | no | skipped with a warning. `[tmux] send_prompts = true`: `send-keys` after `ready` |
| Capture for `save` | yes (`pane process-info`) | partial | best effort, warn on gaps |

Fallbacks are visible: `--dry-run` shows the substituted steps, and `open` warns once.

## Adding a backend

1. Implement the interface.
2. Declare capabilities honestly.
3. Add `docs/backends/<name>.md` with the step mapping.
4. Add it to the shared contract tests (below).

No change to the CLI, config, or planner should be needed. If one is, the interface is wrong.

**External backends (later):** an executable `gogo-backend-<name>` on `PATH` that reads a plan as JSON on stdin and writes results as JSON on stdout. The plan is the contract, so any language works. The dotfiles prototype already uses this shape (`gogo/<tool>/run`). Built-in backends come first; this is the escape hatch.

## Testing

- **Config and plan:** table tests. A space file in, a plan out, compared against a golden file. Most of the logic lives here, and none of it needs a terminal.
- **Backend contract:** one test suite every backend passes, run against a real multiplexer in CI (tmux is easy; herdr where available). Open, re-open (focus, not duplicate), `--fresh`, close, capture round trip.
- **Fake backend:** records steps, for CLI tests.

## Idempotency

v1: open or focus, matched by name (herdr workspace label, tmux session name). It never edits a running workspace. `--fresh` closes and reopens. A diffing "apply" is a non-goal for now.
