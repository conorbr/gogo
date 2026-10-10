#!/bin/sh
# gogo installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/conorbr/gogo/main/scripts/install.sh | sh
#   curl -fsSL .../install.sh | GOGO_VERSION=0.1.0 sh
#   curl -fsSL .../install.sh | GOGO_INSTALL_DIR=/usr/local/bin sh
#
# What it does, in full:
#
#   detects    your OS (macOS or Linux) and CPU (x86_64 or arm64)
#   finds      the latest release on github.com/conorbr/gogo/releases,
#              or the one in GOGO_VERSION
#   downloads  the matching archive and its .sha256, over anonymous HTTPS
#   verifies   the checksum; a mismatch aborts before anything is written
#   writes     one file, gogo, into ~/.local/bin (or GOGO_INSTALL_DIR)
#
# No root, no edits to your shell config. To uninstall, delete that one file.

set -eu

REPO="conorbr/gogo"
INSTALL_DIR="${GOGO_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf 'gogo-install: %s\n' "$*"; }
fail() {
  printf 'gogo-install: error: %s\n' "$*" >&2
  exit 1
}
need() { command -v "$1" >/dev/null 2>&1 || fail "needs $1"; }

need curl
need tar
need uname
need mktemp

case "$(uname -s)" in
  Darwin) os="apple-darwin" ;;
  Linux) os="unknown-linux-musl" ;;
  *) fail "no prebuilt gogo for $(uname -s). With Rust: cargo install gogo-spaces" ;;
esac
case "$(uname -m)" in
  x86_64 | amd64) arch="x86_64" ;;
  arm64 | aarch64) arch="aarch64" ;;
  *) fail "no prebuilt gogo for $(uname -m). With Rust: cargo install gogo-spaces" ;;
esac
target="$arch-$os"

if [ -n "${GOGO_VERSION:-}" ]; then
  version="${GOGO_VERSION#v}"
else
  # /releases/latest redirects to /releases/tag/vX.Y.Z. No API, no rate limit.
  latest=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest") ||
    fail "couldn't reach github.com"
  version="${latest##*/tag/v}"
  case "$version" in
    "" | */*) fail "couldn't work out the latest version from $latest" ;;
  esac
fi

name="gogo-v$version-$target"
url="https://github.com/$REPO/releases/download/v$version/$name.tar.gz"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
trap 'exit 1' INT TERM

say "downloading gogo $version for $target"
curl -fsSL "$url" -o "$tmp/$name.tar.gz" || fail "download failed: $url"
curl -fsSL "$url.sha256" -o "$tmp/$name.tar.gz.sha256" || fail "download failed: $url.sha256"

if command -v sha256sum >/dev/null 2>&1; then
  check="sha256sum -c"
elif command -v shasum >/dev/null 2>&1; then
  check="shasum -a 256 -c"
else
  fail "needs sha256sum or shasum to verify the download"
fi
(cd "$tmp" && $check "$name.tar.gz.sha256" >/dev/null 2>&1) ||
  fail "checksum mismatch for $name.tar.gz. Nothing was installed"

tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$INSTALL_DIR"
# Copy beside the target, then rename, so a running gogo is never half-written.
cp "$tmp/$name/gogo" "$INSTALL_DIR/.gogo.new"
chmod 755 "$INSTALL_DIR/.gogo.new"
mv -f "$INSTALL_DIR/.gogo.new" "$INSTALL_DIR/gogo"

say "installed $("$INSTALL_DIR/gogo" version) to $INSTALL_DIR/gogo"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    found=$(command -v gogo || true)
    if [ -n "$found" ] && [ "$found" != "$INSTALL_DIR/gogo" ]; then
      say "note: $found comes first on your PATH, so that's the gogo you'll run"
    fi
    ;;
  *)
    say "$INSTALL_DIR isn't on your PATH. Add this to your shell's startup file:"
    say "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac
