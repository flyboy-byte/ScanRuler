// SPDX-License-Identifier: AGPL-3.0-only
//! Exact deterministic draws, consensus tie state and robust fitted plane outputs.
#![allow(clippy::unwrap_used)]
use port_core::{
    plane::Plane,
    ransac::{
        ConsensusShape, Mulberry32, RansacError, RansacLimits, RansacOptions, ransac_consensus,
        ransac_plane,
    },
};
struct Case {
    id: String,
    p: Vec<f32>,
    idx: Vec<u32>,
    seed: u32,
    iterations: usize,
    mode: u8,
}
fn cases() -> Vec<Case> {
    let mut out = vec![
        Case {
            id: "insufficient".into(),
            p: vec![0.; 87],
            idx: (0..29).collect(),
            seed: 1,
            iterations: 256,
            mode: 1,
        },
        Case {
            id: "collapsed".into(),
            p: vec![1.; 180],
            idx: (0..60).collect(),
            seed: 1,
            iterations: 256,
            mode: 1,
        },
        Case {
            id: "collinear".into(),
            p: (0..60).flat_map(|i| [i as f32, 0., 0.]).collect(),
            idx: (0..60).collect(),
            seed: 1,
            iterations: 256,
            mode: 1,
        },
    ];
    for seed in [1u32, 7, 99, 1039] {
        for kind in 0..3 {
            for iterations in [0, 32, 256] {
                let mut state = seed;
                let mut p = Vec::new();
                let (nx, ny) = if kind == 2 { (40, 30) } else { (20, 20) };
                for y in 0..ny {
                    for x in 0..nx {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        let noise = (state % 100) as f32 / 5000. - 0.01;
                        if kind == 2 {
                            if x < 27 {
                                p.extend_from_slice(&[x as f32, y as f32, 7. + noise]);
                            } else {
                                p.extend_from_slice(&[27., y as f32, (x - 27) as f32 + noise]);
                            }
                        } else {
                            let z = 2. * x as f32 - 3. * y as f32
                                + 7.
                                + if kind == 1 {
                                    noise + if state % 5 == 0 { 5. } else { 0. }
                                } else {
                                    0.
                                };
                            p.extend_from_slice(&[x as f32, y as f32, z]);
                        }
                    }
                }
                out.push(Case {
                    id: format!("seed_{seed}_kind{kind}_it{iterations}"),
                    idx: (0..(p.len() / 3) as u32).collect(),
                    p,
                    seed,
                    iterations,
                    mode: 1,
                });
            }
        }
    }
    for iterations in [0, 32, 256] {
        out.push(Case {
            id: format!("initial_tie_it{iterations}"),
            p: (0..60).flat_map(|i| [0., 0., i as f32]).collect(),
            idx: (0..60).collect(),
            seed: 7,
            iterations,
            mode: 2,
        });
    }
    out
}
struct Tie {
    calls: u32,
}
impl ConsensusShape for Tie {
    type Model = Plane;
    fn prepare(&mut self, _: f64, _: &mut Mulberry32) -> Result<Option<Plane>, RansacError> {
        Ok(Some(Plane {
            normal: [1., 0., 0.],
            d: 0.,
        }))
    }
    fn generate(&mut self, _: &mut Mulberry32) -> Result<Option<Plane>, RansacError> {
        self.calls += 1;
        Ok(Some(Plane {
            normal: [0., 1., 0.],
            d: 0.,
        }))
    }
    fn residual(&self, m: &Plane, x: f64, y: f64, z: f64) -> f64 {
        m.residual(x, y, z)
    }
}
#[test]
fn host_guards() {
    let limits = RansacLimits {
        max_points: 100,
        max_iterations: 256,
    };
    assert!(matches!(
        ransac_plane(&[0.; 3], &[1], RansacOptions::default(), limits),
        Err(RansacError::InvalidInput)
    ));
    assert!(matches!(
        ransac_plane(
            &[],
            &[],
            RansacOptions {
                iterations: 257,
                seed: 1
            },
            limits
        ),
        Err(RansacError::Limit)
    ));
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_ransac_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/ransac-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw robust-plane evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let compare = |id: &str, input: Vec<u8>, raw: Vec<u8>, rng: bool, present: bool| {
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/ransac-plane.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let d = dir.join(id);
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
        assert_eq!(raw.len(), original.stdout.len(), "{id}: lengths");
        if rng || !present {
            assert_eq!(raw, original.stdout, "{id}: exact state");
        } else {
            assert_eq!(raw[0], original.stdout[0], "{id}: presence");
            for (a, b) in raw[1..41]
                .as_chunks::<8>()
                .0
                .iter()
                .zip(original.stdout[1..41].as_chunks::<8>().0)
            {
                let a = f64::from_le_bytes(*a);
                let b = f64::from_le_bytes(*b);
                assert!(
                    (a - b).abs() <= 1e-12 * (1. + b.abs()),
                    "{id}: numeric {a} vs {b}"
                );
            }
            assert_eq!(
                raw[41..],
                original.stdout[41..],
                "{id}: ordered inliers/count/generate calls"
            );
        }
        println!("{id}: full values/state compared");
    };
    for seed in [0u32, 1, 7, 99, 1039, 0x5eed, 0x80000000, u32::MAX] {
        let mut input = vec![0];
        input.extend(0u32.to_le_bytes());
        input.extend(seed.to_le_bytes());
        input.extend(1024u32.to_le_bytes());
        let mut rng = Mulberry32::new(seed);
        let raw = (0..1024)
            .flat_map(|_| rng.next_f64().to_le_bytes())
            .collect();
        compare(&format!("rng_{seed}"), input, raw, true, false);
    }
    let fixtures = cases();
    assert_eq!(fixtures.len(), 42);
    for c in fixtures {
        let opts = RansacOptions {
            iterations: c.iterations,
            seed: c.seed,
        };
        let limits = RansacLimits {
            max_points: 2000,
            max_iterations: 256,
        };
        let mut calls = 0;
        let result = if c.mode == 2 {
            let mut shape = Tie { calls: 0 };
            let result = ransac_consensus(&c.p, &c.idx, opts, limits, &mut shape)
                .unwrap()
                .map(|r| (r.model, r.sigma_est, r.inliers));
            calls = shape.calls;
            result
        } else {
            ransac_plane(&c.p, &c.idx, opts, limits)
                .unwrap()
                .map(|r| (r.plane, r.sigma, r.inliers))
        };
        let present = result.is_some();
        let mut raw = vec![u8::from(present)];
        if let Some((p, sigma, inliers)) = result {
            for v in p.normal.into_iter().chain([p.d, sigma]) {
                raw.extend(v.to_le_bytes());
            }
            raw.extend((inliers.len() as u32).to_le_bytes());
            raw.extend(inliers.iter().flat_map(|v| v.to_le_bytes()));
        }
        raw.extend(calls.to_le_bytes());
        let mut input = vec![c.mode];
        input.extend((c.p.len() as u32).to_le_bytes());
        input.extend(c.seed.to_le_bytes());
        input.extend((c.iterations as u32).to_le_bytes());
        input.extend(c.p.iter().flat_map(|v| v.to_bits().to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        compare(&c.id, input, raw, false, present);
    }
}
