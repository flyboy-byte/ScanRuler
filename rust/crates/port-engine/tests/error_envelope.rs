// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original error fields/JSON, with owned-storage and budget regressions.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use port_engine::error::{CommandError, CommandErrorCode, EnvelopeError};

struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/error_cases.rs")
}
fn read_case(c: &Case) -> CommandError {
    let units: Vec<u16> = c.input[5..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    assert_eq!(
        u32::from_le_bytes(c.input[1..5].try_into().unwrap()) as usize,
        units.len()
    );
    CommandError::from_utf16(CommandErrorCode::ALL[usize::from(c.input[0])], &units, 4096).unwrap()
}
fn utf16_bytes(units: &[u16]) -> Vec<u8> {
    units.iter().flat_map(|u| u.to_le_bytes()).collect()
}
fn payload(e: &CommandError) -> Vec<u8> {
    let fields = [
        e.code().as_str().as_bytes().to_vec(),
        utf16_bytes(&e.name().encode_utf16().collect::<Vec<_>>()),
        utf16_bytes(e.message_units()),
        utf16_bytes(&e.display_units().unwrap()),
        e.to_json().unwrap().into_bytes(),
    ];
    let mut b = b"ERR1".to_vec();
    for f in fields {
        b.extend((f.len() as u32).to_le_bytes());
        b.extend(f);
    }
    b
}

#[test]
fn error_messages_are_owned_and_limits_do_not_publish_partial_values() {
    let mut input = vec![0xd800, 0, 0xdc00, 0xd83e, 0xdd80];
    let error = CommandError::from_utf16(CommandErrorCode::Busy, &input, input.len()).unwrap();
    input.fill(65);
    assert_eq!(error.message_units(), &[0xd800, 0, 0xdc00, 0xd83e, 0xdd80]);
    assert_eq!(
        error.to_json().unwrap(),
        "{\"code\":\"busy\",\"message\":\"\\ud800\\u0000\\udc00🦀\"}"
    );
    assert_eq!(
        CommandError::from_utf16(CommandErrorCode::Busy, &input, input.len() - 1),
        Err(EnvelopeError::MessageLimit)
    );
    assert_eq!(
        CommandError::from_utf8(CommandErrorCode::Failed, "🦀", 1),
        Err(EnvelopeError::MessageLimit)
    );
    assert_eq!(
        CommandError::from_utf8(CommandErrorCode::Timeout, "", 0)
            .unwrap()
            .to_string(),
        "CommandError"
    );
    assert_eq!(
        CommandError::from_utf8(CommandErrorCode::Timeout, "partial change", 20)
            .unwrap()
            .to_string(),
        "CommandError: partial change"
    );
    for c in cases() {
        assert!(payload(&read_case(&c)).starts_with(b"ERR1"));
    }
}

#[test]
#[ignore = "live original command-envelope comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_engine_error_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/engine-error-diff")
        .join(format!("run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(run.join("candidate.txt"), sha.stdout).unwrap();
    let mut identity = String::from(
        "contract: engine.error.envelope.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: code/name/message/display/JSON exact fields, UTF16 preserved\nseeds: 1,7,99,1039\n",
    );
    for (label, program, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let r = Command::new(program)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(r.status.success());
        identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&r.stdout).trim()
        ));
    }
    std::fs::write(run.join("identity.txt"), identity).unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 52);
    for c in cases {
        let candidate = payload(&read_case(&c));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/engine-errors.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(c.input).unwrap();
        let original = child.wait_with_output().unwrap();
        let d = run.join(c.id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", candidate.as_slice()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(d.join(name), b).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(
            candidate, original.stdout,
            "{}: complete error state/JSON",
            c.id
        );
        if c.id == "code-00-unicode" {
            let mut changed = candidate.clone();
            changed[8] ^= 1;
            assert_ne!(changed, original.stdout, "a changed code byte must fail");
            assert_ne!(
                &candidate[..candidate.len() - 1],
                original.stdout,
                "missing JSON data must fail"
            );
        }
        println!("{}: all original fields/JSON compared exactly", c.id);
    }
    println!(
        "52 complete constructor/serialization cases; raw {}",
        run.display()
    );
}
