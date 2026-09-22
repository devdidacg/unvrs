#!/bin/bash
set -e

BIN_DIR="/usr/local/bin"

echo "  unvrs uninstaller"
echo ""

if [ ! -f "$BIN_DIR/unvrs" ]; then
    echo "  ⚠ unvrs not found at $BIN_DIR/unvrs"
    echo "  It may not be installed."
    exit 0
fi

echo "  ● Removing $BIN_DIR/unvrs..."
sudo rm -f "$BIN_DIR/unvrs"

# Remove config
CONFIG_DIR="$HOME/.config/unvrs"
if [ -d "$CONFIG_DIR" ]; then
    echo "  ● Removing config at $CONFIG_DIR..."
    rm -rf "$CONFIG_DIR"
fi

echo ""
echo "  ✓ unvrs removed"
echo ""
