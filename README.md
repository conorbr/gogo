# gogo

Open a configured workspace in your terminal multiplexer with one command.

```bash
gogo herdr blog
```

Opens the `blog` space in herdr: the right directory, the right tabs, and the right commands in them. If it's already open, gogo focuses it instead.

**Status:** v0.1. herdr only for now; tmux is planned. Design and roadmap in [`docs/`](docs/README.md).

## Install

With Rust 1.85 or later:

```bash
cargo install gogo-spaces
```

The crate is `gogo-spaces` because `gogo` was taken. The command it installs is `gogo`.

From a checkout: `cargo install --path .`

## A space

`~/.config/gogo/spaces/blog.toml`:

```toml
dir = "~/Developer/breen"

[[tab]]
name = "Claude"
run = "claude"

[[tab]]
name = "Editor"
run = "fresh ."

[[tab]]
name = "Server"
run = "npm run dev"
```

A `.gogo.toml` in a project works too, and `GOGO_PATH` adds more folders. Full format: [`docs/config.md`](docs/config.md).

## Use

```bash
gogo herdr blog            # open or focus
gogo herdr blog --dry-run  # show the plan, change nothing
gogo herdr                 # list spaces, * marks open ones
gogo check                 # validate every space
gogo --json herdr blog     # for scripts
```

Everything else: [`docs/cli.md`](docs/cli.md).

## Develop

```bash
cargo test                         # unit tests and golden plans
UPDATE_GOLDEN=1 cargo test         # accept plan changes in tests/golden/
cargo clippy --all-targets
```

## Licence

MIT or Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
