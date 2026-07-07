#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
HOOK_DIR="$HOME/.config/frecli"
ZSHRC="${ZDOTDIR:-$HOME}/.zshrc"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found — install Rust from https://rustup.rs" >&2
    exit 1
fi

echo "Building release binary..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml"

mkdir -p "$BIN_DIR" "$HOOK_DIR"
cp "$REPO_DIR/target/release/frecli-rs" "$BIN_DIR/frecli-rs"
cp "$REPO_DIR/scripts/hook.zsh" "$HOOK_DIR/hook.zsh"
echo "Installed binary to $BIN_DIR/frecli-rs"
echo "Installed hook to   $HOOK_DIR/hook.zsh"

# Add the source line unless some frecli hook is already sourced.
if grep -Eq '(^|[^#])source .*frecli.*hook\.zsh' "$ZSHRC" 2>/dev/null; then
    echo "A frecli hook is already sourced in $ZSHRC — leaving it as is."
else
    printf '\n# frecli\nsource "%s"\n' "$HOOK_DIR/hook.zsh" >> "$ZSHRC"
    echo "Added source line to $ZSHRC"
fi

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "warning: $BIN_DIR is not on your PATH — add this to $ZSHRC before the frecli line:
    export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac

echo "Done. Restart your shell (or run: source \"$HOOK_DIR/hook.zsh\")"
