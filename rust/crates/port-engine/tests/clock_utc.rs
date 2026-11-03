// SPDX-License-Identifier: AGPL-3.0-only
//! Real Date TimeClip/toISOString and complete original page-state comparisons.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::js_date::{DateError, utc_iso_millis};
use port_engine::page_stale::{MarkError, PageStale};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/clock-utc/cases.tsv");
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn error(e: DateError) -> String {
    let (name, message) = e.source_error().expect("not a source error");
    format!(
        "{name}:{}",
        hex(&message.encode_utf16().collect::<Vec<_>>())
    )
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let ms = f64::from_bits(u64::from_str_radix(p[1], 16).unwrap());
    let direct = match utc_iso_millis(ms) {
        Ok(s) => format!("ok:{}", hex(&s)),
        Err(e) => format!("throw:{}", error(e)),
    };
    let mut page = PageStale::default();
    let mut calls = 0;
    let mut frames = Vec::new();
    for action in ["mark", "get", "epoch", "invalid", "reset", "mark"] {
        let mut failure = "-".to_string();
        if action == "reset" {
            page.reset()
        } else if action != "get" {
            if page.since().is_none() {
                calls += 1
            }
            let value = if action == "epoch" {
                0.
            } else if action == "invalid" {
                f64::NAN
            } else {
                ms
            };
            match page.mark_millis(value, 27) {
                Ok(()) => {}
                Err(MarkError::Clock(e)) => failure = error(e),
                Err(e) => panic!("host refusal {e:?}"),
            }
        }
        let since = page
            .since()
            .map(|s| String::from_utf16(s).unwrap())
            .unwrap_or_else(|| "null".to_string());
        frames.push(format!("{since},{calls},{failure}"));
    }
    format!("{}\t{direct}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn bounded_conversion_and_lazy_adapter() {
    let mut page = PageStale::default();
    assert_eq!(page.mark_millis(0., 0), Err(MarkError::TextLimit));
    assert_eq!(page.since(), None);
    assert_eq!(
        page.mark_millis(f64::INFINITY, 27),
        Err(MarkError::Clock(DateError::InvalidTime))
    );
    assert_eq!(page.since(), None);
    page.mark_millis(-0., 24).unwrap();
    let since = page.since().unwrap().to_vec();
    assert_eq!(page.mark_millis(f64::NAN, 0), Ok(()));
    assert_eq!(page.since(), Some(since.as_slice()));
    assert_eq!(DateError::Allocation.source_error(), None);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "real Date and original page state require Node22 and pinned modules"]
fn pinned_clock_utc_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-stale-detect.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/clock-utc.mjs"))
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
    assert_eq!(originals.len(), 311);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/utc-diff/run-{run}"));
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
        "311 exact Date/UTC/clock/error/state observations; {}",
        dir.display()
    );
}
