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

## Building

```bash
cargo build --release
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
