// SPDX-License-Identifier: AGPL-3.0-only
//! Source comparison for area-weighted normals and invalid mesh guards.
#![allow(clippy::unwrap_used)]
use port_core::normals::{NormalError, compute_vertex_normals};
fn cases() -> Vec<(String, Vec<f32>, Vec<u32>)> {
    let mut out = vec![
        ("empty".into(), vec![], vec![]),
        ("isolated".into(), vec![1., 2., 3.], vec![]),
        (
            "triangle".into(),
            vec![0., 0., 0., 1., 0., 0., 0., 1., 0.],
            vec![0, 1, 2],
        ),
        ("collapsed".into(), vec![1.; 9], vec![0, 1, 2]),
        ("repeated_indices".into(), vec![1., 2., 3.], vec![0, 0, 0]),
    ];
    for seed in [1u32, 7, 99, 1039] {
        let mut state = seed;
        let mut p = Vec::new();
        let mut indices = Vec::new();
        for _ in 0..90 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            p.push((state as i32) as f32 / 7919.);
        }
        for _ in 0..300 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            indices.push(state % 30);
        }
        out.push((format!("seed_{seed}"), p, indices));
    }
    out.push((
        "tiny_fallback".into(),
        vec![0., 0., 0., 1e-15, 0., 0., 0., 1e-15, 0.],
        vec![0, 1, 2],
    ));
    out
}
#[test]
fn guards_and_known_normal() {
    assert_eq!(
        compute_vertex_normals(&[0.; 9], &[0, 1, 3], 3),
        Err(NormalError::InvalidMesh)
    );
    assert_eq!(
        compute_vertex_normals(&[0.; 2], &[], 3),
        Err(NormalError::InvalidMesh)
    );
    assert_eq!(
        compute_vertex_normals(&[0.; 3], &[], 0),
        Err(NormalError::VertexLimit)
    );
    assert_eq!(
        compute_vertex_normals(&[1., 2., 3.], &[], 1).unwrap(),
        [0., 0., 1.]
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_normals_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/normals-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw normals evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 10);
    for (id, p, idx) in fixtures {
        let normals = compute_vertex_normals(&p, &idx, p.len() / 3).unwrap();
        let raw: Vec<u8> = normals
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect();
        let mut input = (p.len() as u32).to_le_bytes().to_vec();
        input.extend(p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/vertex-normals.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let case = dir.join(&id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert!(original.status.success(), "{id}: oracle failed");
        assert!(raw == original.stdout, "{id}: normals differ");
        println!("{id}: {} normal components compared exactly", normals.len());
    }
}
