# Work log

## 2026-10-10

- Started the repo as a design space. No code.
- Prototype first, in `~/dotfiles`: `gogo herdr blog` builds a three-tab herdr workspace (Claude, Editor, Server) from a line-format space file, or focuses it if open. Tested by building throwaway workspaces and closing them.
- Wrote the brief, CLI surface, space format, architecture (config to plan to backend), herdr and tmux mappings, roadmap, and decisions.
- Key call: spaces belong to no backend. The backend is chosen at open time.
- Language: Rust.
