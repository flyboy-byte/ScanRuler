// SPDX-License-Identifier: AGPL-3.0-only
//! Exact-bit welding source comparisons and boundary regressions.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use port_core::weld::{WeldError, filter_degenerate_triangles, weld_triangle_soup};

fn fixtures() -> Vec<(String, Vec<f32>)> {
    let mut cases = vec![
        ("empty".into(), vec![]),
        ("triangle".into(), vec![0., 0., 0., 1., 0., 0., 0., 1., 0.]),
        ("repeated".into(), vec![1.; 27]),
    ];
    let bits = [
        0, 0x80000000, 0x7fc12345, 0x7f812345, 0x7f800000, 0xff800000, 1,
    ];
    for (i, b) in bits.into_iter().enumerate() {
        cases.push((format!("bits_{i}"), vec![f32::from_bits(b); 18]));
    }
    for seed in [1u32, 7, 99, 1039] {
        let mut state = seed;
        let mut soup = Vec::new();
        for _ in 0..1200 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let n = (state % 31) as f32;
            soup.extend_from_slice(&[n, n / 2., n * -3.]);
        }
        cases.push((format!("seed_{seed}"), soup));
    }
    cases.push(("progress".into(), vec![1.; 2_097_156 * 3]));
    cases
}
#[test]
fn identity_and_guards() {
    let m = weld_triangle_soup(&[0., 0., 0., -0., 0., 0., 0., 0., 0.], 3, |_| {}).unwrap();
    assert_eq!(m.indices, [0, 1, 0]);
    assert_eq!(m.positions.len(), 6);
    assert_eq!(filter_degenerate_triangles(&m.indices).unwrap(), vec![]);
    assert_eq!(
        filter_degenerate_triangles(&[1, 2, 3, 2, 2, 3, 3, 4, 5]).unwrap(),
        vec![1, 2, 3, 3, 4, 5]
    );
    assert_eq!(
        filter_degenerate_triangles(&[1]),
        Err(WeldError::IncompleteTriple)
    );
    assert!(matches!(
        weld_triangle_soup(&[0.], 1, |_| {}),
        Err(WeldError::IncompleteTriple)
    ));
    assert!(matches!(
        weld_triangle_soup(&[0.; 3], 0, |_| {}),
        Err(WeldError::VertexLimit)
    ));
}
#[test]
#[ignore = "requires live frozen Node oracle"]
fn pinned_weld_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/weld-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw welding evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let cases = fixtures();
    assert_eq!(cases.len(), 15);
    for (id, soup) in cases {
        let mut progress = String::new();
        let m = weld_triangle_soup(&soup, soup.len() / 3, |t| {
            progress.push_str(t);
            progress.push('\n');
        })
        .unwrap();
        let filtered = filter_degenerate_triangles(&m.indices).unwrap();
        let mut raw = (m.positions.len() as u32).to_le_bytes().to_vec();
        raw.extend(m.positions.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        raw.extend(
            m.indices
                .iter()
                .chain(&filtered)
                .flat_map(|v| v.to_le_bytes()),
        );
        let input: Vec<u8> = soup
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect();
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-weld.cjs"))
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
            ("original.progress.txt", original.stderr.as_slice()),
            ("rust.progress.txt", progress.as_bytes()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert!(original.status.success(), "{id}: oracle failed");
        assert!(
            raw == original.stdout,
            "{id}: positions/indices/filtered differ"
        );
        assert_eq!(
            progress.as_bytes(),
            original.stderr,
            "{id}: progress differs"
        );
        println!(
            "{id}: {} vertices, {} indices compared exactly",
            m.positions.len() / 3,
            m.indices.len()
        );
    }
}
