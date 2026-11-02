// SPDX-License-Identifier: AGPL-3.0-only
//! Full source errors, coerced values, weight and source faults.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::schema::SchemaError;
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/schema_cases.rs")
}
include!("support/schema_wire.rs");
#[test]
fn host_limits_are_separate_and_input_is_owned() {
    for c in cases() {
        let (s, v, p) = decode(c.input);
        assert!(validate(&s, &v, &p, Limits::default()).is_ok(), "{}", c.id);
    }
    let c = cases().into_iter().find(|c| c.id == "edge-007").unwrap();
    let (s, v, p) = decode(c.input);
    let before = v.clone();
    let r = validate(&s, &v, &p, Limits::default()).unwrap();
    assert_eq!(v, before);
    assert_ne!(r.value, before);
    for limits in [
        Limits {
            nodes: 0,
            ..Limits::default()
        },
        Limits {
            units: 0,
            ..Limits::default()
        },
        Limits {
            depth: 0,
            ..Limits::default()
        },
        Limits {
            work: 0,
            ..Limits::default()
        },
        Limits {
            output: 0,
            ..Limits::default()
        },
    ] {
        assert_eq!(validate(&s, &v, &p, limits), Err(SchemaError::Limit));
    }
    let bad = Value::Object(vec![
        (vec![120], Value::Null),
        (vec![120], Value::Boolean(true)),
    ]);
    assert_eq!(
        validate(&s, &bad, &p, Limits::default()),
        Err(SchemaError::InvalidData)
    );
}
#[test]
#[ignore = "live original schema-validator comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_schema_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/schema-diff")
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
        "contract: engine.schema.validate.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: errors/coerced value/schema weight/budget/fault exact UTF16 fields\nseeds: 1,7,99,1039\n",
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
    assert_eq!(cases.len(), 697);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/schema.cjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Read stdout while writing: large outputs cannot deadlock the pipe.
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        stdin.write_all(&input).unwrap();
    });
    let original = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let mut offset = 0;
    for c in cases {
        let candidate = payload(c.input);
        let len =
            u32::from_le_bytes(original.stdout[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let observed = &original.stdout[offset..offset + len];
        offset += len;
        let d = run.join(c.id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", observed),
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
        assert!(
            original.stderr.is_empty(),
            "{}: unexpected source diagnostics",
            c.id
        );
        assert_eq!(
            candidate, observed,
            "{}: complete errors/value/weight",
            c.id
        );
        if c.id == "type-number-text" {
            let mut changed = candidate.clone();
            changed[16] ^= 1;
            assert_ne!(
                changed, observed,
                "a deliberately changed error line must fail"
            );
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing JSON data must fail"
            );
        }
        println!(
            "{}: all original errors/coercions/weight compared exactly",
            c.id
        );
    }
    assert_eq!(offset, original.stdout.len());
    println!(
        "697 complete validator/coercion/weight cases; raw {}",
        run.display()
    );
}

#[test]
fn exponential_oneof_work_and_invalid_schema_refuse_without_partial_state() {
    use port_engine::schema_value::Value::*;
    let key = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    let mut schema = Object(vec![(key("type"), Number(1.))]);
    assert_eq!(
        validate(&schema, &Null, &key("input"), Limits::default()),
        Err(SchemaError::InvalidData)
    );
    schema = Object(vec![(key("type"), String(key("number")))]);
    for _ in 0..9 {
        schema = Object(vec![(key("oneOf"), Array(vec![schema.clone(), schema]))]);
    }
    assert_eq!(
        validate(
            &schema,
            &String(key("true")),
            &key("input"),
            Limits {
                work: 50,
                ..Limits::default()
            }
        ),
        Err(SchemaError::Limit)
    );
}
