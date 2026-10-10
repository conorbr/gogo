# gogo

Open a configured workspace in your terminal multiplexer with one command.

```bash
gogo herdr blog
```

Opens the `blog` space in herdr: the right directory, the right tabs, and the right commands in them. If it's already open, gogo focuses it instead.

**Status:** v0.1. herdr only for now; tmux is planned. Design and roadmap in [`docs/`](docs/README.md).

## Install

Prebuilt binaries for macOS and Linux. No Rust needed.

### macOS

Apple Silicon and Intel.

**Homebrew:**

```bash
brew install conorbr/tap/gogo
```

**Or download the binary** into `~/.local/bin`:

```bash
VERSION=0.1.0
TARGET="$(uname -m | sed s/arm64/aarch64/)-apple-darwin"
curl -fsSL "https://github.com/conorbr/gogo/releases/download/v$VERSION/gogo-v$VERSION-$TARGET.tar.gz" | tar -xz
mkdir -p ~/.local/bin && install -m 755 "gogo-v$VERSION-$TARGET/gogo" ~/.local/bin/gogo
gogo version
```

Make sure `~/.local/bin` is on your `PATH`. If you download the archive in a browser instead, macOS may block the binary as unverified. Clear that with `xattr -d com.apple.quarantine ~/.local/bin/gogo`.

### Linux

x86_64 and arm64. Statically linked, so it runs on any distribution.

**Homebrew on Linux:**

```bash
brew install conorbr/tap/gogo
```

**Or download the binary** into `~/.local/bin`:

```bash
VERSION=0.1.0
TARGET="$(uname -m | sed s/arm64/aarch64/)-unknown-linux-musl"
curl -fsSL "https://github.com/conorbr/gogo/releases/download/v$VERSION/gogo-v$VERSION-$TARGET.tar.gz" | tar -xz
mkdir -p ~/.local/bin && install -m 755 "gogo-v$VERSION-$TARGET/gogo" ~/.local/bin/gogo
gogo version
```

### With Rust

Any platform with Rust 1.85 or later:

```bash
cargo install gogo-spaces
```

The crate is `gogo-spaces` because `gogo` was taken on crates.io. The command it installs is `gogo`. Latest from GitHub: `cargo install --git https://github.com/conorbr/gogo`. From a checkout: `cargo install --path .`

### Checksums

Every archive on [Releases](https://github.com/conorbr/gogo/releases) has a `.sha256` file next to it. To check a download: `shasum -a 256 -c gogo-v0.1.0-<target>.tar.gz.sha256`.

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

## Release

1. Bump `version` in `Cargo.toml` and `VERSION=` in both download snippets above. Commit.
2. Tag and push: `git tag -a vX.Y.Z -m "gogo X.Y.Z" && git push origin main vX.Y.Z`. The Release workflow builds the binaries and attaches them to the GitHub Release.
3. `cargo publish`.
4. Update the formula in [conorbr/homebrew-tap](https://github.com/conorbr/homebrew-tap) (steps in its README).

## Licence

MIT or Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
