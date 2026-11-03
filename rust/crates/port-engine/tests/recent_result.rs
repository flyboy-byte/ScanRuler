// SPDX-License-Identifier: AGPL-3.0-only
//! Original/native full typed values, number bits, property order and copy identity.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    recent_result::{self, Error, Kept, Limits},
    schema_value::Value as V,
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/recent-result/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn string(i: usize) -> Vec<u16> {
    let base = match i % 10 {
        0 => text("ascii"),
        1 => vec![34, 92, 8, 12, 10, 13, 9, 0, 31],
        2 => text("漢💩"),
        3 => vec![0xd800, 120, 0xdfff],
        4 => vec![0xd83d, 0xdca9],
        5 => vec![0xd800, 0xd800, 0xdc00, 0xdc00],
        6 => vec![0x2028, 0x2029],
        7 => vec![],
        8 => (0..32).collect(),
        _ => vec![120, 0xdfff, 0xd800, 121],
    };
    base.repeat(i % 3 + 1)
}
fn model(kind: usize, arg: usize, bits: u64) -> V {
    let n = f64::from_bits(bits);
    match kind {
        0 => V::Undefined,
        1 => V::Null,
        2 => V::Boolean(arg != 0),
        3 => V::Number(n),
        4 => V::String(string(arg)),
        5 => V::Bytes(vec![0; arg]),
        6 => V::Array(vec![
            V::Undefined,
            V::Null,
            V::Boolean(true),
            V::Number(n),
            V::String(string(arg)),
            V::Bytes(vec![0; arg % 17]),
            V::Object(vec![
                (text("x"), V::Undefined),
                (text("y"), V::String(text("z"))),
            ]),
        ]),
        7 => V::Object(vec![
            (text("z"), V::Number(n)),
            (text("10"), V::String(text("ten"))),
            (text("2"), V::String(text("two"))),
            (text("01"), V::String(text("leading"))),
            (text(""), V::Undefined),
            (
                text("__proto__"),
                V::Object(vec![(text("safe"), V::Boolean(true))]),
            ),
            (text("u"), V::Undefined),
            (vec![0xd800], V::String(string(arg))),
            (text("4294967294"), V::Null),
            (text("4294967295"), V::Boolean(false)),
            (text("0"), V::Bytes(vec![0; arg % 9])),
            (
                text("a"),
                V::Array(vec![
                    V::Undefined,
                    V::Number(-0.0),
                    V::Number(f64::INFINITY),
                ]),
            ),
        ]),
        8 => {
            let mut v = V::String(text("hi"));
            for _ in 0..arg {
                v = V::Array(vec![v])
            }
            v
        }
        9 => V::String(vec![120; arg]),
        10 => V::String(match bits {
            0 => vec![1; arg],
            1 => vec![34; arg],
            2 => vec![0xd800; arg],
            _ => text("💩").repeat(arg),
        }),
        11 => V::Object(vec![(
            match bits {
                0 => text("x"),
                1 => vec![0xd800],
                2 => text("1"),
                _ => vec![34],
            },
            V::String(vec![120; arg]),
        )]),
        12 => V::Array(vec![V::Number(n), V::String(vec![120; arg])]),
        _ => panic!("unknown model"),
    }
}
fn wire(value: &V) -> String {
    fn count(out: &mut Vec<u8>, n: usize) {
        out.extend_from_slice(&u32::try_from(n).unwrap().to_le_bytes())
    }
    fn txt(out: &mut Vec<u8>, s: &[u16]) {
        count(out, s.len());
        for c in s {
            out.extend_from_slice(&c.to_le_bytes())
        }
    }
    fn walk(out: &mut Vec<u8>, v: &V) {
        match v {
            V::Undefined => out.push(0),
            V::Null => out.push(1),
            V::Boolean(b) => out.push(if *b { 3 } else { 2 }),
            V::Number(n) => {
                out.push(4);
                out.extend_from_slice(&n.to_bits().to_le_bytes())
            }
            V::String(s) => {
                out.push(5);
                txt(out, s)
            }
            V::Array(items) => {
                out.push(6);
                count(out, items.len());
                for v in items {
                    walk(out, v)
                }
            }
            V::Object(fields) => {
                out.push(7);
                count(out, fields.len());
                for (k, v) in fields {
                    txt(out, k);
                    walk(out, v)
                }
            }
            V::Bytes(b) => {
                out.push(8);
                count(out, b.len());
                out.extend_from_slice(b)
            }
        }
    }
    let mut out = Vec::new();
    walk(&mut out, value);
    out.iter().map(|n| format!("{n:02x}")).collect()
}
fn packed(kept: &Kept) -> String {
    match kept {
        Kept::Absent => "absent".into(),
        Kept::Result(v) => format!("value:{}", wire(v)),
        Kept::Omitted { .. } => format!(
            "omit:{}",
            wire(&V::String(text(&kept.omitted_message().unwrap())))
        ),
    }
}
fn mutate(v: &mut V) {
    match v {
        V::Bytes(b) => b.fill(13),
        V::Array(items) => {
            for item in items.iter_mut() {
                mutate(item)
            }
            items.push(V::Null)
        }
        V::Object(fields) => {
            for (_, v) in fields.iter_mut() {
                mutate(v)
            }
            fields.push((text("changed"), V::String(text("changed"))))
        }
        _ => {}
    }
}
fn limits() -> Limits {
    Limits {
        nodes: 4096,
        depth: 64,
        input_units: 4_000_000,
        json_units: 32_000_000,
    }
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mut input = model(
        p[1].parse().unwrap(),
        p[2].parse().unwrap(),
        u64::from_str_radix(p[3], 16).unwrap(),
    );
    let kept = recent_result::keep(&input, limits()).unwrap();
    let before = packed(&kept);
    mutate(&mut input);
    format!("{}\t{before}\t{}\n", p[0], packed(&kept))
}
#[test]
fn exact_threshold_and_host_errors_do_not_become_source_omissions() {
    assert!(matches!(
        recent_result::keep(&V::String(vec![120; 15998]), limits()).unwrap(),
        Kept::Result(_)
    ));
    assert!(matches!(
        recent_result::keep(&V::String(vec![120; 15999]), limits()).unwrap(),
        Kept::Omitted { characters: 16001 }
    ));
    let mut l = limits();
    l.json_units = 10;
    assert_eq!(
        recent_result::keep(&V::String(vec![120; 20]), l),
        Err(Error::Limit)
    );
    let duplicate = V::Object(vec![(text("x"), V::Null), (text("x"), V::Null)]);
    assert_eq!(
        recent_result::keep(&duplicate, limits()),
        Err(Error::DuplicateKey)
    );
    l = limits();
    l.nodes = 1;
    assert_eq!(
        recent_result::keep(&V::Object(vec![(text("x"), V::Undefined)]), l),
        Err(Error::Limit)
    );
    l = limits();
    l.input_units = 0;
    assert_eq!(
        recent_result::keep(&V::String(vec![120]), l),
        Err(Error::Limit)
    );
    let mut v = V::Null;
    for _ in 0..129 {
        v = V::Array(vec![v])
    }
    l = limits();
    l.depth = usize::MAX;
    assert_eq!(recent_result::keep(&v, l), Err(Error::Limit));
}
#[test]
fn full_independent_data_models_execute() {
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
#[test]
#[ignore = "complete original retained owned data requires Node22 and original graph"]
fn pinned_recent_result_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-recent-result.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/recent-result.mjs"))
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
    assert_eq!(originals.len(), 413);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/recent-result-diff/run-{run}"));
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
        "413 complete retained data/type/length/copy observations; {}",
        dir.display()
    );
}
