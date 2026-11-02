// SPDX-License-Identifier: AGPL-3.0-only
//! Complete ordered original/Rust index outputs, including permissive boundaries.
#![allow(clippy::unwrap_used)]
use port_core::mesh_selection::{SelectionError, remap_vertices, triangles_within};
struct Case {
    id: String,
    mode: u32,
    n: usize,
    a: Vec<u32>,
    b: Vec<u32>,
}
fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (id, a, b) in [
        ("empty", vec![], vec![]),
        ("empty_ids", vec![], vec![0, -1]),
        ("empty_map", vec![0, 1, u32::MAX], vec![]),
        (
            "mixed",
            vec![4, 0, 2, 1, 2, 3],
            vec![8, -1, 3, i32::MAX, -2],
        ),
        ("duplicates", vec![2, 2, 0, 1, 0], vec![3, 2, 1]),
        ("all_removed", vec![1, 0, 2, 99], vec![-1, i32::MIN, -2]),
    ] {
        out.push(Case {
            id: format!("remap_{id}"),
            mode: 0,
            n: 0,
            a,
            b: b.into_iter().map(|v| v as u32).collect(),
        });
    }
    for (id, n, a, b) in [
        ("empty", 0, vec![], vec![]),
        ("none_marked", 3, vec![0, 1, 2], vec![]),
        ("all_marked", 4, vec![0, 1, 2, 2, 1, 3], vec![3, 2, 1, 0]),
        ("partial_mark", 4, vec![0, 1, 2, 2, 1, 3], vec![0, 1, 2]),
        (
            "out_of_range",
            3,
            vec![0, 1, 3, 2, 1, 0],
            vec![0, 1, 2, 3, u32::MAX],
        ),
        ("tail_one", 3, vec![0, 1, 2, 1], vec![0, 1, 2]),
        ("tail_two", 3, vec![0, 1, 2, 2, 1], vec![0, 1, 2]),
        (
            "repeated",
            3,
            vec![0, 0, 0, 0, 1, 2, 0, 1, 2],
            vec![2, 0, 0, 1],
        ),
    ] {
        out.push(Case {
            id: format!("triangles_{id}"),
            mode: 1,
            n,
            a,
            b,
        });
    }
    for seed in [1u32, 7, 99, 1039] {
        for n in [0, 1, 17] {
            for m in [0, 31] {
                let mut state = seed;
                let mut next = || {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    state
                };
                let ids = (0..m).map(|_| next() % (n as u32 + 2)).collect();
                let map = (0..n)
                    .map(|_| {
                        let x = next();
                        if x % 5 == 0 { u32::MAX } else { x % 37 }
                    })
                    .collect();
                out.push(Case {
                    id: format!("remap_seed_{seed}_n{n}_m{m}"),
                    mode: 0,
                    n,
                    a: ids,
                    b: map,
                });
                let indices = (0..m * 3 + (seed % 3) as usize)
                    .map(|_| next() % (n as u32 + 2))
                    .collect();
                let selected = (0..m).map(|_| next() % (n as u32 + 2)).collect();
                out.push(Case {
                    id: format!("triangles_seed_{seed}_n{n}_m{m}"),
                    mode: 1,
                    n,
                    a: indices,
                    b: selected,
                });
            }
        }
    }
    out
}
fn output(c: &Case) -> Vec<u8> {
    let indices = if c.mode == 0 {
        let map: Vec<_> = c.b.iter().map(|&v| v as i32).collect();
        remap_vertices(&c.a, &map, c.a.len()).unwrap()
    } else {
        triangles_within(&c.a, &c.b, c.n, c.n, c.a.len()).unwrap()
    };
    let mut bytes = (indices.len() as u32).to_le_bytes().to_vec();
    bytes.extend(indices.into_iter().flat_map(u32::to_le_bytes));
    bytes
}
#[test]
fn source_filtering_order_tails_and_budgets() {
    assert_eq!(
        remap_vertices(&[2, 0, 1, 2, 9], &[4, -1, 3], 4).unwrap(),
        [3, 4, 3]
    );
    assert_eq!(remap_vertices(&[0, 1], &[-1, -2], 0).unwrap(), []);
    assert_eq!(
        remap_vertices(&[0], &[3], 0),
        Err(SelectionError::StorageLimit)
    );
    assert_eq!(
        triangles_within(&[0, 1, 2, 2, 1], &[0, 1, 2], 3, 3, 3).unwrap(),
        [0, 1, 2]
    );
    assert_eq!(
        triangles_within(&[0, 1, 3], &[0, 1, 2, 3], 3, 3, 3).unwrap(),
        []
    );
    assert_eq!(
        triangles_within(&[], &[], 4, 3, 0),
        Err(SelectionError::StorageLimit)
    );
    assert_eq!(
        triangles_within(&[0, 1, 2], &[0, 1, 2], 3, 3, 2),
        Err(SelectionError::StorageLimit)
    );
    let selected = remap_vertices(&[0, 1, 2, 3], &[0, -1, 1, 2], 3).unwrap();
    assert_eq!(
        triangles_within(&[0, 1, 2], &selected, 3, 3, 3).unwrap(),
        [0, 1, 2]
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_mesh_selection_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/mesh-selection-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw selection evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 62);
    for c in cases {
        let raw = output(&c);
        let mut input = c.mode.to_le_bytes().to_vec();
        input.extend((c.n as u32).to_le_bytes());
        input.extend((c.a.len() as u32).to_le_bytes());
        input.extend(c.a.iter().chain(&c.b).flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-selection.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let d = dir.join(&c.id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(d.join(name), data).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(
            raw, original.stdout,
            "{}: ordered index output differs",
            c.id
        );
        println!("{}: {} output indices exact", c.id, (raw.len() - 4) / 4);
    }
}
