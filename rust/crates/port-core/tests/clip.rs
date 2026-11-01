// SPDX-License-Identifier: AGPL-3.0-only
//! Used-point clipping values, full selections and last-good continuations.
#![allow(clippy::unwrap_used)]
use port_core::clip::{ClipError, clipped_refit, fit_plane_clipped};
struct Case {
    id: String,
    p: Vec<f32>,
    idx: Vec<u32>,
    k: f64,
    mode: u8,
}
fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for seed in [1u32, 7, 99, 1039] {
        for k in [0., 1., 3.] {
            let mut state = seed;
            let mut p = Vec::new();
            for y in 0..20 {
                for x in 0..20 {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    let mut z = 2. * x as f32 - 3. * y as f32 + 7. + (state % 100) as f32 / 5000.;
                    if (x + y * 20) % 37 == 0 {
                        z += 1.;
                    }
                    p.extend_from_slice(&[x as f32, y as f32, z]);
                }
            }
            out.push(Case {
                id: format!("seed_{seed}_k{k}"),
                p,
                idx: (0..400).collect(),
                k,
                mode: 0,
            });
        }
    }
    for mode in 1..=4 {
        let mut p = vec![0.; 60];
        for i in 0..20 {
            p[i * 3] = if mode == 3 {
                if i == 19 { -1. } else { 1. }
            } else if i == 19 {
                100.
            } else {
                0.
            };
        }
        out.push(Case {
            id: format!("generic_{mode}"),
            p,
            idx: (0..20).collect(),
            k: 1.,
            mode,
        });
    }
    out.push(Case {
        id: "insufficient".into(),
        p: vec![0.; 6],
        idx: vec![0, 1],
        k: 3.,
        mode: 0,
    });
    out
}
#[test]
fn invalid_selection_guard() {
    assert!(matches!(
        fit_plane_clipped(&[0.; 3], &[1], 3., 1),
        Err(ClipError::InvalidInput)
    ));
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_clip_comparison() {
    use std::{
        cell::Cell,
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/clip-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw clipping evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 17);
    for c in fixtures {
        let calls = Cell::new(0u32);
        let result = if c.mode == 0 {
            fit_plane_clipped(&c.p, &c.idx, c.k, c.idx.len())
                .unwrap()
                .map(|r| {
                    (
                        [
                            r.plane.normal[0],
                            r.plane.normal[1],
                            r.plane.normal[2],
                            r.plane.d,
                            r.sigma,
                            r.span,
                            r.min,
                            r.max,
                        ],
                        r.used,
                    )
                })
        } else {
            clipped_refit(
                &c.p,
                &c.idx,
                c.k,
                c.idx.len(),
                |_| {
                    calls.set(calls.get() + 1);
                    Ok(if c.mode == 1 && calls.get() > 1 {
                        None
                    } else {
                        Some(if c.mode == 4 { calls.get() - 1 } else { 0 })
                    })
                },
                |model, x, _, _| {
                    if c.mode == 2 || c.mode == 4 && *model > 0 || c.mode == 3 && x < 0. {
                        f64::NAN
                    } else {
                        x
                    }
                },
            )
            .unwrap()
            .map(|r| {
                (
                    [
                        0.,
                        0.,
                        0.,
                        f64::from(r.model),
                        r.sigma,
                        r.span,
                        r.min,
                        r.max,
                    ],
                    r.used,
                )
            })
        };
        let mut raw = vec![u8::from(result.is_some())];
        if let Some((v, used)) = &result {
            raw.extend(v.iter().flat_map(|v| v.to_le_bytes()));
            raw.extend((used.len() as u32).to_le_bytes());
            raw.extend(used.iter().flat_map(|v| v.to_le_bytes()));
        }
        raw.extend(calls.get().to_le_bytes());
        let mut input = (c.p.len() as u32).to_le_bytes().to_vec();
        input.extend(c.k.to_le_bytes());
        input.push(c.mode);
        input.extend(c.p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/plane-clipped.cjs"))
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
        assert_eq!(raw[0], original.stdout[0], "{}: presence", c.id);
        if result.is_some() {
            for (a, b) in raw[1..65]
                .as_chunks::<8>()
                .0
                .iter()
                .zip(original.stdout[1..65].as_chunks::<8>().0)
            {
                let a = f64::from_le_bytes(*a);
                let b = f64::from_le_bytes(*b);
                assert!(
                    (a - b).abs() <= 1e-12 * (1. + b.abs()),
                    "{}: numeric {a} vs {b}",
                    c.id
                );
            }
            assert_eq!(
                raw[65..],
                original.stdout[65..],
                "{}: used IDs/count/callbacks",
                c.id
            );
        } else {
            assert_eq!(raw, original.stdout, "{}: callback state", c.id);
        }
        println!(
            "{}: fit={}, full statistics and ordered used IDs compared",
            c.id,
            result.is_some()
        );
    }
}
