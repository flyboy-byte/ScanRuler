// SPDX-License-Identifier: AGPL-3.0-only
//! Full root and parent state after every independent operation.
#![allow(clippy::unwrap_used)]
use port_core::components::{ComponentError, find_root, make_union_find, vertex_components};
struct Case {
    id: String,
    mode: u32,
    n: usize,
    data: Vec<u32>,
    ops: Vec<[u32; 3]>,
}
fn cases() -> Vec<Case> {
    let mut out = vec![
        Case {
            id: "empty".into(),
            mode: 0,
            n: 0,
            data: vec![],
            ops: vec![],
        },
        Case {
            id: "singleton".into(),
            mode: 0,
            n: 1,
            data: vec![],
            ops: vec![[0, 0, 0], [1, 0, 0], [0, 0, 0]],
        },
    ];
    for reverse in [false, true] {
        let mut ops: Vec<_> = (0..7)
            .map(|i| {
                if reverse {
                    [1, i + 1, i]
                } else {
                    [1, i, i + 1]
                }
            })
            .collect();
        ops.extend([[0, 0, 0], [0, 1, 0], [1, 0, 3], [0, 0, 0], [0, 7, 0]]);
        out.push(Case {
            id: format!("chain_{reverse}"),
            mode: 0,
            n: 8,
            data: vec![],
            ops,
        });
    }
    for seed in [1u32, 7, 99, 1039] {
        for n in [1, 17, 64] {
            let mut state = seed;
            let mut next = || {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                state
            };
            let ops = (0..64)
                .map(|_| {
                    [
                        u32::from(next() % 3 != 0),
                        next() % n as u32,
                        next() % n as u32,
                    ]
                })
                .collect();
            out.push(Case {
                id: format!("ops_seed_{seed}_n{n}"),
                mode: 0,
                n,
                data: vec![],
                ops,
            });
        }
    }
    let mut meshes = vec![
        ("empty".into(), 0, vec![]),
        ("isolated".into(), 6, vec![]),
        ("triangle".into(), 3, vec![0, 1, 2]),
        ("duplicate".into(), 4, vec![0, 1, 2, 0, 1, 2]),
        ("repeated".into(), 3, vec![0, 0, 0, 0, 1, 1, 2, 1, 2]),
        ("shared".into(), 4, vec![0, 1, 2, 2, 1, 3]),
        ("nonmanifold".into(), 5, vec![0, 1, 2, 1, 0, 3, 0, 1, 4]),
        ("disconnected".into(), 8, vec![2, 1, 0, 3, 4, 5, 0, 1, 2]),
    ];
    for seed in [1u32, 7, 99, 1039] {
        for n in [1, 17, 64] {
            for t in [1, 31] {
                let mut state = seed;
                let data = (0..t * 3)
                    .map(|_| {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        state % n as u32
                    })
                    .collect();
                meshes.push((format!("seed_{seed}_n{n}_t{t}"), n, data));
            }
        }
    }
    for (id, n, data) in meshes {
        let ops = (0..n).rev().map(|i| [0, i as u32, 0]).collect();
        out.push(Case {
            id: format!("mesh_{id}"),
            mode: 1,
            n,
            data,
            ops,
        });
    }
    for (id, data) in [
        ("long_chain", vec![1, 2, 3, 4, 4]),
        ("branched", vec![1, 3, 3, 5, 5, 5]),
        ("forest", vec![0, 0, 2, 2, 4]),
    ] {
        let n = data.len();
        let mut ops: Vec<_> = (0..n).map(|i| [0, i as u32, 0]).collect();
        ops.extend([[0, 0, 0], [0, 0, 0]]);
        out.push(Case {
            id: format!("standalone_{id}"),
            mode: 2,
            n,
            data,
            ops,
        });
    }
    out
}
fn output(c: &Case) -> Vec<u8> {
    let mut result = vec![c.n as u32];
    if c.mode == 2 {
        let mut parent = c.data.clone();
        result.extend(&parent);
        for op in &c.ops {
            result.push(find_root(&mut parent, op[1]).unwrap());
            result.extend(&parent);
        }
    } else {
        let mut sets = if c.mode == 0 {
            make_union_find(c.n, c.n).unwrap()
        } else {
            vertex_components(&c.data, c.n, c.n).unwrap()
        };
        result.extend(sets.parent());
        for &[kind, a, b] in &c.ops {
            let root = if kind == 0 {
                sets.find(a).unwrap()
            } else {
                sets.union(a, b).unwrap();
                u32::MAX
            };
            result.push(root);
            result.extend(sets.parent());
        }
    }
    result.into_iter().flat_map(u32::to_le_bytes).collect()
}
#[test]
fn root_direction_halving_and_guards() {
    let mut parent = vec![1, 2, 3, 4, 4];
    assert_eq!(find_root(&mut parent, 0).unwrap(), 4);
    assert_eq!(parent, [2, 2, 4, 4, 4]);
    for mut invalid in [vec![1, 0], vec![2, 1]] {
        let before = invalid.clone();
        assert_eq!(
            find_root(&mut invalid, 0),
            Err(ComponentError::InvalidParent)
        );
        assert_eq!(invalid, before);
    }
    assert_eq!(find_root(&mut [], 0), Err(ComponentError::InvalidIndex));
    let mut sets = make_union_find(4, 4).unwrap();
    sets.union(0, 1).unwrap();
    sets.union(1, 2).unwrap();
    let before = sets.parent().to_vec();
    assert_eq!(sets.union(0, 4), Err(ComponentError::InvalidIndex));
    assert_eq!(sets.parent(), before);
    assert_eq!(sets.find(0).unwrap(), 2);
    assert_eq!(sets.parent(), [2, 2, 2, 3]);
    assert_eq!(make_union_find(5, 4), Err(ComponentError::VertexLimit));
    assert_eq!(
        make_union_find(i32::MAX as usize + 1, usize::MAX),
        Err(ComponentError::VertexLimit)
    );
    assert_eq!(
        vertex_components(&[0, 1], 2, 2),
        Err(ComponentError::InvalidIndex)
    );
    assert_eq!(
        vertex_components(&[0, 1, 3], 3, 3),
        Err(ComponentError::InvalidIndex)
    );
    assert_eq!(
        vertex_components(&[2, 1, 0], 4, 4).unwrap().parent(),
        [2, 2, 2, 3]
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_components_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/components-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw component evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 51);
    for c in cases {
        let raw = output(&c);
        let mut input = c.mode.to_le_bytes().to_vec();
        input.extend((c.n as u32).to_le_bytes());
        input.extend((c.data.len() as u32).to_le_bytes());
        input.extend(
            c.data
                .iter()
                .chain(c.ops.iter().flatten())
                .flat_map(|v| v.to_le_bytes()),
        );
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/components.cjs"))
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
        assert_eq!(raw, original.stdout, "{}: parent/query state differs", c.id);
        println!(
            "{}: {} complete operation snapshots exact",
            c.id,
            c.ops.len() + 1
        );
    }
}
