#!/usr/bin/env bash
# unvrs updater — fetch latest source, build, replace the installed binary.
# Idempotent: safe to re-run.
# Usage: ./update.sh [--prefix DIR]
set -euo pipefail

REPO="https://github.com/devdidacg/unvrs.git"
BIN_DIR="/usr/local/bin"

usage() {
    echo "Usage: update.sh [--prefix DIR]"
    echo "  --prefix DIR   Install the binary into DIR (default: /usr/local/bin)"
    echo "  -h, --help     Show this help"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            if [ $# -lt 2 ]; then
                echo "error: --prefix requires a directory argument" >&2
                exit 2
            fi
            BIN_DIR="$2"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

echo "  unvrs updater"
echo ""

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "  ✗ $1 not found. $2"
        exit 1
    fi
}

need git "Install it first (e.g. sudo pacman -S git / sudo apt install git)."
need cargo "Install Rust: https://rustup.rs"

BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR"' EXIT

echo "  ● Fetching latest..."
git clone --depth 1 "$REPO" "$BUILD_DIR/src" >/dev/null

echo "  ● Building..."
cargo build --release --manifest-path "$BUILD_DIR/src/Cargo.toml"

BIN_SRC="$BUILD_DIR/src/target/release/unvrs"
if [ ! -f "$BIN_SRC" ]; then
    echo "  ✗ build produced no binary at $BIN_SRC" >&2
    exit 1
fi

echo "  ● Updating $BIN_DIR/unvrs..."
if [ -d "$BIN_DIR" ] && [ -w "$BIN_DIR" ]; then
    cp "$BIN_SRC" "$BIN_DIR/unvrs"
    chmod 0755 "$BIN_DIR/unvrs"
elif command -v sudo >/dev/null 2>&1; then
    sudo mkdir -p "$BIN_DIR"
    sudo cp "$BIN_SRC" "$BIN_DIR/unvrs"
    sudo chmod 0755 "$BIN_DIR/unvrs"
else
    echo "  ✗ $BIN_DIR is not writable and sudo is unavailable." >&2
    echo "    Re-run with --prefix pointing to the directory unvrs is installed in." >&2
    exit 1
fi

echo ""
echo "  ✓ unvrs updated successfully"
echo ""
if command -v unvrs >/dev/null 2>&1; then
    echo "  Version: $(unvrs --version)"
else
    echo "  Note: $BIN_DIR is not in your PATH; add it to use \`unvrs\`."
fi
echo ""
