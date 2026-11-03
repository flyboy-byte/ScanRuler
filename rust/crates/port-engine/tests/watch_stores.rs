// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original source-array identities and conditional cleanup states.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::watch_stores::{StoreList, StoreSources};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/watch-stores/cases.tsv");
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let a: usize = p[1].parse().unwrap();
    let b: usize = p[2].parse().unwrap();
    let mode: usize = p[3].parse().unwrap();
    let stores: Vec<_> = (0..3).map(Rc::new).collect();
    let arrays: Vec<StoreList<_>> = vec![
        Rc::from(vec![]),
        Rc::from(vec![stores[0].clone(), stores[1].clone()]),
        Rc::from(vec![
            stores[1].clone(),
            stores[0].clone(),
            stores[1].clone(),
        ]),
        Rc::from(vec![stores[0].clone(), stores[1].clone()]),
    ];
    let source = StoreSources::new();
    let mut seen = vec![source.current()];
    seen.extend(arrays.iter().cloned());
    let mut resets = Vec::new();
    let mut frames = Vec::new();
    let frame = |frames: &mut Vec<String>, seen: &mut Vec<StoreList<_>>| {
        let current = source.current();
        let index = seen
            .iter()
            .position(|s| Rc::ptr_eq(s, &current))
            .unwrap_or_else(|| {
                let index = seen.len();
                seen.push(current.clone());
                index
            });
        let members = current
            .iter()
            .map(|s| {
                stores
                    .iter()
                    .position(|original| Rc::ptr_eq(s, original))
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");
        frames.push(format!("{index}|{members}|0"));
    };
    let programs: Vec<Vec<&str>> = vec![
        vec!["get", "a", "r0", "b", "r0", "r1", "get"],
        vec!["a", "b", "r0", "get", "r1"],
        vec!["a", "d0", "get", "b", "d1", "get"],
        vec!["a", "r0", "r0", "get", "a", "r0", "get"],
        vec!["a", "b", "a", "r0", "get", "r1", "get", "r2"],
    ];
    frame(&mut frames, &mut seen);
    for op in &programs[mode] {
        if *op == "a" || *op == "b" {
            resets.push(Some(
                source.configure(arrays[if *op == "a" { a } else { b }].clone()),
            ))
        } else if let Some(i) = op.strip_prefix('r') {
            if let Some(reset) = &resets[i.parse::<usize>().unwrap()] {
                reset.release()
            }
        } else if let Some(i) = op.strip_prefix('d') {
            resets[i.parse::<usize>().unwrap()] = None
        }
        frame(&mut frames, &mut seen);
    }
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn opaque_borrowed_unsized_stores_and_no_drop_cleanup() {
    let text = String::from("borrowed");
    let store: Rc<dyn std::fmt::Debug + '_> = Rc::new(&text);
    let array: StoreList<dyn std::fmt::Debug + '_> = Rc::from(vec![store]);
    let source = StoreSources::new();
    let old = source.configure(array.clone());
    drop(source.configure(array.clone()));
    assert!(Rc::ptr_eq(&source.current(), &array));
    old.release();
    let empty = source.current();
    old.release();
    assert!(Rc::ptr_eq(&source.current(), &empty));
    drop(source.configure(array.clone()));
    old.release();
    assert!(!Rc::ptr_eq(&source.current(), &empty));
    assert!(source.current().is_empty());
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "complete original watched-store source requires Node22 and original graph"]
fn pinned_watch_stores_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-watch-stores.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/watch-stores.mjs"))
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
    assert_eq!(originals.len(), 80);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/watch-stores-diff/run-{run}"));
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
        "80 complete source-array identity/release observations; {}",
        dir.display()
    );
}
