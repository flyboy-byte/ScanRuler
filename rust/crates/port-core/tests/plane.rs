// SPDX-License-Identifier: AGPL-3.0-only
//! Pre-registered fixed-selection fit and residual comparison contract.
#![allow(clippy::unwrap_used)]
use port_core::plane::fit_plane_tls;
fn cases() -> Vec<(String, Vec<f32>, Vec<u32>)> {
    let mut out = vec![
        ("empty".into(), vec![], vec![]),
        ("insufficient".into(), vec![0.; 6], vec![0, 1]),
        ("coincident".into(), vec![1.; 9], vec![0, 1, 2]),
        (
            "collinear".into(),
            vec![0., 0., 0., 1., 2., 4., 2., 4., 8., 3., 6., 12.],
            vec![0, 1, 2, 3],
        ),
    ];
    for seed in [1u32, 7, 99, 1039] {
        for noisy in [false, true] {
            let mut state = seed;
            let mut p = Vec::new();
            for y in 0..10 {
                for x in 0..10 {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = if noisy {
                        (state % 100) as f32 / 10000. - 0.005
                    } else {
                        0.
                    };
                    p.extend_from_slice(&[
                        x as f32,
                        y as f32,
                        2. * x as f32 - 3. * y as f32 + 7. + noise,
                    ]);
                }
            }
            let idx = (0..100).filter(|i| i % 3 != 0).collect();
            out.push((format!("tilted_{seed}_{noisy}"), p, idx));
        }
    }
    out.push((
        "horizontal".into(),
        vec![0., 0., 7., 1., 0., 7., 0., 1., 7., 1., 1., 7.],
        vec![0, 1, 2, 3, 2, 1],
    ));
    out
}
#[test]
fn source_refusal_and_analytic_contract() {
    let p = fit_plane_tls(&[0., 0., 7., 1., 0., 7., 0., 1., 7.], &[0, 1, 2])
        .unwrap()
        .unwrap();
    assert!(p.residual(0., 0., 7.).abs() < 1e-12);
    assert!(fit_plane_tls(&[0.; 9], &[0, 1, 2]).unwrap().is_none());
    assert!(fit_plane_tls(&[0.; 3], &[1]).is_err());
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_plane_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/plane-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw plane evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 13);
    for (id, p, idx) in fixtures {
        let fit = fit_plane_tls(&p, &idx).unwrap();
        let mut raw = vec![u8::from(fit.is_some())];
        if let Some(fit) = fit {
            for v in fit.normal.into_iter().chain([fit.d]) {
                raw.extend(v.to_le_bytes());
            }
            for &i in &idx {
                let j = i as usize * 3;
                raw.extend(
                    fit.residual(f64::from(p[j]), f64::from(p[j + 1]), f64::from(p[j + 2]))
                        .to_le_bytes(),
                );
            }
        }
        let mut input = (p.len() as u32).to_le_bytes().to_vec();
        input.extend(p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/plane-tls.cjs"))
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
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(d.join(name), data).unwrap();
        }
        assert!(
            original.status.success(),
            "{id}: {}",
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(raw.len(), original.stdout.len(), "{id}: length differs");
        assert_eq!(raw[0], original.stdout[0], "{id}: presence differs");
        let mut worst = 0.0f64;
        for (a, b) in raw[1..]
            .as_chunks::<8>()
            .0
            .iter()
            .zip(original.stdout[1..].as_chunks::<8>().0)
        {
            let a = f64::from_le_bytes(*a);
            let b = f64::from_le_bytes(*b);
            let error = (a - b).abs();
            assert!(
                error <= 1e-12 * (1. + b.abs()),
                "{id}: field {a} vs {b}, error {error}"
            );
            worst = worst.max(error);
        }
        println!(
            "{id}: fit={}, full fields/residuals compared; max abs error {worst:e}",
            fit.is_some()
        );
    }
}
