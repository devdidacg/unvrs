# Architecture

unvrs is a universal package-manager CLI: it does **not** replace package
managers — it orchestrates them. Every user-visible operation flows through a
fixed pipeline:

```
CLI (clap)                                  src/cli.rs
  → Dispatcher                              src/dispatcher.rs
      → Resolver  (which backend?)          src/resolver.rs
      → Plan      (what exactly?)           src/plan.rs
      → Executor  (run it safely)           src/executor.rs
          → Backend (PackageManager impl)   src/backends/*
```

## Module map

| Module | Responsibility |
|---|---|
| `cli.rs` | clap definitions only. No business logic. |
| `main.rs` | process entry: init context, load config, dispatch, render result, exit code. |
| `dispatcher.rs` | orchestrates resolve → plan → execute; search/list/update/upgrade/clean; installed-map tracking. |
| `resolver.rs` | deterministic backend selection, candidate reports, `--explain` rendering. |
| `plan.rs` | pure data (`Plan`, `PlannedAction`) + human/JSON rendering. Never executes. |
| `executor.rs` | `CommandSpec` (program + argv), safe execution, timeouts, exit-code mapping. |
| `backends/mod.rs` | `PackageManager` trait, registry (`all_backends`), availability filters, `run_mutation`. |
| `backends/*.rs` | per-PM knowledge: command lines, output parsing, distro IDs. |
| `registry.rs` | backend lookup by name (incl. container aliases). |
| `config.rs` | TOML config, policy (`prefer`/`avoid`), path resolution incl. env overrides. |
| `profile.rs` | named package profiles (TOML), name validation. |
| `history.rs` | transaction history (JSON, capped at 500 entries). |
| `doctor.rs` | system/backend/config diagnostics, stable JSON shape. |
| `error.rs` | `UnvrsError` variants + actionable `suggestion()`. |
| `ui.rs` | spinner (TTY-only, `Drop`-safe), icons, color helpers. |
| `context.rs` | global CLI flags via `OnceLock` (no `static mut`). |
| `os.rs`, `package.rs`, `cache.rs`, `suggest.rs` | OS detection, domain types, search cache, Levenshtein did-you-mean. |

## The install/remove pipeline

1. **Parse** — clap validates shape; global flags (`--json`, `--no-color`,
   `-v…`) go to `context`.
2. **Resolve** (`resolver.rs`) — for each requested package:
   - `validate_package_id` rejects option-like or shell-hostile names up front.
   - Each registered backend is probed *read-only* (`search`); the result is
     recorded as a `CandidateReport` (`selected` / `candidate` / `no-results` /
     `unavailable` / `error` / `cross-distro-not-allowed` / `not-installed` /
     `avoided`).
   - Sorting key: `(policy_rank, native_rank, !exact_match, class,
     registration_index)` — fully deterministic.
   - If *no* backend produced a candidate but some errored, the user gets
     `BackendUnavailable` with the per-backend notes — never a bogus
     "package not found".
   - `remove` additionally consults the installed-map: a package may only be
     removed through a backend that reports it installed.
3. **Plan** (`plan.rs`) — resolutions become a `Plan` of `PlannedAction`s.
   `attach_commands` pulls the exact `CommandSpec` from the backend, so the
   plan shows the real command and whether sudo is needed. Plans are pure
   data: `unvrs plan …` and `--dry` print them and stop. `rollback_supported`
   is honestly `false` for every backend today.
4. **Execute** (`dispatcher::execute_plan`) — actions run in order, each via
   `backends::run_mutation`; the first failure stops the transaction. Every
   real (non-dry) transaction is appended to history with backend, command,
   exit code and warnings.
5. **Render** (`main::finish_execution`) — human output with icons, or a
   machine-stable JSON envelope. Failures exit `1`; JSON errors always carry
   `error` + optional `suggestion`/`did_you_mean`.

## Backend abstraction

```rust
trait PackageManager {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn is_compatible(&self, os: &OperatingSystem) -> bool;
    fn is_universal(&self) -> bool;               // default false
    fn native_distro_ids(&self) -> &[&str];       // os-release ID/ID_LIKE
    fn requires_root(&self) -> bool;              // default false
    fn capabilities(&self) -> BackendCapabilities;
    fn version_probe(&self) -> Option<CommandSpec>;

    // read-only
    fn search(&self, pkg: &str) -> Result<Vec<PackageCandidate>>;
    fn info(&self, pkg: &str) -> Result<Option<PackageInfo>>;
    fn list_installed(&self) -> Result<Vec<InstalledPackage>>;
    fn outdated(&self) -> Result<Vec<OutdatedPackage>>;   // default empty

    // mutation specs — single source of truth
    fn install_spec(&self, pkg: &str) -> Option<CommandSpec>;
    fn remove_spec(&self, pkg: &str) -> Option<CommandSpec>;
    fn update_spec(&self) -> Option<CommandSpec>;
    fn upgrade_spec(&self) -> Option<CommandSpec>;
    fn clean_spec(&self) -> Option<CommandSpec>;
}
```

Two invariants:

- **The core never branches on backend names.** Distro knowledge lives in
  `native_distro_ids()` / `is_compatible()`; class knowledge lives in
  `BackendClass` classification.
- **Specs are exec-form only.** `install_spec`/`remove_spec` etc. return
  `CommandSpec { program, args, display }` — argv is passed directly to
  `std::process::Command`, never through a shell. The package name must be
  its own argv element (`pkg`, `channel.pkg`, or `flathub/pkg`), never spliced
  into a one-liner. A test (`mutation_specs_never_use_shell_meta_for_package`)
  enforces this across every backend.

`BackendClass` ranks how a backend relates to *this* system:

| Class | Meaning | Gate |
|---|---|---|
| `Native` | the distro's own PM (os-release `ID`/`ID_LIKE` match) | — |
| `Universal` | works anywhere (flatpak, snap, nix, guix, brew) | — |
| `Container` | PM inside Docker/Podman (`apt (docker)` …) | daemon reachable |
| `Cross` | non-native PM executing on the host | requires `--cross-distro` |

## Safety model

- **No shell interpolation.** Exec-form argv end to end; hostile package
  names are rejected by `validate_package_id` before resolution.
- **Cross-distro gate.** A non-native backend never runs directly unless the
  user passes `--cross-distro` (hidden legacy alias `--force`).
- **Read-only planning.** `plan` / `--dry` / `--explain` never execute
  mutations and are never recorded in history.
- **Timeouts.** All command execution uses `execute_with_timeout` (version
  probes 10s, mutations bounded) — a hung PM cannot hang unvrs forever.
- **Privilege transparency.** Plans state `requires_root` per action and
  render the exact `sudo …` command the user would run.
- **Actionable failures.** `CommandFailed` embeds a 400-char stderr snippet;
  permission-looking stderr gains a `sudo unvrs …` hint; unknown backends get
  Levenshtein did-you-mean; every error may carry `suggestion()`.
- **Spinner hygiene.** Only runs on a TTY and in human mode; `Drop` clears
  the line even when an error propagates via `?`, and JSON stdout stays pure.

## Configuration, profiles, history

| What | Where | Override |
|---|---|---|
| `config.toml` | `<user config>/unvrs/config.toml` | `UNVRS_CONFIG_DIR` |
| Profiles | `<user config>/unvrs/profiles/<name>.toml` | `UNVRS_CONFIG_DIR` |
| History | `<user data>/unvrs/history.json` | `UNVRS_DATA_DIR` |

- Config errors abort startup with the file path (no silent fallback to
  defaults).
- Policy: `[resolver] prefer = ["native", "flatpak", "apt"]` orders the
  resolver; `avoid = [...]` demotes backends to `avoided`.
- Profile names: `[A-Za-z0-9_-]{1,64}` — no path traversal, ever.
- History: max 500 entries; ids start at 1 (legacy `0` entries still load);
  `rollback_supported` recorded truthfully per entry.

## Output contract (JSON)

With `--json` stdout is exactly one JSON document (no spinner, no ANSI):

- mutations: `{ "success": bool, "operation": string, "actions": [...] }`
- errors: `{ "error": string, "did_you_mean"?: string, "suggestion"?: string }`
- `doctor`: `{ system, backends[], config, privileges, status, issues[] }`
- `search` / `list` / `history`: bare arrays

Exit codes: `0` success, `1` any handled failure, `2` clap usage errors.

## Testing strategy

- **Unit tests** next to the code: spec safety, resolver ordering, policy,
  history compat, executor exit mapping, doctor JSON shape, plan rendering.
- **Integration tests** (`tests/cli.rs`): spawn the real binary with
  `UNVRS_CONFIG_DIR`/`UNVRS_DATA_DIR` pointed at temp dirs — help/version,
  JSON validity, did-you-mean, injection-shaped names, profile lifecycle,
  dry-runs never recorded, no-ANSI guarantees.
- **CI** (`.github/workflows/ci.yml`): `fmt --check`, `clippy -D warnings`,
  `test` on Ubuntu + Windows, `cargo audit`, `cargo deny`, Docker build +
  smoke test.
