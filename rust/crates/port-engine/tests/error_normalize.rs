// SPDX-License-Identifier: AGPL-3.0-only
//! Pinned complete registry normalization at owned host boundaries.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    error::{CommandError, CommandErrorCode, EnvelopeError},
    error_normalize::{Caught, NormalizeError, normalize},
    page_stale::PageStale,
};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/error-normalize/cases.tsv");
fn hex(text: &[u16]) -> String {
    text.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let kind = p[3];
    let text: Vec<_> = p[4]
        .as_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|h| u16::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect();
    let mut page = PageStale::default();
    if p[1] == "1" {
        page.mark(64, || {
            Ok::<_, &str>("1999-12-31T23:59:59.999Z".encode_utf16().collect())
        })
        .unwrap()
    }
    let typed = kind.strip_prefix("command-").map(|n| {
        Arc::new(
            CommandError::from_utf16(
                CommandErrorCode::ALL[n.parse::<usize>().unwrap()],
                &text,
                4096,
            )
            .unwrap(),
        )
    });
    let caught = if let Some(e) = &typed {
        Caught::Command(e.clone())
    } else {
        match kind {
            "text" => Caught::Text(&text),
            "error" => Caught::Error(&text),
            _ => Caught::Other,
        }
    };
    let log = RefCell::new(Vec::new());
    let result = normalize(
        &mut page,
        caught,
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
            if kind == "coerce-fault" {
                log.borrow_mut().push("coerce");
                return Err("coercion-fault");
            }
            if kind == "object" {
                log.borrow_mut().push("coerce")
            }
            if kind == "array" {
                let mut converted = text.clone();
                converted.extend([44, 44]);
                return Ok(converted);
            }
            Ok(text.clone())
        },
    );
    let result = match result {
        Ok(e) => format!(
            "{},{},{},{}",
            e.name(),
            e.code().as_str(),
            hex(e.message_units()),
            usize::from(typed.as_ref().is_some_and(|t| Arc::ptr_eq(t, &e)))
        ),
        Err(NormalizeError::Clock(e) | NormalizeError::Message(e)) => {
            format!("throw,{}", hex(&e.encode_utf16().collect::<Vec<_>>()))
        }
        Err(e) => panic!("host error {e:?}"),
    };
    let since = page
        .since()
        .map(|s| String::from_utf16(s).unwrap())
        .unwrap_or_else(|| "null".to_string());
    format!("{}\t{since}\t{}\t{result}\n", p[0], log.borrow().join(","))
}
#[test]
fn typed_identity_bypasses_limits_and_callbacks_and_other_text_cannot_mark() {
    let mut page = PageStale::default();
    let original =
        Arc::new(CommandError::from_utf16(CommandErrorCode::Busy, &[0xd800], 1).unwrap());
    let result = normalize(
        &mut page,
        Caught::Command(original.clone()),
        0,
        0,
        || -> Result<Vec<u16>, ()> { panic!("typed clock") },
        || panic!("typed string"),
    )
    .unwrap();
    assert!(Arc::ptr_eq(&result, &original));
    assert_eq!(page.since(), None);
    let text: Vec<_> = "Unable to preload CSS".encode_utf16().collect();
    let result = normalize(
        &mut page,
        Caught::Other,
        0,
        4096,
        || -> Result<Vec<u16>, ()> { panic!("other clock") },
        || Ok(text.clone()),
    )
    .unwrap();
    assert_eq!(result.code(), CommandErrorCode::Failed);
    assert_eq!(page.since(), None);
    let result = normalize(
        &mut page,
        Caught::Text(&text),
        0,
        4096,
        || Ok::<_, ()>(vec![65]),
        || panic!("text conversion"),
    );
    assert_eq!(
        result,
        Err(NormalizeError::Host(EnvelopeError::MessageLimit))
    );
    assert_eq!(page.since(), None);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
fn opaque_non_clone_conversion_fault_is_preserved() {
    #[derive(Debug, PartialEq, Eq)]
    struct Fault(u16);
    let mut page = PageStale::default();
    let result = normalize(
        &mut page,
        Caught::Other,
        0,
        0,
        || -> Result<Vec<u16>, Fault> { panic!("ordinary fault has no clock") },
        || Err(Fault(0xd800)),
    );
    assert_eq!(result, Err(NormalizeError::Message(Fault(0xd800))));
    assert_eq!(page.since(), None);
}
#[test]
#[ignore = "complete pinned normalization requires Node22 and original graph"]
fn pinned_error_normalize_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-normalize.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/error-normalize.mjs"))
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
    assert_eq!(originals.len(), 304);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/normalize-diff/run-{run}"));
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
        "304 exact normalized identity/envelope/state observations; {}",
        dir.display()
    );
}
