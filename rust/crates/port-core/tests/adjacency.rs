// SPDX-License-Identifier: AGPL-3.0-only
//! Full ordered CSR comparison against the independently executed source.
#![allow(clippy::unwrap_used)]
use port_core::adjacency::{AdjacencyError, build_adjacency};
fn cases() -> Vec<(String, usize, Vec<u32>)> {
    let mut out = vec![
        ("empty".into(), 0, vec![]),
        ("isolated".into(), 6, vec![]),
        ("triangle".into(), 3, vec![0, 1, 2]),
        ("duplicate".into(), 4, vec![0, 1, 2, 0, 1, 2]),
        (
            "repeated_corners".into(),
            3,
            vec![0, 0, 0, 0, 1, 1, 2, 1, 2],
        ),
        ("shared_edge".into(), 4, vec![0, 1, 2, 2, 1, 3]),
        ("nonmanifold".into(), 5, vec![0, 1, 2, 1, 0, 3, 0, 1, 4]),
        (
            "disconnected_reversed".into(),
            8,
            vec![2, 1, 0, 3, 4, 5, 0, 1, 2],
        ),
    ];
    for seed in [1u32, 7, 99, 1039] {
        for n in [1, 2, 17, 127] {
            for triangles in [0, 1, 31] {
                let mut state = seed;
                let indices = (0..triangles * 3)
                    .map(|_| {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        state % n as u32
                    })
                    .collect();
                out.push((format!("seed_{seed}_v{n}_t{triangles}"), n, indices));
            }
        }
    }
    out
}
#[test]
fn ordered_neighbors_and_storage_guards() {
    let graph = build_adjacency(&[0, 1, 2, 2, 1, 3], 4, 4, 12).unwrap();
    assert_eq!(graph.offsets, [0, 2, 6, 10, 12]);
    assert_eq!(graph.list, [1, 2, 0, 2, 2, 3, 0, 1, 1, 3, 2, 1]);
    assert_eq!(
        build_adjacency(&[0, 1], 2, 2, 4),
        Err(AdjacencyError::InvalidMesh)
    );
    assert_eq!(
        build_adjacency(&[0, 1, 3], 3, 3, 6),
        Err(AdjacencyError::InvalidMesh)
    );
    assert_eq!(
        build_adjacency(&[], 4, 3, 0),
        Err(AdjacencyError::StorageLimit)
    );
    assert_eq!(
        build_adjacency(&[0, 1, 2], 3, 3, 5),
        Err(AdjacencyError::StorageLimit)
    );
    assert_eq!(
        build_adjacency(&[], usize::MAX, usize::MAX, 0),
        Err(AdjacencyError::StorageLimit)
    );
    let soup = [
        0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 1., 0., 1., 0., 0., 1., 1., 0.,
    ];
    let mesh = port_core::weld::weld_triangle_soup(&soup, soup.len() / 3, |_| {}).unwrap();
    assert_eq!(mesh.indices, [0, 1, 2, 2, 1, 3]);
    assert_eq!(
        build_adjacency(&mesh.indices, mesh.positions.len() / 3, 4, 12).unwrap(),
        graph
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_adjacency_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/adjacency-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw adjacency evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 56);
    for (id, n, indices) in cases {
        let graph = build_adjacency(&indices, n, n, indices.len() * 2).unwrap();
        let mut raw = (graph.offsets.len() as u32).to_le_bytes().to_vec();
        raw.extend((graph.list.len() as u32).to_le_bytes());
        raw.extend(
            graph
                .offsets
                .iter()
                .chain(&graph.list)
                .flat_map(|v| v.to_le_bytes()),
        );
        let mut input = (n as u32).to_le_bytes().to_vec();
        input.extend(indices.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/adjacency.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let d = dir.join(&id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, bytes) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(d.join(name), bytes).unwrap();
        }
        assert!(
            original.status.success(),
            "{id}: {}",
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(raw, original.stdout, "{id}: ordered CSR differs");
        println!(
            "{id}: {} offsets and {} ordered neighbors exact",
            graph.offsets.len(),
            graph.list.len()
        );
    }
}
