// SPDX-License-Identifier: AGPL-3.0-only
//! Pinned first-timestamp and reset traces, with fallible host clock sampling.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::page_stale::{MarkError, PageStale};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/stale-clock/cases.tsv");
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let fail_at: usize = p[2].parse().unwrap();
    let stamps: Vec<_> = p[3].split(',').collect();
    let mut state = PageStale::default();
    let mut calls = 0;
    let mut frames = Vec::new();
    for action in p[1].split(',') {
        let error = match action {
            "get" => "-",
            "reset" => {
                state.reset();
                "-"
            }
            "mark" => {
                let result = state.mark(64, || {
                    let stamp = stamps[calls % stamps.len()];
                    calls += 1;
                    if calls == fail_at {
                        Err("clock-fault")
                    } else {
                        Ok(stamp.encode_utf16().collect())
                    }
                });
                match result {
                    Ok(()) => "-",
                    Err(MarkError::Clock(e)) => e,
                    Err(MarkError::TextLimit) => panic!("host limit"),
                }
            }
            _ => panic!("unknown action"),
        };
        let since = state
            .since()
            .map(|s| String::from_utf16(s).unwrap())
            .unwrap_or_else(|| "null".to_string());
        frames.push(format!("{since},{calls},{error}"));
    }
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn failed_sampling_does_not_mark_and_marked_state_does_not_sample() {
    let mut state = PageStale::default();
    assert_eq!(
        state.mark(0, || Ok::<_, Vec<u16>>(vec![65])),
        Err(MarkError::TextLimit)
    );
    assert_eq!(state.since(), None);
    let opaque = vec![0xd800];
    assert_eq!(
        state.mark(8, || Err(opaque.clone())),
        Err(MarkError::Clock(opaque))
    );
    assert_eq!(state.since(), None);
    assert_eq!(state.mark(2, || Ok::<_, ()>(vec![65, 0xdfff])), Ok(()));
    assert_eq!(
        state.mark(0, || -> Result<Vec<u16>, ()> { panic!("already marked") }),
        Ok(())
    );
    assert_eq!(state.since(), Some([65, 0xdfff].as_slice()));
    state.reset();
    assert_eq!(state.since(), None);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "complete pinned clock trace requires Node22 and original git object"]
fn pinned_stale_clock_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-stale-detect.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/stale-clock.mjs"))
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
    assert_eq!(originals.len(), 64);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/stale-clock-diff/run-{run}"));
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
        "64 exact timestamp/clock/error observations; {}",
        dir.display()
    );
}
