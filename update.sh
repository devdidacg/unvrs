#!/bin/bash
set -e

REPO="https://github.com/devdidacg/unvrs.git"
DIR="/tmp/unvrs-build"
BIN_DIR="/usr/local/bin"

echo "  unvrs updater"
echo ""

# Check dependencies
if ! command -v git &> /dev/null; then
    echo "  ✗ git not found."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "  ✗ rust/cargo not found."
    exit 1
fi

# Clone fresh
rm -rf "$DIR"
echo "  ● Fetching latest..."
git clone --depth 1 "$REPO" "$DIR"
cd "$DIR"

# Build
echo "  ● Building..."
cargo build --release

# Install
echo "  ● Updating $BIN_DIR/unvrs..."
sudo cp target/release/unvrs "$BIN_DIR/unvrs"
sudo chmod +x "$BIN_DIR/unvrs"

# Cleanup
rm -rf "$DIR"

echo ""
echo "  ✓ unvrs updated successfully"
echo ""
echo "  Version: $(unvrs --version)"
echo ""
