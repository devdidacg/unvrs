# unvrs

**Universal package manager CLI — one interface over many package managers.**

[![Version](https://img.shields.io/badge/version-0.2.4-blue.svg)](https://github.com/devdidacg/unvrs/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

---

## What is unvrs?

unvrs is a unified CLI that orchestrates existing package managers. It does not replace native package managers — it delegates to them.

```bash
sudo unvrs install fish
```

unvrs detects the operating system, finds available package managers, searches for the package, and installs it through the appropriate native backend.

### Features

- **16 package manager backends** — all fully implemented
- Compact output with icons (✓ ✗ ⚠ ●)
- Animated spinner during operations
- Short flags for quick use (`-s`, `-i`, `-r`, etc.)
- OS compatibility warnings
- Auto-detects available package managers
- TOML configuration
- Typed errors with context

---

## Supported Backends

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
| nix | ![](https://img.shields.io/badge/-Implemented-brightgreen) | NixOS |
| guix | ![](https://img.shields.io/badge/-Implemented-brightgreen) | GNU Guix |
| flatpak | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Any Linux |
| snap | ![](https://img.shields.io/badge/-Implemented-brightgreen) | Any Linux |
| pkg | ![](https://img.shields.io/badge/-Implemented-brightgreen) | FreeBSD |
| brew | ![](https://img.shields.io/badge/-Implemented-brightgreen) | macOS, Linux |

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
| `unvrs doctor` | — | Diagnose system |

### Examples

```bash
# Search for a package across all backends
unvrs -s fish
unvrs search firefox

# Install a package (uses the best available backend)
sudo unvrs -i fish
sudo unvrs install neovim

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

# Diagnose your system
unvrs doctor
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
```

### OS Compatibility

If you try to install from a backend that doesn't match your OS, unvrs warns you:

```
  ⚠ apt (not native to Linux)
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
  → Core (resolver, OS detection, config)
    → Backend registry
      → Individual backends (pacman, apt, dnf, ...)
```

### Project structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Public modules
├── cli.rs           # Clap CLI definitions
├── config.rs        # TOML config loading
├── error.rs         # Typed errors
├── executor.rs      # Safe process execution
├── os.rs            # OS detection
├── package.rs       # Domain models
├── registry.rs      # Backend discovery
├── resolver.rs      # Universal search/install logic
├── ui.rs            # Spinner, icons, formatting
└── backends/
    ├── mod.rs       # PackageManager trait
    ├── pacman.rs    # Arch Linux
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
    └── brew.rs      # macOS/Linux
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

---

## License

MIT
