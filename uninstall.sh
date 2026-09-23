#!/usr/bin/env bash
# unvrs uninstaller — remove the installed binary.
# Config/data are kept unless --purge is given.
# Idempotent: exits 0 even if unvrs is not installed.
# Usage: ./uninstall.sh [--prefix DIR] [--purge]
set -euo pipefail

BIN_DIR="/usr/local/bin"
PURGE=0

usage() {
    echo "Usage: uninstall.sh [--prefix DIR] [--purge]"
    echo "  --prefix DIR   Look for the binary in DIR (default: /usr/local/bin)"
    echo "  --purge        Also delete config and data directories"
    echo "  -h, --help     Show this help"
    echo ""
    echo "Config:  \${XDG_CONFIG_HOME:-\$HOME/.config}/unvrs"
    echo "Data:    \${XDG_DATA_HOME:-\$HOME/.local/share}/unvrs"
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
        --purge)
            PURGE=1
            shift
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

echo "  unvrs uninstaller"
echo ""

BIN="$BIN_DIR/unvrs"
if [ ! -e "$BIN" ]; then
    echo "  ⚠ unvrs not found at $BIN (nothing to remove from $BIN_DIR)"
else
    echo "  ● Removing $BIN..."
    if [ -w "$BIN_DIR" ]; then
        rm -f "$BIN"
    elif command -v sudo >/dev/null 2>&1; then
        sudo rm -f "$BIN"
    else
        echo "  ✗ $BIN_DIR is not writable and sudo is unavailable." >&2
        exit 1
    fi
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/unvrs"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/unvrs"

if [ "$PURGE" -eq 1 ]; then
    for dir in "$CONFIG_DIR" "$DATA_DIR"; do
        if [ -d "$dir" ]; then
            echo "  ● Removing $dir..."
            rm -rf "$dir"
        fi
    done
    echo ""
    echo "  ✓ unvrs removed completely"
else
    if [ -d "$CONFIG_DIR" ] || [ -d "$DATA_DIR" ]; then
        echo "  ● Keeping user data:"
        [ -d "$CONFIG_DIR" ] && echo "    config: $CONFIG_DIR"
        [ -d "$DATA_DIR" ] && echo "    data:   $DATA_DIR"
        echo "    (re-run with --purge to delete it)"
    fi
    echo ""
    echo "  ✓ unvrs removed"
fi
echo ""
