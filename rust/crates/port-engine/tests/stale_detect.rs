// SPDX-License-Identifier: AGPL-3.0-only
//! Pinned source predicate observations and host-boundary checks.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::stale_import::{ImportFault, TextLimit, is_stale_import};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/stale-detect/cases.tsv");
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let text: Vec<_> = parts[2]
        .as_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|h| u16::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect();
    let fault = match parts[1] {
        "text" => ImportFault::Text(&text),
        "error" | "type-error" => ImportFault::Error(&text),
        "command" => ImportFault::CommandError,
        _ => ImportFault::Other,
    };
    format!(
        "{}\t{}\tnull\n",
        parts[0],
        usize::from(is_stale_import(fault, 4096).unwrap())
    )
}
#[test]
fn bounded_owned_text_and_category_short_circuit() {
    let text: Vec<_> = "Failed to fetch dynamically imported module"
        .encode_utf16()
        .collect();
    assert_eq!(
        is_stale_import(ImportFault::Text(&text), text.len()),
        Ok(true)
    );
    assert_eq!(
        is_stale_import(ImportFault::Error(&text), text.len() - 1),
        Err(TextLimit)
    );
    assert_eq!(is_stale_import(ImportFault::CommandError, 0), Ok(false));
    assert_eq!(is_stale_import(ImportFault::Other, 0), Ok(false));
    assert_eq!(is_stale_import(ImportFault::Text(&[]), 0), Ok(false));
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "complete pinned predicate requires Node22 and original git object"]
fn pinned_stale_detect_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-stale-detect.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/stale-detect.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let original = child.wait_with_output().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let output = String::from_utf8(original.stdout).unwrap();
    let originals: Vec<_> = output.split_inclusive('\n').collect();
    assert_eq!(originals.len(), 308);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/stale-detect-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let candidate = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &candidate.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&candidate.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    for (input, expected) in INPUT.lines().zip(originals) {
        let id = input.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        let actual = observe(input);
        for (name, data) in [
            ("input.bin", input.as_bytes()),
            ("original.bin", expected.as_bytes()),
            ("rust.bin", actual.as_bytes()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert_eq!(actual, expected, "{id}");
    }
    println!(
        "308 exact predicate/page-state observations; {}",
        dir.display()
    );
}
