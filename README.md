# unvrs

**Universal package manager CLI — one interface over many package managers.**

[![Version](https://img.shields.io/badge/version-0.5.1-blue.svg)](https://github.com/devdidacg/unvrs/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

---

## What is unvrs?

unvrs is a unified CLI that orchestrates existing package managers. It does not replace native package managers — it delegates to them.

```bash
sudo unvrs install fish
```

unvrs detects the operating system, finds available package managers, searches for the package, and installs it through the appropriate backend.

### Features

- **24 package manager backends** — all fully implemented
- **AUR support** — searches AUR when yay/paru is installed
- **Universal backends** — flatpak, snap, nix, guix, brew work on ANY Linux distro
- **Container backends** — run apt, dnf, yum, apk, pacman, zypper via Docker/Podman
- **Cross-distro installs** — install from apt on Arch, from pacman on Ubuntu, etc.
- Compact output with icons (✓ ✗ ⚠ ●)
- Animated spinner during operations
- Short flags for quick use (`-s`, `-i`, `-r`, etc.)
- OS compatibility warnings
- Auto-detects available package managers
- **JSON output** (`--json`) for scripts and automation
- **No-color mode** (`--no-color`) for pipes and CI
- **Outdated packages** (`unvrs outdated`)
- **Installation history** (`unvrs history`)
- **Dry run** (`unvrs install --dry`)
- **Backend filter** (`unvrs search --backend pacman`)
- **Force install** (`unvrs install --force`) — install from any backend regardless of OS
- **Clean cache** (`unvrs clean`)
- **Search cache** — faster repeated searches
- TOML configuration
- Typed errors with context

---

## Supported Backends

### Native Backends

| Backend | Status | Platforms |
|:--------|:------:|-----------|
| pacman | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Arch Linux, Manjaro, EndeavourOS |
| apt | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Debian, Ubuntu, Linux Mint |
| dnf | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Fedora, RHEL, CentOS 8+ |
| yum | ![](https://img.shields.io/badge/-Implemented-brightgreen) | RHEL/CentOS 7 |
| zypper | ![](https://img.shields.io/badge/-Implemented-brightgreen) | openSUSE, SLES |
| apk | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Alpine Linux |
| xbps | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Void Linux |
| moss | ![](https://img.shields.io/badge/-Implemented-brightgreen) | moss-based distros |
| emerge | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Gentoo, Funtoo |
| eopkg | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Solus |

### Universal Backends (work on ANY Linux)

| Backend | Status | Notes |
|:--------|:------:|-------|
| flatpak | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Containerized apps |
| snap | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Containerized apps |
| nix | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Reproducible builds |
| guix | ![](https://img.shields.io/badge/-Implemented-brightgreen) | GNU project |
| brew | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Homebrew |

### Container Backends (via Docker/Podman)

| Backend | Status | Image |
|:--------|:------:|-------|
| apt (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | debian:latest |
| apt (podman) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | debian:latest |
| dnf (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | fedora:latest |
| dnf (podman) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | fedora:latest |
| yum (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | centos:7 |
| apk (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | alpine:latest |
| pacman (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | archlinux:latest |
| zypper (docker) | ![](https://img.shields.io/badge/-Implemented-brightgreen) | opensuse/leap:latest |

---

## Installation

### Quick install (one-liner)

```bash
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/install.sh | bash
```

### Install / Update / Uninstall

```bash
# Install
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/install.sh | bash

# Update to latest version
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/update.sh | bash

# Uninstall
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/uninstall.sh | bash
```

### Docker

```bash
docker build -t unvrs .
docker run unvrs doctor
```

### Manual install

<details>
<summary><b>Arch Linux / Archcraft</b></summary>

```bash
sudo pacman -S rust git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

</details>

<details>
<summary><b>Debian / Ubuntu</b></summary>

```bash
sudo apt install rustc cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

</details>

<details>
<summary><b>Fedora</b></summary>

```bash
sudo dnf install rust cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

</details>

### Verify

```bash
unvrs --version
unvrs doctor
```

---

## Usage

### Commands

| Command | Short | Description |
|---------|:-----:|-------------|
| `unvrs search <pkg>` | `-s` | Search all backends |
| `unvrs info <pkg>` | `-I` | Show package details |
| `unvrs install <pkg>` | `-i` | Install a package |
| `unvrs remove <pkg>` | `-r` | Remove a package |
| `unvrs update` | `-U` | Update package lists |
| `unvrs upgrade` | `-u` | Upgrade packages |
| `unvrs list` | `-l` | List installed packages |
| `unvrs outdated` | — | Show packages with updates |
| `unvrs history` | — | Show installation history |
| `unvrs clean` | — | Clean package cache |
| `unvrs doctor` | — | Diagnose system |

### Global flags

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format |
| `--no-color` | Disable colored output |

### Command flags

| Flag | Command | Description |
|------|---------|-------------|
| `--backend <name>` | search | Search only in a specific backend |
| `--dry` | install | Simulate installation without changes |
| `--force` | install | Install from any backend (cross-distro) |

### Examples

```bash
# Search for a package across all backends
unvrs -s fish
unvrs search firefox

# Search only in pacman
unvrs search --backend pacman neovim

# Install a package (uses the best available backend)
sudo unvrs -i fish
sudo unvrs install neovim

# Dry run — simulate installation
sudo unvrs install --dry fish

# Force install from any backend (cross-distro)
sudo unvrs install --force apt git    # Install git from apt on Arch
sudo unvrs install --force dnf vim    # Install vim from dnf on Ubuntu

# Get package info
unvrs -I git

# Remove a package
sudo unvrs -r fish

# Update package lists
sudo unvrs -U

# Upgrade all packages
sudo unvrs -u

# List installed packages
unvrs -l

# Check for outdated packages
unvrs outdated

# View installation history
unvrs history

# Clean package cache
sudo unvrs clean

# Diagnose your system
unvrs doctor

# JSON output for scripts
unvrs --json search fish
unvrs --json list

# No-color output for pipes
unvrs --no-color list | grep vim
```

### Output example

```
  Searched fish

  ✓ pacman
  ✗ apt
  ✗ dnf

  Results:
  ● fish 4.0.0 (pacman)
    Friendly interactive shell
  ● fish 4.0.0 (pacman (aur))
```

### JSON output

```json
[
  {
    "name": "fish",
    "version": "4.0.0",
    "source": "system",
    "backend": "pacman",
    "architecture": null,
    "description": "Friendly interactive shell"
  }
]
```

### OS Compatibility

If you try to install from a backend that doesn't match your OS, unvrs warns you:

```
  ⚠ apt (not native to Linux)
```

With `--force`, you can install from any backend regardless of OS:

```bash
# On Arch, install from apt via Docker
sudo unvrs install --force apt git
```

---

## Configuration

Optional config at `~/.config/unvrs/config.toml`:

```toml
[resolver]
preferred_backends = ["pacman"]

[output]
color = true
verbose = false
```

---

## Architecture

```
CLI (clap)
  → Core (resolver, OS detection, config, cache)
    → Backend registry
      → Individual backends (pacman, apt, dnf, ...)
      → Universal backends (flatpak, snap, nix, guix, brew)
      → Container backends (apt-docker, dnf-docker, ...)
```

### Project structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Public modules
├── cli.rs           # Clap CLI definitions
├── config.rs        # TOML config loading
├── cache.rs         # Search result cache
├── history.rs       # Installation history
├── error.rs         # Typed errors
├── executor.rs      # Safe process execution
├── os.rs            # OS detection
├── package.rs       # Domain models
├── registry.rs      # Backend discovery
├── dispatcher.rs  # Universal search/install logic
├── ui.rs            # Spinner, icons, formatting
└── backends/
    ├── mod.rs       # PackageManager trait
    ├── pacman.rs    # Arch Linux + AUR
    ├── apt.rs       # Debian/Ubuntu
    ├── dnf.rs       # Fedora
    ├── yum.rs       # RHEL/CentOS 7
    ├── zypper.rs    # openSUSE
    ├── apk.rs       # Alpine
    ├── xbps.rs      # Void Linux
    ├── moss.rs      # moss-based distros
    ├── emerge.rs    # Gentoo
    ├── eopkg.rs     # Solus
    ├── nix.rs       # NixOS
    ├── guix.rs      # GNU Guix
    ├── flatpak.rs   # Any Linux
    ├── snap.rs      # Any Linux
    ├── pkg.rs       # FreeBSD
    ├── brew.rs      # macOS/Linux
    └── container.rs # Docker/Podman backends
```

---

## Development

```bash
# Build
cargo build

# Run
cargo run -- -s fish

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

---

## Security

- No shell injection — arguments are passed separately via `std::process::Command`
- Native package managers handle signature verification and dependency resolution
- Root operations require explicit `sudo`
- Container backends run with `--rm` flag (auto-cleanup)

---

## License

MIT
