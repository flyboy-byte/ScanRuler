// SPDX-License-Identifier: AGPL-3.0-only
//! Project tasks: `ci` is the fast gate; `oracle` is the native source-comparison gate.
//!
//! The plan checks are still the temporary Python bootstrap (`docs/port-plan/tools`); they
//! move into this crate under P39. `oracle` discovers the required native suites before
//! running them; the general differential report/ratchet commands remain unbuilt.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn root() -> PathBuf {
    // rust/xtask -> repository root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn run(program: &str, args: &[&str], dir: &Path) -> Result<(), String> {
    println!("\n$ {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| format!("could not start {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} {} failed ({status})", args.join(" ")))
    }
}

fn capture(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "missing".to_string())
}

fn pin() -> Result<(), String> {
    let path = root().join("docs/port-plan/reference-pin.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    for (key, want) in [
        ("commit", port_core::REFERENCE_SHA),
        ("app_version", port_core::REFERENCE_APP_VERSION),
    ] {
        let needle = format!("\"{key}\": \"{want}\"");
        if !text.contains(&needle) {
            return Err(format!(
                "reference-pin.json does not say {needle}; port-core and the pin disagree"
            ));
        }
    }
    println!(
        "reference pin agrees with port-core ({})",
        port_core::REFERENCE_SHA
    );
    Ok(())
}

fn plan() -> Result<(), String> {
    let r = root();
    run(
        "python3",
        &[
            "docs/port-plan/tools/port_plan.py",
            "--root",
            "docs/port-plan",
            "check-generated",
        ],
        &r,
    )?;
    run(
        "python3",
        &["docs/port-plan/tools/port_plan.py", "selftest"],
        &r,
    )?;
    run(
        "python3",
        &["docs/port-plan/tools/ci_report.py", "status", "--check"],
        &r,
    )?;
    run(
        "python3",
        &[
            "-m",
            "unittest",
            "discover",
            "-s",
            "docs/port-plan/tools",
            "-p",
            "test_*.py",
        ],
        &r,
    )
}

fn env() {
    println!("rustc  {}", capture("rustc", &["-V"]));
    println!("cargo  {}", capture("cargo", &["-V"]));
    println!("node   {}", capture("node", &["-v"]));
    println!("python {}", capture("python3", &["-V"]));
    println!("git    {}", capture("git", &["rev-parse", "HEAD"]));
    println!("os     {}", capture("uname", &["-sr"]));
    println!(
        "wasm32 {}",
        if capture("rustc", &["--print", "target-list"]).contains("wasm32-unknown-unknown") {
            "target known"
        } else {
            "missing"
        }
    );
}

fn workspace() -> PathBuf {
    root().join("rust")
}

fn ci() -> Result<(), String> {
    let w = workspace();
    pin()?;
    plan()?;
    run("cargo", &["fmt", "--all", "--check"], &w)?;
    run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &w,
    )?;
    run("cargo", &["test", "--workspace"], &w)?;
    wasm()
}

fn wasm() -> Result<(), String> {
    run(
        "cargo",
        &[
            "check",
            "-p",
            "port-core",
            "-p",
            "port-engine",
            "-p",
            "port-io",
            "--target",
            "wasm32-unknown-unknown",
        ],
        &workspace(),
    )
}

// Existing source-comparison entrypoints. Removing/renaming one requires explicit
// coverage review; an empty or filtered Cargo run must never appear successful.
const ORACLE_TESTS: &[&str] = &[
    "closed_shells::tests::pinned_closed_shells_comparison",
    "pinned_adjacency_comparison",
    "pinned_bounds_comparison",
    "pinned_centroid_comparison",
    "pinned_clip_comparison",
    "pinned_components_comparison",
    "pinned_curvature_gate_comparison",
    "pinned_linear_comparison",
    "pinned_mesh_selection_comparison",
    "pinned_mesh_graph_comparison",
    "pinned_marked_plane_comparison",
    "pinned_normals_comparison",
    "pinned_orientation_comparison",
    "pinned_plane_comparison",
    "pinned_ransac_comparison",
    "pinned_weld_comparison",
    "pinned_original_comparison",
    "pinned_stl_dispatch_comparison",
    "pinned_ply_ascii_comparison",
    "pinned_ply_binary_comparison",
    "pinned_engine_error_comparison",
    "pinned_obj_texture_comparison",
    "pinned_obj_mesh_comparison",
    "pinned_obj_files_comparison",
    "pinned_drop_files_comparison",
    "pinned_schema_comparison",
    "pinned_registry_comparison",
    "pinned_history_record_comparison",
    "pinned_history_local_comparison",
    "pinned_history_guards_comparison",
    "pinned_history_restore_comparison",
    "pinned_history_core_comparison",
    "pinned_import_queue_comparison",
    "pinned_command_deadline_comparison",
    "pinned_settle_run_comparison",
];

fn validate_oracle_discovery(list: &str) -> Result<(), String> {
    let wanted: std::collections::BTreeSet<_> = ORACLE_TESTS.iter().copied().collect();
    if wanted.len() != ORACLE_TESTS.len() {
        return Err("duplicate configured oracle entrypoint".into());
    }
    let mut found = std::collections::BTreeSet::new();
    for line in list.lines() {
        let Some(name) = line.trim().strip_suffix(": test") else {
            continue;
        };
        if wanted.contains(name) && !found.insert(name) {
            return Err(format!("duplicate oracle entrypoint: {name}"));
        }
    }
    let missing: Vec<_> = wanted.difference(&found).copied().collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing native oracle tests: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

/// The original app needs Node 22 or newer. The system Node may be older, so the oracle
/// gate re-runs itself once under `fnm exec --using=22` (a user-space install, nothing
/// global changes) and otherwise fails with the reason. Node is oracle-only.
fn ensure_oracle_node(task_args: &[String]) -> Result<bool, String> {
    let version = capture("node", &["-v"]);
    let major = version
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok());
    if major.is_some_and(|m| m >= 22) {
        return Ok(false);
    }
    if std::env::var_os("SCANRULER_ORACLE_REEXEC").is_some() {
        return Err(format!(
            "Node {version} is below 22 even under fnm; run `fnm install 22`"
        ));
    }
    let status = Command::new("fnm")
        .args(["exec", "--using=22", "--", "cargo", "xtask"])
        .args(task_args)
        .env("SCANRULER_ORACLE_REEXEC", "1")
        .current_dir(workspace())
        .status()
        .map_err(|e| {
            format!("Node {version} is below 22 and fnm could not start ({e}); install Node 22 (e.g. `fnm install 22`)")
        })?;
    if status.success() {
        Ok(true)
    } else {
        Err("oracle failed under Node 22 (see output above)".into())
    }
}

// Discovery and execution must inspect the same owned crates. In particular,
// registering an engine suite cannot leave it invisible to either invocation.
fn oracle_args() -> Vec<&'static str> {
    vec![
        "test",
        "-p",
        "port-core",
        "-p",
        "port-io",
        "-p",
        "port-engine",
        "--tests",
    ]
}

fn oracle() -> Result<(), String> {
    if ensure_oracle_node(&["oracle".to_string()])? {
        return Ok(());
    }
    pin()?;
    let output = Command::new("cargo")
        .args(oracle_args())
        .args(["--", "--ignored", "--list"])
        .current_dir(workspace())
        .output()
        .map_err(|e| format!("oracle discovery could not start: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "oracle discovery failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    validate_oracle_discovery(&String::from_utf8_lossy(&output.stdout))?;
    println!(
        "all {} required native oracle entrypoints discovered",
        ORACLE_TESTS.len()
    );
    env();
    let mut args = oracle_args();
    args.extend(["--no-fail-fast", "--", "--ignored", "--nocapture"]);
    run("cargo", &args, &workspace())
}

const FUZZ_USAGE: &str = "usage: cargo xtask fuzz <ply|stl> [--seed N] [--cases N] [--out DIR] [--selfcheck] [--minimize]";

/// Standing seeded mutation fuzz of a parser against the frozen original. `--selfcheck`
/// deliberately corrupts the Rust output and requires the comparison to report divergences.
fn fuzz(args: &[String]) -> Result<(), String> {
    let unit = args.first().ok_or(FUZZ_USAGE)?;
    if unit != "ply" && unit != "stl" {
        return Err(format!("unknown parser unit {unit:?}; {FUZZ_USAGE}"));
    }
    let mut seed = "1".to_string();
    let mut cases = "500".to_string();
    let mut out = None;
    let mut selfcheck = false;
    let mut minimize = false;
    let mut rest = args[1..].iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--seed" => seed = rest.next().ok_or(FUZZ_USAGE)?.clone(),
            "--cases" => cases = rest.next().ok_or(FUZZ_USAGE)?.clone(),
            "--out" => out = Some(rest.next().ok_or(FUZZ_USAGE)?.clone()),
            "--selfcheck" => selfcheck = true,
            "--minimize" => minimize = true,
            other => return Err(format!("unknown option {other:?}; {FUZZ_USAGE}")),
        }
    }
    for (name, value) in [("seed", &seed), ("cases", &cases)] {
        if value.parse::<u64>().is_err() {
            return Err(format!("--{name} must be a non-negative integer"));
        }
    }
    let mut task = vec!["fuzz".to_string()];
    task.extend(args.iter().cloned());
    if ensure_oracle_node(&task)? {
        return Ok(());
    }
    pin()?;
    let mut command = Command::new("cargo");
    command
        .args([
            "test",
            "-p",
            "port-io",
            "--test",
            "fuzz_standing",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("SCANRULER_FUZZ_UNIT", unit)
        .env("SCANRULER_FUZZ_SEED", &seed)
        .env("SCANRULER_FUZZ_CASES", &cases)
        .current_dir(workspace());
    if let Some(dir) = out {
        command.env("SCANRULER_FUZZ_OUT", dir);
    }
    if minimize {
        command.env("SCANRULER_FUZZ_MINIMIZE", "1");
    }
    if selfcheck {
        command.env("SCANRULER_FUZZ_SABOTAGE", "1");
    }
    println!(
        "\n$ fuzz {unit} seed={seed} cases={cases}{}",
        if selfcheck { " (selfcheck)" } else { "" }
    );
    let status = command
        .status()
        .map_err(|e| format!("could not start cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("fuzz {unit} failed ({status})"))
    }
}

fn wasm_probe() -> Result<(), String> {
    let w = workspace();
    run(
        "cargo",
        &[
            "rustc",
            "-p",
            "port-wasm-probe",
            "--target",
            "wasm32-unknown-unknown",
            "--lib",
            "--",
            "-C",
            "link-dead-code=yes",
            "-C",
            "link-arg=--export-all",
            "-C",
            "symbol-mangling-version=v0",
            "-C",
            "link-arg=--max-memory=67108864",
        ],
        &w,
    )?;
    let output = w.join("target/wasm32-unknown-unknown/debug/port_wasm_probe");
    std::fs::write(
        output.with_extension("candidate.txt"),
        capture("git", &["rev-parse", "HEAD"]),
    )
    .map_err(|e| format!("recording Wasm candidate: {e}"))?;
    std::fs::write(
        output.with_extension("rustc.txt"),
        capture("rustc", &["-V"]),
    )
    .map_err(|e| format!("recording Wasm compiler: {e}"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().cloned().unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "oracle" => oracle(),
        "pin" => pin(),
        "plan" => plan(),
        "wasm" => wasm(),
        "wasm-probe" => wasm_probe(),
        "fuzz" => fuzz(&args[1..]),
        "env" => {
            env();
            Ok(())
        }
        _ => Err("usage: cargo xtask <ci|oracle|fuzz|pin|plan|wasm|wasm-probe|env>".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod oracle_tests {
    use super::*;
    fn complete() -> String {
        ORACLE_TESTS
            .iter()
            .map(|name| format!("{name}: test\n"))
            .collect()
    }
    #[test]
    fn accepts_full_roster_with_other_output() {
        assert!(
            validate_oracle_discovery(&(complete() + "other: test\n0 tests, 0 benchmarks\n"))
                .is_ok()
        );
    }
    #[test]
    fn rejects_empty_and_unrelated_output() {
        assert!(validate_oracle_discovery("").is_err());
        assert!(validate_oracle_discovery("other: test\n").is_err());
    }
    #[test]
    fn rejects_each_missing_entrypoint() {
        for name in ORACLE_TESTS {
            assert!(
                validate_oracle_discovery(&complete().replace(&format!("{name}: test\n"), ""))
                    .is_err()
            );
        }
    }
    #[test]
    fn rejects_duplicate_entrypoint() {
        assert!(
            validate_oracle_discovery(&(complete() + &format!("{}: test\n", ORACLE_TESTS[0])))
                .is_err()
        );
    }
}
