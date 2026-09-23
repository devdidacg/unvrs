# Adding a backend

A backend adapts one package manager to the `PackageManager` trait
(`src/backends/mod.rs`). The core never knows backend-specific command lines
or parsing rules — everything lives in your module.

## Checklist

1. Create `src/backends/<name>.rs`.
2. Implement the trait (table below).
3. Register it in `all_backends()` (`src/backends/mod.rs`).
4. Add `pub mod <name>;` at the top of `src/backends/mod.rs`.
5. Run the test suite — the generic tests already cover every registered
   backend:
   - `every_backend_declares_specs`
   - `mutation_specs_never_use_shell_meta_for_package`
   - `all_backends_have_unique_names`

## Required methods

| Method | Contract |
|---|---|
| `name()` | Unique, stable id (`"apt"`, `"flatpak"`, `"apt (docker)"`). Container backends must include `(docker)`/`(podman)` — several classifiers key off that. |
| `is_available()` | Cheap check that the PM exists on PATH (`which`). No network, no mutations. |
| `is_compatible(os)` | OS-family gate (usually `os.family == OsFamily::Linux`). Universal backends return `true` everywhere they run. |
| `search(pkg)` | Read-only search → `Vec<PackageCandidate>`. Empty vec = not found. Must tolerate any string (validated upstream, but never assume). |
| `info(pkg)` | Read-only details or `None`. |
| `list_installed()` | Installed packages. Used by `remove` gating and `unvrs list`. |
| `install_spec(pkg)` / `remove_spec(pkg)` | The mutation command lines (see rules below). |
| `update_spec()` / `upgrade_spec()` / `clean_spec()` | Refresh lists / upgrade all / clean cache. |

## Optional methods (override the defaults)

| Method | Default | When to override |
|---|---|---|
| `is_universal()` | `false` | Works on any distro (flatpak, snap, nix, guix, brew). |
| `native_distro_ids()` | `&[]` | os-release `ID`/`ID_LIKE` values where this is *the* native PM (`apt` → `&["debian","ubuntu",…]`). |
| `requires_root()` | `false` | Mutations need root (apt, dnf, pacman…). |
| `capabilities()` | `standard()` | Advertise `dry_run`/`rollback`/`transactional` honestly. |
| `version_probe()` | `None` | `--version`-style command for `unvrs doctor`. |
| `outdated()` | empty | Support `unvrs outdated`. |

## Command-spec rules (enforced by tests)

```rust
fn install_spec(&self, package: &str) -> Option<CommandSpec> {
    Some(CommandSpec::new("apt-get", ["install", "-y", package]))
}
```

- **Exec-form only.** The spec is `program` + `args: Vec<String>`; it is
  executed via `std::process::Command` — there is no shell. Never build
  `"sh -c …"` strings and never concatenate the package into a single
  argument with spaces, `;`, `|`, `&`, `$`, or backticks.
- **The package is its own argv element**, exactly `package`, or as a
  prefixed attribute the test accepts: `channel.package` (nix) or
  `flathub/package` (flatpak). If your PM needs `channel/package`, build
  `format!("{channel}/{package}")` as one argv element — fine; splitting it
  across shell tokens is not.
- **No user-controlled flags.** If a package name could start with `-`, the
  central `validate_package_id` has already rejected it; still, prefer
  argument positions that cannot be parsed as options (e.g. after a
  subcommand).
- **Return `None` only for genuinely unsupported operations** — the core
  reports "op not supported by backend" instead of failing oddly.

## Execution behavior you get for free

- `run_mutation` executes your spec, maps non-zero exits to messages with a
  stderr snippet and a `sudo` hint when the stderr looks like a permission
  error, and returns a uniform `InstallationResult`.
- Timeouts, exit-code capture, and history recording are handled by the
  dispatcher — do not record history inside a backend.
- Read-only methods (`search`, `info`, `list_installed`, `outdated`,
  `version_probe`) are safe to call in parallel contexts; keep them fast
  (prefer one command invocation, cache only via the core's search cache).

## Worked example (skeleton)

```rust
use crate::error::Result;
use crate::executor::CommandSpec;
use crate::package::*;
use super::PackageManager;

pub struct FooBackend;

impl PackageManager for FooBackend {
    fn name(&self) -> &'static str { "foo" }
    fn is_available(&self) -> bool { which::which("foo").is_ok() }
    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }
    fn native_distro_ids(&self) -> &'static [&'static str] { &["foolinux"] }
    fn requires_root(&self) -> bool { true }
    fn version_probe(&self) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let out = crate::executor::execute_with_timeout(
            "foo", &["search", package],
            std::time::Duration::from_secs(15),
        )?;
        Ok(parse_foo_search(&out.stdout))   // your parser
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> { /* … */ }
    fn list_installed(&self) -> Result<Vec<InstalledPackage>> { /* … */ }

    fn install_spec(&self, package: &str) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["install", "-y", package]))
    }
    fn remove_spec(&self, package: &str) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["remove", "-y", package]))
    }
    fn update_spec(&self) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["update-index"]))
    }
    fn upgrade_spec(&self) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["upgrade", "-y"]))
    }
    fn clean_spec(&self) -> Option<CommandSpec> {
        Some(CommandSpec::new("foo", ["clean"]))
    }
}
```

Register it:

```rust
// src/backends/mod.rs
pub mod foo;
// …
pub fn all_backends() -> Vec<Box<dyn PackageManager>> {
    vec![
        // …
        Box::new(foo::FooBackend),
    ]
}
```

## Naming container backends

Containerized PMs are built through `ContainerBackend::apt_docker()`-style
constructors (see `src/backends/container.rs`) rather than new modules —
follow that pattern for Docker/Podman variants so availability (daemon
reachable) and classification keep working.

## Docs to update

- README backend table (status/platform columns).
- `docs/architecture.md` module map if you add new files.
