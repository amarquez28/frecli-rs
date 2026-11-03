#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
HOOK_DIR="$HOME/.config/frecli"

# Usage: install.sh [--shell zsh|bash]   (default: detected from $SHELL)
SHELL_NAME="$(basename "${SHELL:-}")"
if [[ "${1:-}" == "--shell" ]]; then
    SHELL_NAME="${2:-}"
elif [[ -n "${1:-}" ]]; then
    echo "usage: $0 [--shell zsh|bash]" >&2
    exit 1
fi

case "$SHELL_NAME" in
    zsh)
        RC="${ZDOTDIR:-$HOME}/.zshrc"
        ;;
    bash)
        # macOS terminals start bash as a login shell, which skips ~/.bashrc.
        if [[ "$(uname -s)" == "Darwin" ]]; then
            RC="$HOME/.bash_profile"
        else
            RC="$HOME/.bashrc"
        fi
        ;;
    *)
        echo "error: unsupported shell '$SHELL_NAME' — frecli supports zsh and bash." >&2
        echo "       Pass --shell zsh or --shell bash to choose one explicitly." >&2
        exit 1
        ;;
esac
HOOK_FILE="$HOOK_DIR/hook.$SHELL_NAME"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found — install Rust from https://rustup.rs" >&2
    exit 1
fi

echo "Building release binary..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml"

mkdir -p "$BIN_DIR" "$HOOK_DIR"
cp "$REPO_DIR/target/release/frecli-rs" "$BIN_DIR/frecli-rs"
cp "$REPO_DIR/scripts/hook.$SHELL_NAME" "$HOOK_FILE"
echo "Installed binary to $BIN_DIR/frecli-rs"
echo "Installed hook to   $HOOK_FILE"

# Add the source line unless some frecli hook is already sourced.
if grep -Eq "(^|[^#])(source|\\.) .*frecli.*hook\\.$SHELL_NAME" "$RC" 2>/dev/null; then
    echo "A frecli hook is already sourced in $RC — leaving it as is."
else
    printf '\n# frecli\nsource "%s"\n' "$HOOK_FILE" >> "$RC"
    echo "Added source line to $RC"
fi

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "warning: $BIN_DIR is not on your PATH — add this to $RC before the frecli line:
    export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac

echo "Done. Restart your shell (or run: source \"$HOOK_FILE\")"
