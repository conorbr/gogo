# gogo

Open a configured workspace in your terminal multiplexer with one command.

```bash
gogo herdr blog
```

Opens the `blog` space in herdr: the right directory, the right tabs, and the right commands in them. If it's already open, gogo focuses it instead.

**Status:** v0.1. herdr only for now; tmux is planned. Design and roadmap in [`docs/`](docs/README.md).

## Install

Prebuilt binaries for macOS (Apple Silicon and Intel) and Linux (x86_64 and arm64). No Rust needed.

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/conorbr/gogo/main/scripts/install.sh | sh
```

It detects your OS and CPU, downloads the latest release, verifies its checksum, and puts `gogo` in `~/.local/bin`. No root. [Read the script](scripts/install.sh) first if you like; the header says exactly what it does. `GOGO_VERSION=0.1.0` pins a version and `GOGO_INSTALL_DIR` changes where it goes.

### Homebrew

```bash
brew install conorbr/tap/gogo
```

Works on macOS and Linux.

### With Rust

Any platform with Rust 1.85 or later:

```bash
cargo install gogo-spaces
```

The crate is `gogo-spaces` because `gogo` was taken on crates.io. The command it installs is `gogo`. Latest from GitHub: `cargo install --git https://github.com/conorbr/gogo`. From a checkout: `cargo install --path .`

### By hand

Download the archive for your platform from [Releases](https://github.com/conorbr/gogo/releases), check it against the `.sha256` beside it (`shasum -a 256 -c <file>.sha256`), and put `gogo` somewhere on your `PATH`. If you downloaded it in a browser on macOS, clear the quarantine flag first: `xattr -d com.apple.quarantine gogo`. Linux builds are statically linked and run on any distribution.

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
gogo herdr stop            # close every workspace, stop every herdr session
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

## Release

1. Bump `version` in `Cargo.toml`. Commit.
2. Tag and push: `git tag -a vX.Y.Z -m "gogo X.Y.Z" && git push origin main vX.Y.Z`. The Release workflow builds the binaries and attaches them to the GitHub Release.
3. `cargo publish`.
4. Update the formula in [conorbr/homebrew-tap](https://github.com/conorbr/homebrew-tap) (steps in its README).

## Licence

MIT or Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
