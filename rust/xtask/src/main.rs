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
            "check",
        ],
        &r,
    )?;
    run(
        "python3",
        &["docs/port-plan/tools/port_plan.py", "selftest"],
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
    "pinned_bounds_comparison",
    "pinned_centroid_comparison",
    "pinned_clip_comparison",
    "pinned_curvature_gate_comparison",
    "pinned_linear_comparison",
    "pinned_normals_comparison",
    "pinned_orientation_comparison",
    "pinned_plane_comparison",
    "pinned_ransac_comparison",
    "pinned_weld_comparison",
    "pinned_original_comparison",
    "pinned_stl_dispatch_comparison",
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

fn oracle() -> Result<(), String> {
    pin()?;
    let output = Command::new("cargo")
        .args([
            "test",
            "-p",
            "port-core",
            "-p",
            "port-io",
            "--tests",
            "--",
            "--ignored",
            "--list",
        ])
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
    run(
        "cargo",
        &[
            "test",
            "-p",
            "port-core",
            "-p",
            "port-io",
            "--tests",
            "--no-fail-fast",
            "--",
            "--ignored",
            "--nocapture",
        ],
        &workspace(),
    )
}

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "oracle" => oracle(),
        "pin" => pin(),
        "plan" => plan(),
        "wasm" => wasm(),
        "env" => {
            env();
            Ok(())
        }
        _ => Err("usage: cargo xtask <ci|oracle|pin|plan|wasm|env>".to_string()),
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
