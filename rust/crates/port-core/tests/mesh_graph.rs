// SPDX-License-Identifier: AGPL-3.0-only
//! Complete independently executed pipeline state, progress and source errors.
#![allow(clippy::unwrap_used, clippy::panic)] // Test assertions only.
use port_core::mesh_graph::{GraphError, GraphLimits, ParsedMesh, build_mesh_graph};
struct Case {
    id: String,
    p: Vec<f32>,
    indices: Option<Vec<u32>>,
    soup: bool,
    keep: bool,
}
fn case(id: &str, p: Vec<f32>, indices: Option<Vec<u32>>, soup: bool, keep: bool) -> Case {
    Case {
        id: id.into(),
        p,
        indices,
        soup,
        keep,
    }
}
fn cases() -> Vec<Case> {
    let triangle = vec![0., 0., 0., 1., 0., 0., 0., 1., 0.];
    let mut out = vec![
        case("empty", vec![], None, false, false),
        case("incomplete_positions", vec![0., 0.], None, false, false),
        case("incomplete_soup", vec![0.; 3], None, true, false),
        case(
            "nan_before_bad_indices",
            vec![f32::NAN, 0., 0.],
            Some(vec![1]),
            false,
            false,
        ),
        case("infinite", vec![0., 0., f32::INFINITY], None, false, false),
        case(
            "incomplete_before_nan",
            vec![f32::NAN, 0.],
            None,
            false,
            false,
        ),
        case(
            "incomplete_indices",
            triangle.clone(),
            Some(vec![0, 1]),
            false,
            false,
        ),
        case(
            "outside_indices",
            triangle.clone(),
            Some(vec![0, 1, 3]),
            false,
            false,
        ),
        case(
            "soup_incomplete_indices",
            triangle.clone(),
            Some(vec![0, 1]),
            true,
            false,
        ),
        case(
            "soup_outside_indices",
            triangle.clone(),
            Some(vec![0, 1, 3]),
            true,
            false,
        ),
        case("missing_indices", triangle.clone(), None, false, false),
        case(
            "empty_indices",
            triangle.clone(),
            Some(vec![]),
            false,
            false,
        ),
        case("collapsed_soup", vec![0.; 9], None, true, false),
        case(
            "collapsed_indices",
            triangle.clone(),
            Some(vec![0, 0, 1]),
            false,
            false,
        ),
        case(
            "mixed_degenerates",
            triangle.clone(),
            Some(vec![0, 0, 1, 0, 1, 2, 1, 1, 2]),
            false,
            false,
        ),
        case(
            "soup_ignores_valid_indices",
            triangle.clone(),
            Some(vec![2, 1, 0]),
            true,
            false,
        ),
        case(
            "signed_zero",
            vec![-0., 0., 0., 1., 0., 0., 0., 1., 0.],
            None,
            true,
            false,
        ),
        case(
            "collinear_usable_ids",
            vec![0., 0., 0., 1., 1., 1., 2., 2., 2.],
            Some(vec![0, 1, 2]),
            false,
            false,
        ),
    ];
    let tetra = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.];
    let faces = vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
    let open = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 1., 1., 0.];
    for soup in [false, true] {
        for keep in [false, true] {
            out.push(case(
                &format!("triangle_s{soup}_k{keep}"),
                triangle.clone(),
                if soup { None } else { Some(vec![0, 1, 2]) },
                soup,
                keep,
            ));
            for reversed in [false, true] {
                let mut idx = faces.clone();
                if reversed {
                    for t in idx.as_chunks_mut::<3>().0 {
                        t.swap(1, 2);
                    }
                }
                let p = if soup {
                    idx.iter()
                        .flat_map(|&i| tetra[i as usize * 3..i as usize * 3 + 3].iter().copied())
                        .collect()
                } else {
                    tetra.clone()
                };
                out.push(case(
                    &format!("tetra_s{soup}_k{keep}_r{reversed}"),
                    p,
                    if soup { None } else { Some(idx) },
                    soup,
                    keep,
                ));
            }
            let idx = vec![0, 1, 2, 2, 1, 3];
            let p = if soup {
                idx.iter()
                    .flat_map(|&i| open[i as usize * 3..i as usize * 3 + 3].iter().copied())
                    .collect()
            } else {
                open.clone()
            };
            out.push(case(
                &format!("open_s{soup}_k{keep}"),
                p,
                if soup { None } else { Some(idx) },
                soup,
                keep,
            ));
        }
    }
    for seed in [1u32, 7, 99, 1039] {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            state
        };
        let p: Vec<_> = (0..51)
            .map(|_| (next() % 1001) as f32 / 10. - 50.)
            .collect();
        let mut idx = vec![0, 1, 2];
        idx.extend((0..90).map(|_| next() % 17));
        for soup in [false, true] {
            for keep in [false, true] {
                let positions = if soup {
                    idx.iter()
                        .flat_map(|&i| p[i as usize * 3..i as usize * 3 + 3].iter().copied())
                        .collect()
                } else {
                    p.clone()
                };
                out.push(case(
                    &format!("seed_{seed}_s{soup}_k{keep}"),
                    positions,
                    if soup { None } else { Some(idx.clone()) },
                    soup,
                    keep,
                ));
            }
        }
    }
    out
}
fn text(out: &mut Vec<u8>, s: &str) {
    out.extend((s.len() as u32).to_le_bytes());
    out.extend(s.as_bytes());
}
fn output(c: &Case) -> Vec<u8> {
    let mut progress = Vec::new();
    let parsed = ParsedMesh {
        positions: c.p.clone(),
        indices: c.indices.clone(),
        soup: c.soup,
        keep_winding: c.keep,
    };
    let result = build_mesh_graph(
        parsed,
        GraphLimits {
            vertices: 10000,
            indices: 10000,
            neighbors: 20000,
        },
        |s| progress.push(s.to_owned()),
    );
    let mut out = Vec::new();
    match result {
        Ok(g) => {
            out.push(0);
            out.extend((g.vertex_count as u32).to_le_bytes());
            out.extend((g.triangle_count as u32).to_le_bytes());
            out.extend(g.bbox_diag.to_le_bytes());
            for (len, bytes) in [
                (
                    g.positions.len(),
                    g.positions
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
                (
                    g.indices.len(),
                    g.indices.iter().flat_map(|v| v.to_le_bytes()).collect(),
                ),
                (
                    g.normals.len(),
                    g.normals.iter().flat_map(|v| v.to_le_bytes()).collect(),
                ),
                (
                    g.adjacency.offsets.len(),
                    g.adjacency
                        .offsets
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect(),
                ),
                (
                    g.adjacency.list.len(),
                    g.adjacency
                        .list
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect(),
                ),
            ] {
                out.extend((len as u32).to_le_bytes());
                out.extend(bytes);
            }
        }
        Err(GraphError::Source(s)) => {
            out.push(1);
            text(&mut out, s);
        }
        Err(e) => panic!("unexpected host error {e:?}"),
    }
    out.extend((progress.len() as u32).to_le_bytes());
    for s in progress {
        text(&mut out, &s);
    }
    out
}
#[test]
fn graph_storage_limits_and_adjacency_before_flip() {
    let p = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.];
    let idx = vec![0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2];
    let graph = build_mesh_graph(
        ParsedMesh {
            positions: p.clone(),
            indices: Some(idx.clone()),
            soup: false,
            keep_winding: false,
        },
        GraphLimits {
            vertices: 4,
            indices: 12,
            neighbors: 24,
        },
        |_| {},
    )
    .unwrap();
    assert_ne!(graph.indices, idx);
    assert_eq!(
        graph.adjacency,
        port_core::adjacency::build_adjacency(&idx, 4, 4, 24).unwrap()
    );
    let parsed = || ParsedMesh {
        positions: p.clone(),
        indices: Some(idx.clone()),
        soup: false,
        keep_winding: false,
    };
    assert_eq!(
        build_mesh_graph(
            parsed(),
            GraphLimits {
                vertices: 3,
                indices: 12,
                neighbors: 24
            },
            |_| {}
        )
        .unwrap_err(),
        GraphError::StorageLimit
    );
    assert_eq!(
        build_mesh_graph(
            parsed(),
            GraphLimits {
                vertices: 4,
                indices: 11,
                neighbors: 24
            },
            |_| {}
        )
        .unwrap_err(),
        GraphError::StorageLimit
    );
    assert_eq!(
        build_mesh_graph(
            parsed(),
            GraphLimits {
                vertices: 4,
                indices: 12,
                neighbors: 23
            },
            |_| {}
        )
        .unwrap_err(),
        GraphError::StorageLimit
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_mesh_graph_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/mesh-graph-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw graph evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 50);
    let mut success = 0;
    let mut refusal = 0;
    for c in cases {
        let raw = output(&c);
        let mut input = vec![
            u8::from(c.soup),
            u8::from(c.keep),
            u8::from(c.indices.is_some()),
            0,
        ];
        input.extend((c.p.len() as u32).to_le_bytes());
        input.extend(c.p.iter().flat_map(|v| v.to_le_bytes()));
        if let Some(idx) = &c.indices {
            input.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
        }
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-graph.cjs"))
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
            "{}: graph/progress/source error differs",
            c.id
        );
        if raw[0] == 0 {
            success += 1;
        } else {
            refusal += 1;
        }
        println!(
            "{}: {} complete state/progress bytes exact",
            c.id,
            raw.len()
        );
    }
    assert_eq!((success, refusal), (36, 14));
    println!("full graph: {success} successful states, {refusal} source refusals");
}
