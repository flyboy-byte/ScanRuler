// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original workspace mapping and actual installed-metadata contexts.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    schema_value::Value as V,
    workspaces::{self, Error, Limits},
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/workspaces/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
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

fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn decode(s: &str) -> Vec<u16> {
    (0..s.len())
        .step_by(4)
        .map(|i| u16::from_str_radix(&s[i..i + 4], 16).unwrap())
        .collect()
}
fn limits() -> Limits {
    Limits {
        plugins: 64,
        names: 64,
        text_units: 4096,
    }
}
fn names(metadata: &[(Vec<u16>, Option<Vec<u16>>)]) -> Vec<Vec<u16>> {
    let slots: Vec<_> = metadata.iter().map(|(_, s)| s.as_deref()).collect();
    workspaces::workspace_names(&slots, limits()).unwrap()
}
fn array(names: Vec<Vec<u16>>) -> V {
    V::Array(names.into_iter().map(V::String).collect())
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mask: usize = p[1].parse().unwrap();
    let input = decode(p[2]);
    let mut metadata = Vec::new();
    // Independent input context: original baseline metadata plus data-only probe
    // plugins in their known ASCII install order. Production does not sort/install.
    if mask < 16 {
        metadata.push((text("mesh-tools"), Some(text("mesh"))));
        for i in 0..4 {
            if mask & (1 << i) != 0 {
                let workspace = match i {
                    0 => Some(text("measure")),
                    1 => Some([text("addon💩"), vec![0xd800]].concat()),
                    2 => None,
                    _ => Some(vec![]),
                };
                metadata.push((
                    text(&format!("probe-{}", char::from(b'a' + i as u8))),
                    workspace,
                ));
            }
        }
        metadata.push((text("reverse"), Some(text("reverse"))))
    }
    let installed = V::Array(
        metadata
            .iter()
            .map(|(id, workspace)| {
                V::Array(vec![
                    V::String(id.clone()),
                    workspace.as_ref().map_or(V::Null, |s| V::String(s.clone())),
                ])
            })
            .collect(),
    );
    let mut first = names(&metadata);
    first[0] = text("changed");
    first.push(text("extra"));
    let second = names(&metadata);
    if let Some((_, Some(id))) = metadata.iter_mut().find(|(_, id)| id.is_some()) {
        *id = [text("later"), vec![0xd800]].concat();
    }
    let third = names(&metadata);
    format!(
        "{}\t{}\n",
        p[0],
        [
            hex(workspaces::public_workspace(&input)),
            hex(workspaces::internal_workspace(&input)),
            wire(&installed),
            wire(&array(first)),
            wire(&array(second)),
            wire(&array(third))
        ]
        .join("|")
    )
}
#[test]
fn strict_mapping_and_bounded_copies() {
    let input = text("unknown");
    assert!(std::ptr::eq(
        workspaces::public_workspace(&input),
        input.as_slice()
    ));
    assert_eq!(
        workspaces::public_workspace(&text("elements")),
        text("measure")
    );
    assert_eq!(
        workspaces::internal_workspace(&text("measure")),
        text("elements")
    );
    assert_eq!(
        workspaces::public_workspace(&text("ELEMENTS")),
        text("ELEMENTS")
    );
    let mut l = limits();
    l.names = 3;
    assert_eq!(workspaces::workspace_names(&[], l), Err(Error::Limit));
    l = limits();
    l.plugins = 0;
    assert_eq!(workspaces::workspace_names(&[None], l), Err(Error::Limit));
    l = limits();
    l.text_units = 28;
    assert_eq!(workspaces::workspace_names(&[], l), Err(Error::Limit));
    l.text_units = 29;
    assert_eq!(workspaces::workspace_names(&[], l).unwrap().len(), 4);
    let empty: Vec<u16> = vec![];
    let repeated = text("measure");
    assert_eq!(
        workspaces::workspace_names(&[Some(&empty), None, Some(&repeated)], limits())
            .unwrap()
            .len(),
        6
    );
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
#[test]
#[ignore = "complete original workspace state requires Node22 and original graph"]
fn pinned_workspaces_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-workspaces.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/workspaces.mjs"))
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
    assert_eq!(originals.len(), 306);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/workspaces-diff/run-{run}"));
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
        "306 complete workspace mapping/installed metadata/current names observations; {}",
        dir.display()
    );
}
