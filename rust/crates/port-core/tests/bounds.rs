// SPDX-License-Identifier: AGPL-3.0-only
//! Full source extent/bounds comparisons with exact signed-zero identities.
#![allow(clippy::unwrap_used)]
use port_core::bounds::{bounds_of, extent_of, extent_of_indexed};
struct Case {
    id: String,
    p: Vec<f32>,
    count: usize,
    idx: Vec<u32>,
}
fn cases() -> Vec<Case> {
    let mut out = vec![
        Case {
            id: "empty".into(),
            p: vec![],
            count: 0,
            idx: vec![],
        },
        Case {
            id: "point".into(),
            p: vec![1., 2., 3.],
            count: 1,
            idx: vec![0],
        },
        Case {
            id: "triangle".into(),
            p: vec![0., 0., 0., 1., 2., 3., -1., -2., -3.],
            count: 3,
            idx: vec![2, 1, 2],
        },
        Case {
            id: "prefix".into(),
            p: vec![0., 0., 0., 1., 2., 3., 100., 100., 100.],
            count: 2,
            idx: vec![0, 2],
        },
    ];
    for first in [-0.0f32, 0.] {
        out.push(Case {
            id: format!("zero_{:08x}", first.to_bits()),
            p: vec![first; 6],
            count: 2,
            idx: vec![1, 0],
        });
    }
    for seed in [1u32, 7, 99, 1039] {
        let mut state = seed;
        let mut p = Vec::new();
        for _ in 0..90 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            p.push((state as i32) as f32 / 7919.);
        }
        out.push(Case {
            id: format!("seed_{seed}"),
            p,
            count: 17,
            idx: vec![0, 3, 11, 29, 3],
        });
    }
    out.push(Case {
        id: "tiny_radius".into(),
        p: vec![0., 0., 0., 1e-7, 1e-7, 1e-7],
        count: 2,
        idx: vec![0, 1],
    });
    out.push(Case {
        id: "huge".into(),
        p: vec![f32::MAX, 0., 0., -f32::MAX, 0., 0.],
        count: 2,
        idx: vec![0, 1],
    });
    out
}
#[test]
fn empty_and_host_guards() {
    let e = extent_of(&[], 0).unwrap();
    assert_eq!(e.min, [f64::INFINITY; 3]);
    assert_eq!(bounds_of(&[], 0).unwrap().radius, 1e-4);
    assert!(extent_of(&[0.; 3], 2).is_err());
    assert!(extent_of_indexed(&[0.; 3], &[1]).is_err());
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_bounds_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/bounds-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw bounds evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 12);
    for c in fixtures {
        let e = extent_of(&c.p, c.count).unwrap();
        let ei = extent_of_indexed(&c.p, &c.idx).unwrap();
        let b = bounds_of(&c.p, c.count).unwrap();
        let values: Vec<f64> = e
            .min
            .into_iter()
            .chain(e.max)
            .chain(ei.min)
            .chain(ei.max)
            .chain(b.min)
            .chain(b.max)
            .chain(b.center)
            .chain([b.radius])
            .collect();
        let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let mut input = (c.p.len() as u32).to_le_bytes().to_vec();
        input.extend((c.count as u32).to_le_bytes());
        input.extend(c.p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/mesh-bounds.cjs"))
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
        assert_eq!(raw.len(), 176);
        assert_eq!(original.stdout.len(), 176);
        assert_eq!(
            raw[..168],
            original.stdout[..168],
            "{}: extent/center bit identity",
            c.id
        );
        let reference = f64::from_le_bytes(original.stdout[168..].try_into().unwrap());
        assert!(
            (b.radius - reference).abs() <= 1e-12 * (1. + reference.abs()),
            "{}: radius",
            c.id
        );
        println!("{}: all extents/center exact, radius compared", c.id);
    }
}
