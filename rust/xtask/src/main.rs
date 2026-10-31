// SPDX-License-Identifier: AGPL-3.0-only
//! Project tasks: `cargo xtask <task>`. The product gate is `cargo xtask ci`.
//!
//! The plan checks are still the temporary Python bootstrap (`docs/port-plan/tools`); they
//! move into this crate under P39. Differential suites against the original are added to
//! `ci` as they exist (CI-AND-REGRESSIONS.md), and `ci` must fail, not skip, when a
//! prerequisite is missing.

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

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "pin" => pin(),
        "plan" => plan(),
        "wasm" => wasm(),
        "env" => {
            env();
            Ok(())
        }
        _ => Err("usage: cargo xtask <ci|pin|plan|wasm|env>".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
