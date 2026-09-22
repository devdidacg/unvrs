# unvrs

Universal package manager CLI — one interface over many package managers.

## What is unvrs?

unvrs is a unified CLI that orchestrates existing package managers. It does not replace native package managers — it delegates to them.

```bash
sudo unvrs install fish
```

unvrs detects the operating system, finds available package managers, searches for the package, and installs it through the appropriate native backend.

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

## Installation

### From source (recommended)

Clone the repository and build with Cargo:

```bash
# Install Rust if not present
sudo pacman -S rust

# Clone and build
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release

# Install to system path
sudo cp target/release/unvrs /usr/local/bin/
```

### Verify installation

```bash
unvrs --version
unvrs doctor
```

### Arch Linux / Archcraft

```bash
sudo pacman -S rust git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

### Debian / Ubuntu

```bash
sudo apt install rustc cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

### Fedora

```bash
sudo dnf install rust cargo git
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
```

## Usage

```bash
unvrs search <package>       # Search all backends
unvrs info <package>         # Show package details
sudo unvrs install <package> # Install a package
sudo unvrs remove <package>  # Remove a package
unvrs update                 # Update package lists
unvrs upgrade                # Upgrade packages
unvrs list                   # List installed packages
unvrs doctor                 # Diagnose system
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
cargo run -- search fish

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
