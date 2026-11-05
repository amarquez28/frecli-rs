#!/bin/sh
# Update frecli to the latest GitHub release without needing the repo or Rust.
# Replaces the binary and any hook files already installed; never edits rc files.
#
# Usage: update.sh [--force]    (--force reinstalls even if already up to date)
set -eu

REPO="amarquez28/frecli-rs"
HOOK_DIR="$HOME/.config/frecli"

FORCE=0
case "${1:-}" in
    "") ;;
    --force) FORCE=1 ;;
    *) echo "usage: $0 [--force]" >&2; exit 1 ;;
esac

command -v curl >/dev/null 2>&1 || { echo "error: curl is required" >&2; exit 1; }
command -v tar  >/dev/null 2>&1 || { echo "error: tar is required" >&2; exit 1; }

# Pick the release archive for this platform (names match release.yml).
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)                TARGET="x86_64-unknown-linux-musl" ;;
    Linux-aarch64|Linux-arm64)   TARGET="aarch64-unknown-linux-musl" ;;
    Darwin-arm64)                TARGET="aarch64-apple-darwin" ;;
    Darwin-x86_64)               TARGET="x86_64-apple-darwin" ;;
    *) echo "error: no prebuilt binary for $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac

# Update the binary that is actually in use; fall back to the install default.
if BIN="$(command -v frecli-rs 2>/dev/null)"; then
    :
else
    BIN="$HOME/.local/bin/frecli-rs"
fi
BIN_DIR="$(dirname "$BIN")"

# The /releases/latest redirect names the newest tag (no API rate limit).
LATEST="$(curl -fsSI -o /dev/null -w '%{redirect_url}' \
    "https://github.com/$REPO/releases/latest" | sed 's|.*/tag/v||')"
[ -n "$LATEST" ] || { echo "error: could not determine the latest version" >&2; exit 1; }

# `frecli-rs --version` prints "frecli-rs 1.3.0".
CURRENT=""
[ -x "$BIN" ] && CURRENT="$("$BIN" --version 2>/dev/null | awk '{print $2}')" || true

if [ "$CURRENT" = "$LATEST" ] && [ "$FORCE" -eq 0 ]; then
    echo "frecli is already up to date ($CURRENT)."
    exit 0
fi
echo "Updating frecli ${CURRENT:-(not installed)} -> $LATEST"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT INT TERM

echo "Downloading frecli-rs-$TARGET.tar.gz..."
curl -fsSL "https://github.com/$REPO/releases/download/v$LATEST/frecli-rs-$TARGET.tar.gz" \
    | tar -xz -C "$TMP"

mkdir -p "$BIN_DIR"
# Copy beside the target, then rename: replaces the file atomically and avoids
# "text file busy" if frecli is running.
install -m 755 "$TMP/frecli-rs" "$BIN.new"
mv -f "$BIN.new" "$BIN"
echo "Updated binary: $BIN"

# Refresh only hooks that are already installed, so we never add a shell the
# user didn't set up. The rc file already sources them.
for hook in hook.bash hook.zsh; do
    if [ -f "$HOOK_DIR/$hook" ]; then
        cp "$TMP/$hook" "$HOOK_DIR/$hook"
        echo "Updated hook:   $HOOK_DIR/$hook"
    fi
done

echo "Done. Restart your shell to load the new hook."
