// SPDX-License-Identifier: AGPL-3.0-only
//! Exact source boolean, RMS and callback-order comparisons.
#![allow(clippy::unwrap_used)]
use port_core::curvature_gate::{curved_beyond_plane, rms_over};
use port_core::plane::fit_plane_tls;

struct Case {
    id: String,
    mode: u8,
    callback: u8,
    threshold: f64,
    positions: Vec<f32>,
    idx: Vec<u32>,
}
fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (name, positions, idx) in [
        ("empty", vec![], vec![]),
        (
            "ordered_duplicates",
            vec![0., 1., 2., 3., -4., 5., -6., 7., 8.],
            vec![2, 0, 1, 2],
        ),
        ("signed_zero", vec![-0., 0., -0.], vec![0, 0]),
        (
            "infinite",
            vec![f32::INFINITY, 1., 2., f32::NEG_INFINITY, 3., 4.],
            vec![1, 0],
        ),
        (
            "nonfinite_filter",
            vec![f32::NAN, 0., 1., 3., 4., 5., 0., 8., 9.],
            vec![0, 1, 2],
        ),
    ] {
        for callback in 0..5 {
            out.push(Case {
                id: format!("rms_{name}_cb{callback}"),
                mode: 0,
                callback,
                threshold: 0.,
                positions: positions.clone(),
                idx: idx.clone(),
            });
        }
    }
    let mut noisy = Vec::new();
    let mut flat = Vec::new();
    for y in -2..=2 {
        for x in -2..=2 {
            for z in [-1., 1.] {
                noisy.extend([4. * x as f32, 3. * y as f32, z]);
                flat.extend([4. * x as f32, 3. * y as f32, 0.]);
            }
        }
    }
    let boundary = 1.0f64 / 1.1;
    for (name, positions, idx) in [
        ("noisy", noisy, (0..50).collect()),
        ("flat", flat, (0..50).collect()),
        ("empty", vec![], vec![]),
        (
            "collinear",
            vec![0., 0., 0., 1., 2., 3., 2., 4., 6.],
            vec![0, 1, 2],
        ),
        ("coincident", vec![1.; 9], vec![0, 1, 2]),
    ] {
        for (label, threshold) in [
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("neg_inf", f64::NEG_INFINITY),
            ("negative", -1.),
            ("negative_zero", -0.),
            ("zero", 0.),
            ("half", 0.5),
            ("below", f64::from_bits(boundary.to_bits() - 1)),
            ("equal", boundary),
            ("above", f64::from_bits(boundary.to_bits() + 1)),
            ("one", 1.),
        ] {
            out.push(Case {
                id: format!("curved_{name}_{label}"),
                mode: 1,
                callback: 0,
                threshold,
                positions: positions.clone(),
                idx: idx.clone(),
            });
        }
    }
    out
}
fn output(c: &Case) -> Vec<u8> {
    if c.mode == 0 {
        let mut calls = Vec::new();
        let rms = rms_over(&c.positions, &c.idx, |x, y, z| {
            calls.extend([x, y, z]);
            match c.callback {
                0 => x,
                1 => f64::NAN,
                2 => {
                    if x == 0. {
                        f64::NAN
                    } else {
                        y
                    }
                }
                3 => {
                    if x > 0. {
                        f64::MAX
                    } else {
                        f64::INFINITY
                    }
                }
                4 => x - y + z,
                _ => unreachable!(),
            }
        })
        .unwrap();
        [rms]
            .into_iter()
            .chain(calls)
            .flat_map(f64::to_le_bytes)
            .collect()
    } else {
        let fit = fit_plane_tls(&c.positions, &c.idx).unwrap();
        let rms = fit.map_or(0., |plane| {
            rms_over(&c.positions, &c.idx, |x, y, z| plane.residual(x, y, z)).unwrap()
        });
        let mut out = vec![
            u8::from(curved_beyond_plane(&c.positions, &c.idx, c.threshold).unwrap()),
            u8::from(fit.is_some()),
        ];
        out.extend(rms.to_le_bytes());
        out
    }
}
#[test]
fn finite_filter_and_guard_before_callback() {
    let mut calls = 0;
    assert!(
        rms_over(&[0.; 3], &[0, 1], |_, _, _| {
            calls += 1;
            0.
        })
        .is_err()
    );
    assert_eq!(calls, 0);
    assert!(rms_over(&[], &[], |_, _, _| 0.).unwrap().is_infinite());
    assert_eq!(rms_over(&[0.; 6], &[1, 0, 1], |_, _, _| 3.).unwrap(), 3.);
    assert!(curved_beyond_plane(&[0.; 9], &[0, 1, 2], f64::NAN).unwrap());
    assert!(curved_beyond_plane(&[0.; 2], &[], 0.).is_err());
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_curvature_gate_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/curvature-gate-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw curvature evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 80);
    let mut yes = 0;
    let mut no = 0;
    for c in cases {
        let raw = output(&c);
        let mut input = vec![c.mode, c.callback];
        input.extend(c.threshold.to_le_bytes());
        input.extend((c.positions.len() as u32).to_le_bytes());
        input.extend(c.positions.iter().flat_map(|v| v.to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/curvature-gate.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let case_dir = dir.join(&c.id);
        std::fs::create_dir_all(&case_dir).unwrap();
        for (name, bytes) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(case_dir.join(name), bytes).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(raw, original.stdout, "{}: value/state differs", c.id);
        if c.mode == 1 {
            if raw[0] == 1 {
                yes += 1;
            } else {
                no += 1;
            }
        }
        println!("{}: exact {} value/state bytes", c.id, raw.len());
    }
    assert!(yes > 0 && no > 0);
    println!("curved decisions: {yes} true, {no} false");
}
