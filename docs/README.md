# gogo docs

Design space. Read in this order.

| File | What |
|---|---|
| [brief.md](brief.md) | What gogo is, who it's for, goals, non-goals, prior art |
| [cli.md](cli.md) | The command surface: grammar, commands, flags, output, exit codes |
| [config.md](config.md) | The space file format and where spaces live |
| [architecture.md](architecture.md) | Config to plan to backend. How a new backend plugs in |
| [backends/herdr.md](backends/herdr.md) | How a plan maps onto herdr |
| [backends/tmux.md](backends/tmux.md) | How a plan maps onto tmux |
| [roadmap.md](roadmap.md) | Versions, in order |
| [decisions.md](decisions.md) | Decisions made, and the ones still open |
| [work.md](work.md) | Running log |

## Words

- **Space:** a named description of a workspace. Directory, tabs, panes, what runs in them. Belongs to no backend.
- **Backend:** a terminal multiplexer gogo can drive. herdr first, tmux next.
- **Plan:** the list of steps gogo will take to open a space in a backend. Plain data. What `--dry-run` prints.
- **Tab:** a herdr tab, a tmux window.
- **Pane:** a split inside a tab.
