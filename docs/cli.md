# CLI

## Grammar

```
gogo <backend> <space> [flags]    open a space in a backend (the everyday form)
gogo <command> [args] [flags]     everything else
gogo <backend>                    list spaces, marking which are open in that backend
gogo                              list spaces
```

`gogo herdr blog` is shorthand for `gogo open blog --backend herdr`. Both are supported. The shorthand is the one people type; the long form is the one scripts and docs can rely on.

### Resolving the first word

1. A **command** name (`open`, `list`, ...) runs that command.
2. A **backend** name (`herdr`, `tmux`) means the shorthand.
3. Anything else is an error, with a suggestion if it's close to a space, command, or backend.

Commands and backends share one namespace, so a backend can never be called `list` or `open`. Space names can be anything, since they're never first.

## Commands

| Command | What it does |
|---|---|
| `open <space>` | Open the space, or focus it if open. Attach if run outside the backend |
| `close <space>` | Close the space's workspace in the backend |
| `list` | List spaces and where each is defined. `--open` for only open ones |
| `show <space>` | Print the resolved space (after defaults and `extends`) |
| `plan <space>` | Print the plan without running it. Same as `open --dry-run` |
| `new <space>` | Scaffold a space for the current directory |
| `edit <space>` | Open the space file in `$EDITOR` |
| `save <space>` | Capture a running workspace into a space file |
| `check [space]` | Validate space files. Every space if none is named |
| `doctor` | Check backends are installed and reachable, config is valid, dirs exist |
| `backends` | List backends, their versions, and what each supports |
| `completion <shell>` | Print shell completions (bash, zsh, fish) |
| `version` | Print the version |
| `help [command]` | Help |

## Flags

Global:

| Flag | Meaning |
|---|---|
| `-b, --backend <name>` | Backend to use. Default from config, else detected (see below) |
| `--json` | Machine-readable output on stdout |
| `-q, --quiet` | Errors only |
| `-v, --verbose` | Show each backend call |
| `--config <path>` | Use this config file |
| `--no-color` | Plain output. Also respects `NO_COLOR` |

`open`:

| Flag | Meaning |
|---|---|
| `-n, --dry-run` | Print the plan, change nothing |
| `--fresh` | Close the workspace if open, then open it again |
| `--no-attach` | Build it but don't attach or focus |
| `--no-focus` | Build it but leave focus where it is |
| `--set key=value` | Set a space variable for this run |

`save`:

| Flag | Meaning |
|---|---|
| `--from <workspace>` | Which running workspace to capture. Default: the current one |
| `--to <path>` | Where to write it. Default: user spaces dir |
| `--force` | Overwrite an existing space |

`new`:

| Flag | Meaning |
|---|---|
| `--project` | Write `.gogo.toml` in the current directory instead of the user spaces dir |
| `--from <space>` | Start from an existing space |

## Default backend

When `--backend` is missing from `gogo open blog`:

1. `--backend` flag
2. `GOGO_BACKEND`
3. `default_backend` in config
4. Detected: inside herdr (`HERDR_ENV=1`) means herdr, inside tmux (`TMUX` set) means tmux
5. Error: name a backend

## Inside or outside

- **Inside the target backend:** build or focus, then return. The terminal stays where it is.
- **Outside it:** build or focus, then attach (replace the process with the backend's client). `--no-attach` to skip.

## Output

- Human output on stderr for progress, stdout for results. Results are short: what was opened, where.
- `--json` puts one JSON object on stdout per command, and nothing else. Stable keys, versioned (`"gogo": 1`).
- Never prompts when stdin is not a TTY. Commands that would prompt (`save` over an existing file) fail instead and name the flag.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Done |
| 1 | Failed (a backend call failed, a dir is missing) |
| 2 | Usage error (bad flag, unknown command) |
| 3 | Not found (no such space or backend) |
| 4 | Backend unavailable (not installed, server not running) |
| 5 | Invalid space file |

## Environment

| Variable | Meaning |
|---|---|
| `GOGO_BACKEND` | Default backend |
| `GOGO_CONFIG` | Config file path |
| `GOGO_PATH` | Extra space directories, colon-separated (e.g. a dotfiles repo) |
| `NO_COLOR` | Plain output |

## Examples

```bash
gogo herdr blog                 # open or focus the blog space in herdr
gogo tmux blog                  # same space, in tmux
gogo herdr blog --dry-run       # what would it do
gogo herdr blog --fresh         # start over
gogo save blog                  # capture the current herdr workspace as "blog"
gogo list --open --json         # for a status bar
gogo check                      # validate every space, e.g. in a dotfiles CI
```
