# tmux backend

Second backend. Proves the interface: if tmux needs a change outside its own backend, the interface is wrong.

## Concepts

| gogo | tmux |
|---|---|
| Space (workspace) | Session, named after the space |
| Tab | Window |
| Pane | Pane |
| Agent | No native concept. Falls back to running the agent's command |

Target panes by stable IDs (`%3`), not indexes, since `base-index` and `pane-base-index` vary by config. Use `-P -F '#{pane_id}'` to read IDs back on create.

## Step mapping

| Step | tmux |
|---|---|
| `find_workspace` | `tmux has-session -t =<name>` (`=` for an exact match) |
| `focus` (workspace) | inside: `tmux switch-client -t =<name>`. Outside: attach |
| `create_workspace` | `tmux new-session -d -s <name> -c <dir> -n <first tab> -P -F '#{pane_id}'` |
| `create_tab` | `tmux new-window -t =<name> -c <dir> -n <tab> -P -F '#{pane_id}'` |
| `split` | `tmux split-window -t <pane> -h\|-v -c <dir> [-l <size>] -P -F '#{pane_id}'`. `right` is `-h`, `down` is `-v` |
| `run` | `tmux send-keys -t <pane> -l <command>` then `send-keys -t <pane> Enter` |
| `env` | `new-session`/`new-window -e K=V` |
| `wait_output` | Poll `tmux capture-pane -p -t <pane>` for the text, with a timeout |
| `start_agent` | Fallback: `run` the agent command |
| `prompt_agent` | Fallback: after `ready`, `send-keys -l <text>` then `Enter`. Warns: no guarantee the agent is ready |
| `focus` (tab) | `tmux select-window -t <window>` |
| `close` | `tmux kill-session -t =<name>` |
| `inside()` | `TMUX` is set |
| `attach` | `exec tmux attach-session -t =<name>` |

## Capture (`save`)

`tmux list-windows` and `list-panes -F` give names, dirs (`#{pane_current_path}`), current commands (`#{pane_current_command}`, just the process name), and layout (`#{window_layout}`). The full command line needs `ps` on `#{pane_pid}`. Splits are recovered from the layout string, roughly. Expect `save` from tmux to need hand edits; say so in its output.

## Open questions

- Minimum tmux version. `new-window -e` needs 3.0 or later.
- Respect the user's `default-command` and shell, or force a shell?
- Session names can't contain `.` or `:`. Sanitise space names, or reject them in `check`?
