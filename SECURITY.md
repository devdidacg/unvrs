# Security Policy

## Reporting a vulnerability

If you discover a security vulnerability in unvrs, please report it
responsibly:

- **Preferred:** open a private security advisory at
  https://github.com/devdidacg/unvrs/security/advisories/new
- **Or email the maintainer** via the address on the GitHub profile with the
  subject line `[SECURITY] unvrs`.

Please include:

1. A description of the issue and its impact.
2. Steps to reproduce (PoC commands/inputs if possible).
3. Affected version (`unvrs --version`).

You can expect an acknowledgement within **7 days** and a status update
within **30 days**. Please allow reasonable time for a fix before public
disclosure. Valid reports will be credited in the advisory unless you prefer
to remain anonymous.

## Supported versions

| Version | Supported |
|---|---|
| 0.6.x | ✅ |
| < 0.6  | ❌ (upgrade recommended) |

## Security model

unvrs executes package-manager commands on your behalf. The following
guarantees are enforced by code and by tests:

- **No shell interpolation.** Every mutating command is an exec-form
  `CommandSpec` (program + argv) run via `std::process::Command`. There is
  no `sh -c` path with interpolated input.
- **Input validation.** Package names are validated before resolution;
  option-like (`-rf`), whitespace, and shell-metacharacter inputs are
  rejected.
- **Cross-distro gate.** A non-native package manager never executes on the
  host without an explicit `--cross-distro` flag.
- **Read-only planning.** `plan`, `--dry`, and `--explain` never run
  mutations and are never recorded as transactions.
- **Timeouts.** Backend commands run with timeouts so a hung package manager
  cannot hang unvrs indefinitely.
- **Privilege transparency.** Plans show the exact command, including the
  `sudo` prefix when a backend requires root. unvrs never elevates itself;
  it only prints/executes the command you asked for.
- **Path safety.** Profile names are restricted to `[A-Za-z0-9_-]{1,64}` —
  no path traversal. Config/data paths respect `UNVRS_CONFIG_DIR` and
  `UNVRS_DATA_DIR` only when you set them.

## Trust boundaries

- unvrs **delegates trust** to the underlying package managers for package
  signature verification, repository authentication, and sandboxing.
- Container backends run images pulled from Docker Hub / registry defaults —
  image trust is Docker/Podman's responsibility.
- unvrs makes **no network connections of its own** (see
  [PRIVACY.md](PRIVACY.md)); all network access happens inside the package
  managers it invokes.

## Hardening done in 0.6.0

- Removed shell-based container mutation path (shell injection).
- Added `cargo audit` and `cargo deny` to CI (advisories, licenses, sources).
- Mutation specs are checked by tests across **all** backends
  (`mutation_specs_never_use_shell_meta_for_package`).
- Failed commands surface stderr snippets so operators can detect
  permission/policy failures instead of retrying blindly.
