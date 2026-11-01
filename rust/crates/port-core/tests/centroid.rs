// SPDX-License-Identifier: AGPL-3.0-only
//! Surface/volume centroid, closure and area paired values.
#![allow(clippy::unwrap_used)]
use port_core::centroid::{mesh_centroid, triangle_area_sum};
struct Case {
    id: String,
    p: Vec<f32>,
    idx: Vec<u32>,
}
fn cases() -> Vec<Case> {
    let mut out = vec![
        Case {
            id: "empty".into(),
            p: vec![],
            idx: vec![],
        },
        Case {
            id: "isolated".into(),
            p: vec![1., 2., 3.],
            idx: vec![],
        },
        Case {
            id: "sheet".into(),
            p: vec![0., 0., 0., 1., 0., 0., 0., 1., 0.],
            idx: vec![0, 1, 2],
        },
        Case {
            id: "collapsed".into(),
            p: vec![1.; 9],
            idx: vec![0, 1, 2],
        },
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
                out.push(Case {
                    id: format!("tetra_{scale}_{shift}_{inside}"),
                    p,
                    idx,
                });
            }
        }
    }
    for faces in 1..=3 {
        out.push(Case {
            id: format!("open_tetra_{faces}"),
            p: vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.],
            idx: vec![0, 2, 1, 0, 1, 3, 0, 3, 2][..faces * 3].to_vec(),
        });
    }
    for seed in [1u32, 7, 99, 1039] {
        let mut state = seed;
        let mut p = Vec::new();
        let mut idx = Vec::new();
        for _ in 0..90 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            p.push((state as i32) as f32 / 7919.);
        }
        for _ in 0..90 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            idx.push(state % 30);
        }
        out.push(Case {
            id: format!("seed_{seed}"),
            p,
            idx,
        });
    }
    out
}
#[test]
fn analytic_and_guard_contract() {
    let c = mesh_centroid(
        &[0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.],
        &[0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
    )
    .unwrap();
    assert!(c.closed);
    assert!((c.volume_mm3 - 1. / 6.).abs() < 1e-12);
    for x in c.volume.unwrap() {
        assert!((x - 0.25).abs() < 1e-12);
    }
    assert_eq!(mesh_centroid(&[1., 2., 3.], &[]).unwrap().area, [0.; 3]);
    assert!(triangle_area_sum(&[0.; 3], &[0, 1, 2]).is_err());
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_centroid_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/centroid-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw centroid evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 23);
    for c in fixtures {
        let r = mesh_centroid(&c.p, &c.idx).unwrap();
        let area = triangle_area_sum(&c.p, &c.idx).unwrap();
        let mut values = vec![area];
        if let Some(v) = r.volume {
            values.extend(v);
        }
        values.extend(r.area);
        values.push(r.volume_mm3);
        let mut raw = vec![u8::from(r.closed), u8::from(r.volume.is_some())];
        raw.extend(values.iter().flat_map(|v| v.to_le_bytes()));
        let mut input = (c.p.len() as u32).to_le_bytes().to_vec();
        input.extend(c.p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-centroid.cjs"))
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
        assert_eq!(raw.len(), original.stdout.len(), "{}: lengths", c.id);
        assert_eq!(
            raw[..2],
            original.stdout[..2],
            "{}: closure/volume flags",
            c.id
        );
        for (a, b) in raw[2..]
            .as_chunks::<8>()
            .0
            .iter()
            .zip(original.stdout[2..].as_chunks::<8>().0)
        {
            let a = f64::from_le_bytes(*a);
            let b = f64::from_le_bytes(*b);
            assert!(
                (a - b).abs() <= 1e-12 * (1. + b.abs()),
                "{}: {a} vs {b}",
                c.id
            );
        }
        println!(
            "{}: closed={}, all area/volume/centroid values compared",
            c.id, r.closed
        );
    }
}
