// SPDX-License-Identifier: AGPL-3.0-only
//! Winding/normal state comparisons for the outward orientation heuristic.
#![allow(clippy::unwrap_used)]
use port_core::orientation::{orient_normals_outward, tetra_volume6};
fn cases() -> Vec<(String, Vec<f32>, Vec<u32>)> {
    let mut out = vec![
        ("empty".into(), vec![], vec![]),
        (
            "sheet".into(),
            vec![0., 0., 0., 1., 0., 0., 0., 1., 0.],
            vec![0, 1, 2],
        ),
        ("point".into(), vec![1., 2., 3.], vec![]),
    ];
    for scale in [1., 0.001, 10000.] {
        for shift in [0., 1039.] {
            for inside in [false, true] {
                let mut p = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.];
                for x in &mut p {
                    *x = *x * scale + shift;
                }
                let mut idx = vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
                if inside {
                    for t in idx.as_chunks_mut::<3>().0 {
                        t.swap(1, 2);
                    }
                }
                out.push((format!("tetra_{scale}_{shift}_{inside}"), p, idx));
            }
        }
    }
    out
}
#[test]
fn mutation_and_guard_contract() {
    assert_eq!(
        tetra_volume6([1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [0.; 3]),
        1.
    );
    let mut idx = vec![0, 1, 5];
    let mut ns = vec![1.; 9];
    let before = ns.clone();
    let old = idx.clone();
    assert!(orient_normals_outward(&[0.; 9], &mut idx, &mut ns).is_err());
    assert_eq!(idx, old);
    assert_eq!(ns, before);
    let mut flips = 0;
    for (_, p, mut idx) in cases() {
        let mut ns = vec![1.; p.len()];
        if orient_normals_outward(&p, &mut idx, &mut ns).unwrap() {
            flips += 1;
            assert!(ns.iter().all(|&x| x == -1.));
        }
    }
    assert_eq!(flips, 6);
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_orientation_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/orientation-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw orientation evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 15);
    for (id, p, mut idx) in fixtures {
        let mut ns: Vec<f32> = (0..p.len()).map(|i| i as f32 / 7. - 1.).collect();
        let mut input = (p.len() as u32).to_le_bytes().to_vec();
        input.extend((idx.len() as u32).to_le_bytes());
        input.extend(p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
        input.extend(ns.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        let changed = orient_normals_outward(&p, &mut idx, &mut ns).unwrap();
        let mut raw = vec![u8::from(changed)];
        raw.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
        raw.extend(ns.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-orientation.cjs"))
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
        assert!(
            original.status.success(),
            "{id}: {}",
            String::from_utf8_lossy(&original.stderr)
        );
        assert!(
            raw == original.stdout,
            "{id}: boolean/winding/normals differ"
        );
        println!("{id}: changed={changed}, full winding + normal state compared");
    }
}
