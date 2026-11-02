// SPDX-License-Identifier: AGPL-3.0-only
//! Full source plane element, original region and fixed-angle numerical probes.
#![allow(clippy::unwrap_used)]
use port_core::marked_plane::{MarkedPlaneError, fit_plane_on_selection};
struct Case {
    id: String,
    p: Vec<f32>,
    normals: Vec<f32>,
    idx: Vec<u32>,
    sigma: f64,
}
fn make(id: &str, p: Vec<f32>, idx: Vec<u32>, vote: f32, sigma: f64) -> Case {
    let normals = (0..p.len() / 3).flat_map(|_| [0., 0., vote]).collect();
    Case {
        id: id.into(),
        p,
        normals,
        idx,
        sigma,
    }
}
fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for n in [0, 1, 19] {
        out.push(make(
            &format!("insufficient_{n}"),
            vec![0.; n * 3],
            (0..n as u32).collect(),
            1.,
            2.,
        ));
    }
    out.push(make("coincident", vec![1.; 60], (0..20).collect(), 1., 2.));
    out.push(make(
        "collinear",
        (0..20)
            .flat_map(|i| [i as f32, 2. * i as f32, 3. * i as f32])
            .collect(),
        (0..20).collect(),
        1.,
        2.,
    ));
    out.push(make(
        "duplicate_collapsed",
        vec![0.; 3],
        vec![0; 20],
        1.,
        2.,
    ));
    out.push(make(
        "duplicate_valid",
        vec![0., 0., 0., 4., 0., 0., 0., 3., 0.],
        (0..20).map(|i| i % 3).collect(),
        1.,
        2.,
    ));
    for (name, vote) in [("up", 1.), ("down", -1.), ("zero", 0.), ("nan", f32::NAN)] {
        let p = (0..100)
            .flat_map(|i| [(i % 10) as f32, (i / 10) as f32, 3.])
            .collect();
        out.push(make(
            &format!("square_{name}"),
            p,
            (0..100).collect(),
            vote,
            0.,
        ));
    }
    for tilted in [false, true] {
        for vote in [1., -1.] {
            let a = std::f64::consts::PI / 6.;
            let p = (0..160)
                .flat_map(|i| {
                    let u = (i % 16) as f64;
                    let v = (i / 16) as f64;
                    let x = (u * a.cos() - v * a.sin() - 10.) as f32;
                    let y = (u * a.sin() + v * a.cos() + 7.) as f32;
                    [x, y, if tilted { 0.2 * x - 0.7 * y + 4. } else { 4. }]
                })
                .collect();
            out.push(make(
                &format!("rotated_t{tilted}_vote{vote}"),
                p,
                (0..160).collect(),
                vote,
                2.,
            ));
        }
    }
    for seed in [1u32, 7, 99, 1039] {
        for (label, sigma) in [
            ("none", 0.),
            ("tight", 1.5),
            ("normal", 3.),
            ("nan", f64::NAN),
        ] {
            for vote in [1., -1.] {
                let mut state = seed;
                let p = (0..400)
                    .flat_map(|i| {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        let mut x = (i % 20) as f32;
                        let y = (i / 20) as f32;
                        let mut z = (state % 1001) as f32 / 100000. - 0.005;
                        if i >= 396 {
                            x += 50.;
                            z += 4.;
                        }
                        [x, y, z]
                    })
                    .collect();
                out.push(make(
                    &format!("outliers_seed_{seed}_{label}_vote{vote}"),
                    p,
                    (0..400).collect(),
                    vote,
                    sigma,
                ));
            }
        }
    }
    for n in [4095, 4096, 4097, 8193] {
        let p = (0..n)
            .flat_map(|i| {
                [
                    (i % 97) as f32,
                    (i / 97) as f32,
                    0.125 * (i % 97) as f32 - 0.25 * (i / 97) as f32 + 2.,
                ]
            })
            .collect();
        out.push(make(&format!("sampling_{n}"), p, (0..n).collect(), 1., 0.));
    }
    out
}
fn text(out: &mut Vec<u8>, s: &str) {
    out.extend((s.len() as u32).to_le_bytes());
    out.extend(s.as_bytes());
}
fn output(c: &Case) -> (Vec<u8>, bool) {
    let result = fit_plane_on_selection(&c.p, &c.normals, &c.idx, c.sigma, 10000);
    let mut out = Vec::new();
    let mut clipped = false;
    match result {
        Ok(r) => {
            out.push(0);
            text(&mut out, "plane");
            for v in r
                .center
                .into_iter()
                .chain(r.normal)
                .chain(r.basis_u)
                .chain(r.basis_v)
                .chain([
                    r.extent_u,
                    r.extent_v,
                    r.sigma,
                    r.form_error,
                    r.residual_min,
                    r.residual_max,
                ])
            {
                out.extend(v.to_le_bytes());
            }
            out.extend((r.used_points as u32).to_le_bytes());
            out.extend((r.region_size as u32).to_le_bytes());
            out.extend((r.region.len() as u32).to_le_bytes());
            out.extend(r.region.iter().flat_map(|i| i.to_le_bytes()));
            clipped = r.used_points < r.region_size;
        }
        Err(e) => {
            let MarkedPlaneError::Source(message) = e else {
                unreachable!("unexpected host failure: {e:?}")
            };
            out.push(1);
            text(&mut out, &message);
        }
    }
    for t in 0..45 {
        let a = (t as f64 / 45.) * (std::f64::consts::PI / 2.);
        out.extend(a.cos().to_le_bytes());
        out.extend(a.sin().to_le_bytes());
    }
    (out, clipped)
}
fn numeric(a: &[u8], b: &[u8], id: &str) -> f64 {
    assert_eq!(a.len(), b.len(), "{id}: numeric length");
    assert!(a.len().is_multiple_of(8));
    let mut worst = 0.0f64;
    for (i, (x, y)) in a
        .as_chunks::<8>()
        .0
        .iter()
        .zip(b.as_chunks::<8>().0)
        .enumerate()
    {
        let x = f64::from_le_bytes(*x);
        let y = f64::from_le_bytes(*y);
        assert!(
            x.is_finite() && y.is_finite(),
            "{id} field{i}: nonfinite {x}/{y}"
        );
        if x == 0. && y == 0. {
            assert_eq!(x.to_bits(), y.to_bits(), "{id}: zero sign field{i}");
        }
        let e = (x - y).abs();
        assert!(
            e <= 1e-12 * (1. + y.abs()),
            "{id} field{i}: {x} vs {y} (error {e})"
        );
        worst = worst.max(e);
    }
    worst
}
#[test]
fn source_guard_order_clipping_region_and_host_limits() {
    assert!(matches!(
        fit_plane_on_selection(&[], &[], &[u32::MAX], 2., 0),
        Err(MarkedPlaneError::Source(_))
    ));
    assert_eq!(
        fit_plane_on_selection(&[0.; 3], &[0.; 3], &[1; 20], 2., 20).unwrap_err(),
        MarkedPlaneError::InvalidInput
    );
    let p: Vec<_> = (0..100)
        .flat_map(|i| [(i % 10) as f32, (i / 10) as f32, 3.])
        .collect();
    let n: Vec<_> = (0..100).flat_map(|_| [0., 0., -1.]).collect();
    let idx: Vec<_> = (0..100).collect();
    let r = fit_plane_on_selection(&p, &n, &idx, 0., 100).unwrap();
    assert_eq!(r.region, idx);
    assert_eq!(r.region_size, 100);
    assert!(r.normal[2] < 0.);
    assert_eq!(
        fit_plane_on_selection(&p, &n, &idx, 0., 99).unwrap_err(),
        MarkedPlaneError::InvalidInput
    );
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_marked_plane_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/marked-plane-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    println!("raw marked-plane evidence: {}", dir.display());
    let cases = cases();
    assert_eq!(cases.len(), 51);
    let (mut success, mut refused, mut clipped) = (0, 0, 0);
    for c in cases {
        let (raw, was_clipped) = output(&c);
        let mut input = (c.p.len() as u32).to_le_bytes().to_vec();
        input.extend(c.sigma.to_le_bytes());
        input.extend(c.p.iter().chain(&c.normals).flat_map(|v| v.to_le_bytes()));
        input.extend(c.idx.iter().flat_map(|v| v.to_le_bytes()));
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/marked-plane.cjs"))
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
        for (name, bytes) in [
            ("input.bin", input.as_slice()),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", raw.as_slice()),
        ] {
            std::fs::write(d.join(name), bytes).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        let source = &original.stdout;
        assert_eq!(raw.len(), source.len(), "{}: output length", c.id);
        assert_eq!(raw[0], source[0], "{}: success/refusal", c.id);
        let prefix = 5 + u32::from_le_bytes(raw[1..5].try_into().unwrap()) as usize;
        assert_eq!(&raw[..prefix], &source[..prefix], "{}: kind/error", c.id);
        let mut worst = 0.0f64;
        if raw[0] == 0 {
            success += 1;
            let end = prefix + 18 * 8;
            worst = worst.max(numeric(&raw[prefix..end], &source[prefix..end], &c.id));
            assert_eq!(
                &raw[end..raw.len() - 90 * 8],
                &source[end..source.len() - 90 * 8],
                "{}: counts/original region",
                c.id
            );
        } else {
            refused += 1;
        }
        worst = worst.max(numeric(
            &raw[raw.len() - 90 * 8..],
            &source[source.len() - 90 * 8..],
            &c.id,
        ));
        clipped += usize::from(was_clipped);
        println!(
            "{}: full result/region + 45 angle probes; max abs {worst:e}",
            c.id
        );
    }
    assert_eq!((success, refused), (45, 6));
    assert!(clipped > 0);
    println!(
        "marked plane: {success} successes, {refused} source refusals, {clipped} clipped results retaining original region"
    );
}
