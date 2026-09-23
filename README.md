# unvrs

**Universal package manager CLI — one interface over many package managers.**

[![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)](https://github.com/devdidacg/unvrs/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/devdidacg/unvrs/actions/workflows/ci.yml/badge.svg)](https://github.com/devdidacg/unvrs/actions/workflows/ci.yml)

---

## What is unvrs?

unvrs orchestrates existing package managers — it does not replace them.
It resolves *which* package manager should handle a package, shows you the
exact plan, then executes it safely.

```bash
unvrs plan install fish     # resolve + print the plan — changes nothing
sudo unvrs install fish     # execute through the best backend
```

### Highlights (0.6.0)

- **Plan / apply workflow** — `unvrs plan …`, `--dry`, `--explain` show the
  chosen backend, the reasons, and the exact command before anything runs.
- **Profiles** — save named package sets (`unvrs profile create dev git
  neovim ripgrep`) and apply them later (`unvrs apply dev`).
- **24 backends** — native distro PMs, universal PMs (flatpak, snap, nix,
  guix, brew) and container PMs (apt/dnf/yum/apk/pacman/zypper via
  Docker/Podman).
- **Transparent resolution** — `--explain` prints every candidate, its class
  (native/universal/container/cross-distro), and why the winner won.
- **Safe by default** — exec-form argv only (no shell interpolation),
  cross-distro installs gated behind `--cross-distro`, package names
  validated before anything runs, command timeouts, read-only planning is
  never recorded as a transaction.
- **Transaction history** — every real change is recorded (`unvrs history`,
  `unvrs history show <id>`) with backend, command and exit code. Dry runs
  are never recorded.
- **Machine-readable output** — `--json` on every command for scripts/CI.
- **Actionable errors** — did-you-mean backend names, stderr snippets,
  contextual suggestions, `unvrs doctor` for environment checks.
- Compact output with icons (✓ ✗ ⚠ ●), animated TTY spinner, `-v` verbosity,
  `--no-color` for pipes.

---

## Supported backends

### Native

| Backend | Platforms |
|---|---|
| pacman | Arch, Manjaro, EndeavourOS |
| apt | Debian, Ubuntu, Mint |
| dnf | Fedora, RHEL, CentOS 8+ |
| yum | RHEL/CentOS 7 |
| zypper | openSUSE, SLES |
| apk | Alpine |
| xbps | Void |
| moss | moss-based distros |
| emerge | Gentoo, Funtoo |
| eopkg | Solus |

### Universal (any distro where installed)

flatpak · snap · nix · guix · brew · pkg (FreeBSD)

### Container (via Docker/Podman)

apt · dnf · yum · apk · pacman · zypper — each as `… (docker)` and
`… (podman)` where applicable. Requires a reachable container daemon.

### Cross-distro

Any non-native backend can run directly on your host only with an explicit
`--cross-distro` flag (legacy alias: `--force`). Universal backends never
need it.

---

## Installation

### Quick install

```bash
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/install.sh | bash
```

### Update / uninstall

```bash
# Update to latest
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/update.sh | bash

# Uninstall (keeps config/data)
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/uninstall.sh | bash

# Uninstall and delete config + history
curl -sSL https://raw.githubusercontent.com/devdidacg/unvrs/main/uninstall.sh | bash -s -- --purge
```

Scripts are idempotent and ShellCheck-clean; use `--prefix DIR` to install
somewhere other than `/usr/local/bin`.

### Docker

```bash
docker build -t unvrs .
docker run --rm unvrs doctor
```

### Manual build

```bash
git clone https://github.com/devdidacg/unvrs.git
cd unvrs
cargo build --release
sudo cp target/release/unvrs /usr/local/bin/
unvrs --version && unvrs doctor
```

Dependencies by distro: `sudo pacman -S rust git` ·
`sudo apt install rustc cargo git` · `sudo dnf install rust cargo git`

---

## Usage

### Commands

| Command | Short | Description |
|---|:---:|---|
| `unvrs search <pkg>` | `-s` | Search all backends |
| `unvrs info <pkg>` | `-I` | Show package details |
| `unvrs install <pkg…>` | `-i` | Install package(s) |
| `unvrs remove <pkg…>` | `-r` | Remove installed package(s) |
| `unvrs plan install\|remove <pkg…>` | — | Print the plan, change nothing |
| `unvrs apply <profile>` | — | Apply a named profile |
| `unvrs profile create\|list\|show\|plan\|apply` | — | Manage profiles |
| `unvrs update` | `-U` | Update package lists |
| `unvrs upgrade` | `-u` | Upgrade installed packages |
| `unvrs list` | `-l` | List installed packages |
| `unvrs outdated` | — | Show packages with updates |
| `unvrs history [show <id>]` | — | Transaction history |
| `unvrs clean` | — | Clean package caches |
| `unvrs doctor` | — | Diagnose system configuration |

### Global flags

| Flag | Description |
|---|---|
| `--json` | JSON output (single document on stdout; errors as JSON too) |
| `--no-color` | Disable colors |
| `-v`, `-vv`, `-vvv` | Log verbosity (stderr) |

### Plan & safety flags (install/remove/apply/plan)

| Flag | Description |
|---|---|
| `--dry` | Simulate: resolve and print the plan, record nothing |
| `--explain` | Print the full resolution report (candidates + reasons) |
| `--backend <name>` | Use a specific backend (did-you-mean on typos) |
| `--cross-distro` | Allow a non-native backend to run on the host |
| `--container` | Prefer a container backend (install only) |
| `--force` | Deprecated alias for `--cross-distro` (hidden) |

### Examples

```bash
# Search
unvrs search firefox
unvrs search --backend pacman neovim

# See exactly what would happen — then do it
unvrs plan install firefox
unvrs install --dry fish
unvrs install --explain fish
sudo unvrs install fish vim            # multiple packages

# Pick the backend yourself
sudo unvrs install --backend flatpak vlc
sudo unvrs install --backend apt git --cross-distro   # non-native, explicit

# Remove only what a backend really has installed
sudo unvrs remove fish
unvrs plan remove fish                 # preview first

# Profiles
unvrs profile create dev git neovim ripgrep
unvrs profile list
unvrs profile plan dev                 # preview the whole profile
unvrs apply dev --dry
sudo unvrs apply dev

# Housekeeping
sudo unvrs update
sudo unvrs upgrade
unvrs list
unvrs outdated
sudo unvrs clean

# History (dry runs are never recorded)
unvrs history
unvrs history show 42

# Automation
unvrs --json search fish
unvrs --json doctor
unvrs --json plan install vim
```

### Plan output

```
UNVRS INSTALL PLAN

Package:  fish (resolved: fish)
Backend:  pacman (native)
Source:   Arch Linux repository (pacman)

Reason:
  + native package manager for Arch Linux (arch)
  + policy priority: unlisted (fallback)
  + exact package name match
  + package available in this backend

Command:
  sudo pacman -S --noconfirm fish

Privileges: root/sudo required
Rollback:   not supported by the selected backend(s)
System:     Arch Linux (x86_64)

No changes have been made.
```

### Explain output

```bash
unvrs install --explain fish
```

```
Detected system:
  Arch Linux
  x86_64 (Linux)

Candidates:
  pacman       selected           (native)
  flatpak      no package          (universal)
  apt (docker) unavailable        (container)
    backend not installed
  …

Selected:
  pacman
  resolved package: fish

Reason:
  + native package manager for Arch Linux (arch)
  + exact package name match
  …
```

### JSON contract

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

Errors are JSON too when `--json` is set:

```json
{
  "error": "unknown backend: fltapak",
  "did_you_mean": "flatpak",
  "suggestion": "Run `unvrs doctor` to see valid backend names."
}
```

Exit codes: `0` success · `1` failure · `2` usage error.

---

## Configuration

Optional `~/.config/unvrs/config.toml` (XDG on Linux, `%APPDATA%` on Windows):

```toml
[resolver]
# Order matters: earlier = preferred. Names or class keywords.
prefer = ["native", "universal", "flatpak"]
avoid  = ["snap"]

[output]
color = true
```

- `prefer` accepts backend names (`"apt"`) or keywords
  (`"native"`, `"universal"`, `"container"`).
- `avoid` demotes matching backends (shown as *avoided* in `--explain`).
- Invalid TOML aborts with the file path — no silent fallback to defaults.

Environment overrides (useful for testing/CI/portable installs):

| Variable | Effect |
|---|---|
| `UNVRS_CONFIG_DIR` | Directory for `config.toml` and `profiles/` |
| `UNVRS_DATA_DIR` | Directory for `history.json` |

Profiles live in `<config dir>/profiles/<name>.toml`; names may contain
letters, digits, `-`, `_` (max 64 chars).

---

## Architecture

```
CLI (clap)
  → Dispatcher
      → Resolver   (read-only probe of every backend → deterministic pick)
      → Plan       (exact commands + privileges — pure data)
      → Executor   (exec-form argv, timeouts, exit-code mapping)
          → Backend registry (native / universal / container / cross)
```

Full details: [docs/architecture.md](docs/architecture.md) ·
Adding a backend: [docs/backend-development.md](docs/backend-development.md)

---

## Development

```bash
cargo build                  # build
cargo run -- search fish     # run from source
cargo test                   # unit + integration tests
cargo fmt                    # format
cargo clippy --all-targets --all-features -- -D warnings
```

CI runs fmt, clippy (`-D warnings`), tests on Ubuntu + Windows, `cargo
audit`, `cargo deny`, and a Docker build.

---

## Security

- No shell injection — commands are exec-form (`program` + argv), package
  names validated before resolution, hostile names (`-rf`, spaces, `;|&$\``)
  rejected.
- Non-native backends require explicit `--cross-distro`.
- All backend commands run with timeouts; failures surface stderr snippets.
- Planning is read-only and never recorded as a transaction.
- Root operations only ever run the exact command shown in the plan, via
  `sudo` when the backend declares `requires_root`.
- Dependency hygiene: `cargo audit` + `cargo deny` in CI.

---

## License

MIT
