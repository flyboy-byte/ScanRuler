// SPDX-License-Identifier: AGPL-3.0-only
//! Full pinned staleError body observations at fallible owned host boundaries.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    error::EnvelopeError,
    page_stale::{MarkError, PageStale},
    stale_error::{StaleError, stale_error},
};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/stale-error/cases.tsv");
fn hex(text: &[u16]) -> String {
    text.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let kind = p[3];
    let mut text: Vec<_> = p[4]
        .as_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|h| u16::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect();
    let mut state = PageStale::default();
    if p[1] == "1" {
        state
            .mark(64, || {
                Ok::<_, &str>("1999-12-31T23:59:59.999Z".encode_utf16().collect())
            })
            .unwrap()
    }
    let log = RefCell::new(Vec::new());
    let result = stale_error(
        &mut state,
        64,
        4096,
        || {
            log.borrow_mut().push("clock");
            if p[2] == "1" {
                Err("clock-fault")
            } else {
                Ok(p[5].encode_utf16().collect())
            }
        },
        || {
            match kind {
                "coerce-fault" => {
                    log.borrow_mut().push("coerce");
                    return Err("coercion-fault");
                }
                "getter-fault" => {
                    log.borrow_mut().push("message");
                    return Err("message-fault");
                }
                "object" => log.borrow_mut().push("coerce"),
                "getter" => log.borrow_mut().push("message"),
                "null" => text = "null".encode_utf16().collect(),
                "undefined" => text = "undefined".encode_utf16().collect(),
                "array" => text.extend([44, 44]),
                "text" | "error" | "command" => {}
                _ => panic!("kind"),
            };
            Ok(text)
        },
    );
    let result = match result {
        Ok(e) => format!(
            "{},{},{}",
            e.name(),
            e.code().as_str(),
            hex(e.message_units())
        ),
        Err(StaleError::Mark(MarkError::Clock(e)) | StaleError::Message(e)) => {
            format!("throw,{}", hex(&e.encode_utf16().collect::<Vec<_>>()))
        }
        Err(e) => panic!("unexpected host error {e:?}"),
    };
    let since = state
        .since()
        .map(|s| String::from_utf16(s).unwrap())
        .unwrap_or_else(|| "null".to_string());
    format!("{}\t{since}\t{}\t{result}\n", p[0], log.borrow().join(","))
}
#[test]
fn host_refusal_keeps_mark_and_clock_failure_skips_message() {
    let mut page = PageStale::default();
    let result = stale_error(
        &mut page,
        0,
        0,
        || Ok::<_, Vec<u16>>(vec![65]),
        || panic!("clock limit skips message"),
    );
    assert_eq!(result, Err(StaleError::Mark(MarkError::TextLimit)));
    assert_eq!(page.since(), None);
    let opaque = vec![0xd800];
    let result = stale_error(
        &mut page,
        64,
        64,
        || Err(opaque.clone()),
        || panic!("clock error skips message"),
    );
    assert_eq!(result, Err(StaleError::Mark(MarkError::Clock(opaque))));
    assert_eq!(page.since(), None);
    let result = stale_error(&mut page, 1, 0, || Ok::<_, ()>(vec![65]), || Ok(vec![]));
    assert_eq!(
        result,
        Err(StaleError::Envelope(EnvelopeError::MessageLimit))
    );
    assert_eq!(page.since(), Some([65].as_slice()));
    let result = stale_error(
        &mut page,
        0,
        4096,
        || -> Result<Vec<u16>, Vec<u16>> { panic!("marked") },
        || Err(vec![0xdfff]),
    );
    assert_eq!(result, Err(StaleError::Message(vec![0xdfff])));
    assert_eq!(page.since(), Some([65].as_slice()));
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "complete pinned envelope trace requires Node22 and original git object"]
fn pinned_stale_error_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-stale-detect.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/stale-error.mjs"))
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
    assert_eq!(originals.len(), 120);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/stale-error-diff/run-{run}"));
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
        "120 exact envelope/clock/coercion/state observations; {}",
        dir.display()
    );
}
