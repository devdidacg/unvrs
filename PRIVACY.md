# Privacy Policy

**TL;DR: unvrs collects no data, phones home nowhere, and everything it
stores stays on your machine.**

## Data collection

unvrs does **not**:

- collect or transmit personal data, usage statistics, or telemetry
- include analytics, trackers, crash reporters, or advertising
- make any network connections of its own
- read files outside its own configuration/data directories (plus
  standard OS metadata such as `/etc/os-release` for distro detection)

All network activity happens exclusively inside the package managers unvrs
invokes (pacman, apt, flatpak, …), under their own privacy policies.

## Data stored locally

unvrs writes only the following files, all under directories you control:

| File | Default location | Override | Contents |
|---|---|---|---|
| `config.toml` | `<user config>/unvrs/` | `UNVRS_CONFIG_DIR` | Your resolver/output preferences |
| `profiles/*.toml` | `<user config>/unvrs/profiles/` | `UNVRS_CONFIG_DIR` | Package name lists you created |
| `history.json` | `<user data>/unvrs/` | `UNVRS_DATA_DIR` | Local transaction log (see below) |

Platform defaults:

- **Linux:** `~/.config/unvrs/` and `~/.local/share/unvrs/` (XDG)
- **macOS:** `~/Library/Application Support/unvrs/` etc. via standard dirs
- **Windows:** `%APPDATA%\unvrs\`, `%LOCALAPPDATA%\unvrs\`

### What's in the history file

Each real (non-dry) transaction records: action, package name, backend,
command line, exit code, success flag, warnings, and a timestamp. This data
never leaves your machine unless **you** export or share the file.

- History is capped at the **500 most recent entries**.
- Dry runs (`--dry`, `plan`) are **never** recorded.
- Delete it any time: `rm <data dir>/unvrs/history.json` (or use
  `uninstall.sh --purge`).

## What unvrs sends to third parties

Nothing directly. When you run `unvrs install`, the underlying package
manager contacts its configured repositories — exactly as if you had run
`pacman`, `apt`, etc. yourself. Your package choices may therefore be
visible to those repositories under their own policies.

## Third-party components

unvrs is a static Rust binary with no embedded scripts or web content. Its
dependencies are audited in CI (`cargo audit`, `cargo deny`). See
`Cargo.toml` for the full list.

## Changes to this policy

Material changes will be documented in the repository commit history and
release notes. This file applies to version 0.6.0 and later.

## Contact

Questions: open an issue at https://github.com/devdidacg/unvrs/issues or a
private security advisory for sensitive matters (see [SECURITY.md](SECURITY.md)).
