#!/bin/bash
set -e

REPO="https://github.com/devdidacg/unvrs.git"
DIR="/tmp/unvrs-build"
BIN_DIR="/usr/local/bin"

echo "  unvrs installer"
echo ""

# Check dependencies
if ! command -v git &> /dev/null; then
    echo "  ✗ git not found. Install it first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "  ✗ rust/cargo not found."
    echo "  Install Rust: https://rustup.rs"
    echo "  Or use your package manager:"
    echo "    Arch:   sudo pacman -S rust"
    echo "    Debian: sudo apt install rustc cargo"
    echo "    Fedora: sudo dnf install rust cargo"
    exit 1
fi

# Clone or pull
if [ -d "$DIR" ]; then
    echo "  ● Updating repository..."
    cd "$DIR"
    git pull
else
    echo "  ● Cloning repository..."
    git clone "$REPO" "$DIR"
    cd "$DIR"
fi

# Build
echo "  ● Building..."
cargo build --release

# Install
echo "  ● Installing to $BIN_DIR..."
sudo cp target/release/unvrs "$BIN_DIR/unvrs"
sudo chmod +x "$BIN_DIR/unvrs"

# Cleanup
rm -rf "$DIR"

echo ""
echo "  ✓ unvrs installed successfully"
echo ""
echo "  Run: unvrs --version"
echo "  Run: unvrs doctor"
echo ""
