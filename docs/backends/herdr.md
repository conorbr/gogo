# herdr backend

First backend. Built on the herdr CLI, which returns JSON (`.result.*`). Tested against herdr 0.9.3. The socket API (`herdr api`) is an option later if shelling out gets slow.

## Concepts

| gogo | herdr |
|---|---|
| Space (workspace) | Workspace, matched by `label` |
| Tab | Tab |
| Pane | Pane |
| Agent | A recognised agent in a pane (`herdr agent ...`) |

IDs (`w1`, `w1:t2`, `w1:p3`) are opaque and never reused. Read them from each response; never predict them.

## Step mapping

| Step | herdr |
|---|---|
| `find_workspace` | `herdr workspace list`, match `.label` |
| `focus` (workspace) | `herdr workspace focus <id>` |
| `create_workspace` | `herdr workspace create --cwd <dir> --label <name> [--focus\|--no-focus] [--env K=V]`. Returns `.result.workspace`, `.result.tab`, `.result.root_pane` |
| `create_tab` (first) | Use the workspace's root tab. `herdr tab rename <tab> <name>` |
| `create_tab` | `herdr tab create --workspace <id> --cwd <dir> --label <name> --no-focus [--env K=V]`. Returns `.result.root_pane` |
| `split` | `herdr pane split <pane> --direction right\|down --cwd <dir> --no-focus`. Returns `.result.pane` |
| `run` | `herdr pane run <pane> <command>` |
| `wait_output` | `herdr pane wait-output ...` (check flags) |
| `start_agent` | `herdr agent start <name> --kind <kind> --pane <pane> [-- args]`. Returns once the agent is ready, or `agent_not_ready` if it's blocked at startup |
| `prompt_agent` | `herdr agent prompt <name> <text> --wait --timeout <ms>` |
| `focus` (tab) | check `herdr tab` subcommands |
| `close` | `herdr workspace close <id>` |
| `inside()` | `HERDR_ENV=1` |
| `attach` | `exec herdr` (attaches to the running session) |

## Capture (`save`)

1. `herdr tab list --workspace <id>`: tab labels, order, focus.
2. `herdr pane list --workspace <id>`: panes per tab, `cwd`, recognised `agent`.
3. `herdr pane layout --pane <id>`: split directions and sizes.
4. `herdr pane process-info --pane <id>`: the foreground command (`.foreground_processes[].cmdline`).

Rules:
- A pane with a recognised `agent` becomes `agent = "<kind>"`, not `run`.
- The foreground command becomes `run`. Prefer the command the user typed (`npm run dev`) over what it became (`node .../nuxt dev`): take the process group leader, or the shallowest process.
- Helper processes the user didn't start (`caffeinate`, `esbuild`) are dropped.
- A `cwd` equal to the space `dir` is omitted.

## Open questions

- Does `pane split` take a size?
- Exact `pane wait-output` flags, and whether it sees output from before the call.
- Which `tab` subcommand focuses a tab.
- `agent start` needs the pane at an idle shell prompt. Does a freshly created pane need a wait first?
- Remote machines: herdr's `--machine` passes through cleanly. Expose as `--machine` on `open`?
