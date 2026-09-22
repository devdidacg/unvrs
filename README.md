# unvrs

Universal package manager CLI — one interface over many package managers.

## What is unvrs?

unvrs is a unified CLI that orchestrates existing package managers. It does not replace native package managers — it delegates to them.

```bash
sudo unvrs install fish
```

unvrs detects the operating system, finds available package managers, searches for the package, and installs it through the appropriate native backend.

### Features

- Compact output with icons (✓ ✗ ⚠ ●)
- Animated spinner during operations
- Short flags for quick use (`-s`, `-i`, `-r`, etc.)
- OS compatibility warnings (prevents using wrong backend)
- Auto-detects available package managers
- Typed errors with context

## Supported Backends

| Backend | Status | Platforms |
|---------|--------|-----------|
| pacman | Implemented | Arch Linux, Manjaro, etc. |
| apt | Implemented | Debian, Ubuntu, etc. |
| dnf | Implemented | Fedora, RHEL, etc. |
| yum | Stub | RHEL/CentOS 7 |
| zypper | Stub | openSUSE |
| apk | Stub | Alpine |
| xbps | Stub | Void Linux |
| moss | Stub | moss-based distros |
| emerge | Stub | Gentoo |
| eopkg | Stub | Solus |
| nix | Stub | NixOS |
| guix | Stub | GNU Guix |
| flatpak | Stub | Any Linux |
| snap | Stub | Any Linux |
| pkg | Stub | FreeBSD |
| brew | Stub | macOS |

## Quick Install (one-liner)

```bash
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/install.sh | bash
```

## Installation

### Using the install script

```bash
# Install
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/install.sh | bash

# Update
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/update.sh | bash

# Uninstall
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/uninstall.sh | bash
```

### Manual install — Arch Linux / Archcraft

```bash
sudo pacman -S rust git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

### Manual install — Debian / Ubuntu

```bash
sudo apt install rustc cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

### Manual install — Fedora

```bash
sudo dnf install rust cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

### Verify

```bash
unvrs --version
unvrs doctor
```

## Usage

### Commands

| Command | Short | Description |
|---------|-------|-------------|
| `unvrs search <pkg>` | `-s` | Search all backends |
| `unvrs info <pkg>` | `-I` | Show package details |
| `unvrs install <pkg>` | `-i` | Install a package |
| `unvrs remove <pkg>` | `-r` | Remove a package |
| `unvrs update` | `-U` | Update package lists |
| `unvrs upgrade` | `-u` | Upgrade packages |
| `unvrs list` | `-l` | List installed packages |
| `unvrs doctor` | — | Diagnose system |

### Examples

```bash
# Search
unvrs -s fish
unvrs search firefox

# Install
sudo unvrs -i fish
sudo unvrs install neovim

# Info
unvrs -I git

# Remove
sudo unvrs -r fish

# Update & Upgrade
sudo unvrs -U
sudo unvrs -u

# List installed
unvrs -l

# Diagnose
unvrs doctor
```

### Output

```
  ✓ Searching for fish...

  ✓ pacman
  ✗ apt
  ✗ dnf

  Results:
  ● fish 4.0.0 (pacman)
    Friendly interactive shell
```

### OS Compatibility

If you try to install from a backend that doesn't match your OS, unvrs warns you:

```
  ⚠ apt (not native to Linux)
```

## Configuration

Optional config at `~/.config/unvrs/config.toml`:

```toml
[resolver]
preferred_backends = ["pacman"]

[output]
color = true
verbose = false
```

## Architecture

```
CLI (clap)
  → Core (resolver, OS detection, config)
    → Backend registry
      → Individual backends (pacman, apt, dnf, ...)
```

Core logic is portable. Linux-specific code is isolated in backends.

## Development

```bash
# Build
cargo build

# Run
cargo run -- -s fish

# Test
cargo test

# Lint
cargo fmt
cargo clippy
```

## Testing

```bash
cargo test
```

Unit tests mock process execution. Integration tests run on real Linux systems.

## Security

- No shell injection — arguments are passed separately via `std::process::Command`
- Native package managers handle signature verification and dependency resolution
- Root operations require explicit `sudo`

## License

MIT
