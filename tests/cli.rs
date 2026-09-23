//! End-to-end CLI tests. Each test runs the real binary in an isolated
//! config/data sandbox (`UNVRS_CONFIG_DIR` / `UNVRS_DATA_DIR`) so nothing on
//! the host system is read or written.

use clap::Parser;
use std::process::{Command, Output};
use tempfile::TempDir;
use unvrs::cli::{Cli, Commands, HistoryAction, PlanOperation, ProfileAction};

struct Sandbox {
    root: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().expect("create tempdir"),
        }
    }

    /// Run `unvrs <args>` with isolated config/data directories.
    fn run(&self, args: &[&str]) -> Output {
        let cfg = self.root.path().join("cfg");
        let data = self.root.path().join("data");
        Command::new(env!("CARGO_BIN_EXE_unvrs"))
            .args(args)
            .env("UNVRS_CONFIG_DIR", &cfg)
            .env("UNVRS_DATA_DIR", &data)
            .output()
            .expect("failed to spawn unvrs")
    }

    /// Run and parse stdout as JSON; panics with diagnostics on failure.
    fn run_json(&self, args: &[&str]) -> (Output, serde_json::Value) {
        let out = self.run(args);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
            panic!(
                "stdout is not valid JSON ({e});\n  stdout: {stdout}\n  stderr: {}",
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out, value)
    }
}

fn stdout_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// ---------- argument parsing (in-process) ----------

#[test]
fn parses_multi_package_install() {
    let cli = Cli::try_parse_from(["unvrs", "install", "git", "vim", "--dry"]).unwrap();
    match cli.command {
        Commands::Install {
            packages,
            dry,
            backend,
            cross_distro,
            container,
            explain,
            ..
        } => {
            assert_eq!(packages, vec!["git", "vim"]);
            assert!(dry);
            assert!(backend.is_none());
            assert!(!cross_distro);
            assert!(!container);
            assert!(!explain);
        }
        other => panic!("wrong command: {other:?}"),
    }
}

#[test]
fn parses_plan_remove_subcommand() {
    let cli = Cli::try_parse_from(["unvrs", "plan", "remove", "fish", "--backend", "apt"]).unwrap();
    match cli.command {
        Commands::Plan {
            operation: PlanOperation::Remove {
                packages, backend, ..
            },
        } => {
            assert_eq!(packages, vec!["fish"]);
            assert_eq!(backend.as_deref(), Some("apt"));
        }
        other => panic!("wrong command: {other:?}"),
    }
}

#[test]
fn parses_history_show_and_profile_actions() {
    let cli = Cli::try_parse_from(["unvrs", "history", "show", "42"]).unwrap();
    match cli.command {
        Commands::History {
            action: Some(HistoryAction::Show { id }),
        } => assert_eq!(id, 42),
        other => panic!("wrong command: {other:?}"),
    }

    let cli = Cli::try_parse_from(["unvrs", "profile", "create", "dev", "git", "vim"]).unwrap();
    match cli.command {
        Commands::Profile {
            action: ProfileAction::Create { name, packages },
        } => {
            assert_eq!(name, "dev");
            assert_eq!(packages, vec!["git", "vim"]);
        }
        other => panic!("wrong command: {other:?}"),
    }

    let cli = Cli::try_parse_from(["unvrs", "apply", "dev", "--dry", "--explain"]).unwrap();
    match cli.command {
        Commands::Apply {
            profile,
            dry,
            explain,
            ..
        } => {
            assert_eq!(profile, "dev");
            assert!(dry);
            assert!(explain);
        }
        other => panic!("wrong command: {other:?}"),
    }
}

// ---------- process-level behavior ----------

#[test]
fn help_and_version_exit_zero() {
    let sb = Sandbox::new();
    let out = sb.run(&["--help"]);
    assert!(out.status.success());
    assert!(stdout_str(&out).contains("Universal package manager CLI"));
    assert!(stdout_str(&out).contains("plan"));
    assert!(stdout_str(&out).contains("profile"));

    let out = sb.run(&["--version"]);
    assert!(out.status.success());
    assert!(stdout_str(&out).contains("unvrs"));
}

#[test]
fn doctor_json_is_well_formed() {
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "doctor"]);
    assert!(out.status.success());

    for key in [
        "system",
        "backends",
        "config",
        "privileges",
        "status",
        "issues",
    ] {
        assert!(v.get(key).is_some(), "missing key {key} in {v}");
    }
    let backends = v["backends"].as_array().expect("backends array");
    assert!(!backends.is_empty());
    let b0 = &backends[0];
    assert!(b0["name"].is_string());
    assert!(b0["capabilities"]["rollback"].as_bool().is_some());
    assert!(b0["capabilities"]["dry_run"].as_bool().is_some());

    // JSON mode must never contain ANSI escapes or spinner frames.
    let raw = stdout_str(&out);
    assert!(
        !raw.contains('\u{1b}'),
        "ANSI escape leaked into JSON stdout"
    );
}

#[test]
fn unknown_backend_json_error_carries_suggestion() {
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "install", "--backend", "fltapak", "vim"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"].as_str().unwrap().contains("unknown backend"));
    assert_eq!(v["did_you_mean"], "flatpak");
    assert!(v["suggestion"].as_str().unwrap().contains("doctor"));
}

#[test]
fn invalid_package_name_is_rejected() {
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "install", "--dry", "bad name!"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"]
        .as_str()
        .unwrap()
        .contains("invalid package name"));
    assert!(v["suggestion"]
        .as_str()
        .unwrap()
        .contains("letters, digits"));
}

#[test]
fn option_like_package_is_rejected() {
    // `-rf` would be option injection if ever passed through to a backend.
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "install", "--dry", "--", "-rf"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"]
        .as_str()
        .unwrap()
        .contains("invalid package name"));
}

#[test]
fn search_json_is_an_array() {
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "search", "zzz-unvrs-no-such-pkg-9x"]);
    assert!(out.status.success());
    assert!(v.is_array(), "expected array, got {v}");
    // stdout must be pure JSON (spinner disabled in JSON mode)
    assert!(!stdout_str(&out).contains('\u{1b}'));
}

#[test]
fn history_starts_empty_and_dry_runs_are_never_recorded() {
    let sb = Sandbox::new();

    // Dry-run (success or failure — either is fine for this host).
    let _ = sb.run(&[
        "--json",
        "install",
        "--dry",
        "definitely-not-a-real-pkg-xyz",
    ]);

    let (out, v) = sb.run_json(&["--json", "history"]);
    assert!(out.status.success());
    assert_eq!(v, serde_json::json!([]), "dry-run must not be recorded");
}

#[test]
fn history_show_missing_transaction_errors_cleanly() {
    let sb = Sandbox::new();
    let (out, v) = sb.run_json(&["--json", "history", "show", "123"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"].as_str().unwrap().contains("no transaction"));
    assert!(v.get("suggestion").is_none(), "message is self-contained");
}

#[test]
fn profile_lifecycle_in_isolated_config() {
    let sb = Sandbox::new();

    let (out, v) = sb.run_json(&["--json", "profile", "create", "ci-profile", "git", "vim"]);
    assert!(out.status.success());
    assert_eq!(v["name"], "ci-profile");
    assert_eq!(v["packages"], serde_json::json!(["git", "vim"]));

    let (out, list) = sb.run_json(&["--json", "profile", "list"]);
    assert!(out.status.success());
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(names.contains(&"ci-profile"));

    let (out, show) = sb.run_json(&["--json", "profile", "show", "ci-profile"]);
    assert!(out.status.success());
    assert_eq!(show["packages"], serde_json::json!(["git", "vim"]));

    // Duplicate create fails.
    let (out, v) = sb.run_json(&["--json", "profile", "create", "ci-profile", "x"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"].as_str().unwrap().contains("already exists"));

    // Path traversal in profile name is rejected.
    let (out, v) = sb.run_json(&["--json", "profile", "create", "../evil", "x"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"]
        .as_str()
        .unwrap()
        .contains("invalid profile name"));

    // Unknown profile gets a pointed suggestion.
    let (out, v) = sb.run_json(&["--json", "profile", "show", "missing-profile"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(v["error"].as_str().unwrap().contains("profile not found"));
    assert!(v["suggestion"].as_str().unwrap().contains("profile list"));
}

#[test]
fn no_color_output_has_no_ansi_escapes() {
    let sb = Sandbox::new();
    let out = sb.run(&["--no-color", "doctor"]);
    assert!(out.status.success());
    assert!(
        !stdout_str(&out).contains('\u{1b}'),
        "ANSI escapes present despite --no-color"
    );
}

#[test]
fn invalid_config_is_reported_with_path() {
    let sb = Sandbox::new();
    // Write a broken config into the sandbox config dir.
    let cfg_dir = sb.root.path().join("cfg");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(cfg_dir.join("config.toml"), "not [valid toml").unwrap();

    // Any command must fail fast with the path, instead of silently using
    // defaults (the old behavior).
    let out = Command::new(env!("CARGO_BIN_EXE_unvrs"))
        .args(["--json", "doctor"])
        .env("UNVRS_CONFIG_DIR", &cfg_dir)
        .env("UNVRS_DATA_DIR", sb.root.path().join("data"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    let err = v["error"].as_str().expect("error field");
    assert!(err.contains("invalid config"), "got: {err}");
    assert!(err.contains("config.toml"), "path missing in: {err}");
    assert!(v["suggestion"].as_str().unwrap().contains("Fix the file"));

    // Valid config: doctor succeeds and reports config as valid.
    std::fs::write(
        cfg_dir.join("config.toml"),
        "[policy]\nprefer = [\"native\"]\n",
    )
    .unwrap();
    let (_out, report) = {
        let out = Command::new(env!("CARGO_BIN_EXE_unvrs"))
            .args(["--json", "doctor"])
            .env("UNVRS_CONFIG_DIR", &cfg_dir)
            .env("UNVRS_DATA_DIR", sb.root.path().join("data"))
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
        (out, v)
    };
    assert_eq!(report["config"]["valid"], true);
}
