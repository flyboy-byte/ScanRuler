// SPDX-License-Identifier: AGPL-3.0-only
//! Original-versus-Rust cylinder fitting on the pre-generated corpus.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::{
    circle2d::fit_circle_2d,
    cylinder::{
        Cylinder, axis_from_normals, fit_cylinder_clipped, fit_cylinder_on_axis, ransac_cylinder,
        refine_cylinder_geometric,
    },
    ransac::{RansacLimits, RansacOptions},
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/cylinder/cases.tsv");

struct Case {
    id: String,
    op: String,
    params: Vec<f64>,
    p: Vec<f32>,
    n: Vec<f32>,
    idx: Vec<u32>,
}

fn number(s: &str) -> f64 {
    match s {
        "nan" => f64::NAN,
        "inf" => f64::INFINITY,
        "-inf" => f64::NEG_INFINITY,
        _ => s.parse().unwrap(),
    }
}
fn floats(h: &str) -> Vec<f32> {
    unhex(h)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn case(line: &str) -> Case {
    let f: Vec<_> = line.split('\t').collect();
    fn list(s: &str) -> Vec<&str> {
        if s.is_empty() {
            vec![]
        } else {
            s.split(',').collect()
        }
    }
    Case {
        id: f[0].into(),
        op: f[1].into(),
        params: list(f[2]).into_iter().map(number).collect(),
        p: floats(f[3]),
        n: floats(f[4]),
        idx: list(f[5]).into_iter().map(|s| s.parse().unwrap()).collect(),
    }
}
fn cyl(a: &[f64]) -> Cylinder {
    Cylinder {
        px: a[0],
        py: a[1],
        pz: a[2],
        ax: a[3],
        ay: a[4],
        az: a[5],
        r: a[6],
    }
}
fn put_cyl(b: &mut Vec<u8>, c: &Cylinder) {
    for x in [c.px, c.py, c.pz, c.ax, c.ay, c.az, c.r] {
        b.extend(x.to_le_bytes())
    }
}
fn ids(b: &mut Vec<u8>, v: &[u32]) {
    b.extend((v.len() as u32).to_le_bytes());
    for i in v {
        b.extend(i.to_le_bytes())
    }
}
fn run(c: &Case) -> Vec<u8> {
    let mut b = vec![];
    match c.op.as_str() {
        "clipped" => match fit_cylinder_clipped(&c.p, &c.idx, cyl(&c.params), c.params[7], 100_000)
            .unwrap()
        {
            None => b.push(0),
            Some(r) => {
                b.push(1);
                put_cyl(&mut b, &r.cylinder);
                for x in [r.sigma, r.span, r.min, r.max] {
                    b.extend(x.to_le_bytes())
                }
                ids(&mut b, &r.used)
            }
        },
        "refine" => put_cyl(
            &mut b,
            &refine_cylinder_geometric(&c.p, &c.idx, cyl(&c.params), c.params[7] as usize).unwrap(),
        ),
        "onaxis" => {
            match fit_cylinder_on_axis(&c.p, &c.idx, [c.params[0], c.params[1], c.params[2]])
                .unwrap()
            {
                None => b.push(0),
                Some(r) => {
                    b.push(1);
                    put_cyl(&mut b, &r)
                }
            }
        }
        "axis" => match axis_from_normals(&c.n, &c.idx).unwrap() {
            None => b.push(0),
            Some(a) => {
                b.push(1);
                for x in a {
                    b.extend(x.to_le_bytes())
                }
            }
        },
        "ransac" => match ransac_cylinder(
            &c.p,
            &c.n,
            &c.idx,
            RansacOptions {
                seed: c.params[0] as u32,
                iterations: c.params[1] as usize,
            },
            RansacLimits {
                max_points: 100_000,
                max_iterations: 10_000,
            },
        )
        .unwrap()
        {
            None => b.push(0),
            Some(r) => {
                b.push(1);
                put_cyl(&mut b, &r.cylinder);
                b.extend(r.sigma.to_le_bytes());
                ids(&mut b, &r.inliers)
            }
        },
        "circle" => {
            let pu: Vec<f64> = c
                .idx
                .iter()
                .map(|&i| f64::from(c.p[i as usize * 3]))
                .collect();
            let pv: Vec<f64> = c
                .idx
                .iter()
                .map(|&i| f64::from(c.p[i as usize * 3 + 1]))
                .collect();
            match fit_circle_2d(&pu, &pv).unwrap() {
                None => b.push(0),
                Some(r) => {
                    b.push(1);
                    for x in [r.cu, r.cv, r.r] {
                        b.extend(x.to_le_bytes())
                    }
                }
            }
        }
        op => panic!("unknown op {op}"),
    }
    b
}

#[test]
fn analytic_cylinder_recovers_radius_and_axis() {
    let mut p = vec![];
    for i in 0..200 {
        let th = f64::from(i) * 0.1;
        let z = f64::from(i % 20) - 10.;
        p.extend([(10. * th.cos()) as f32, (10. * th.sin()) as f32, z as f32]);
    }
    let idx: Vec<u32> = (0..200).collect();
    let init = Cylinder {
        px: 0.5,
        py: -0.3,
        pz: 0.,
        ax: 0.05,
        ay: 0.,
        az: 1.,
        r: 9.,
    };
    let r = fit_cylinder_clipped(&p, &idx, init, 0., 1000)
        .unwrap()
        .unwrap();
    assert!((r.cylinder.r - 10.).abs() < 1e-5, "{:?}", r.cylinder);
    assert!(r.cylinder.az.abs() > 1. - 1e-9);
    assert!(fit_cylinder_clipped(&p, &[999], init, 0., 1000).is_err());
}

/// The pinned original's cylinder.ts on the same rows; every result must be
/// bit-identical (floats, used/inlier order and the None/Some shape).
#[test]
#[ignore = "pinned original cylinder modules require Node 22 and typescript"]
fn pinned_cylinder_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/cylinder.cjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(INPUT.as_bytes()).unwrap());
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let original = String::from_utf8(out.stdout).unwrap();
    let rows: Vec<_> = INPUT.lines().collect();
    assert_eq!(original.lines().count(), rows.len());
    let mut failures = vec![];
    let (mut some, mut none) = (0, 0);
    for (row, line) in rows.iter().zip(original.lines()) {
        let c = case(row);
        let (id, hex) = line.split_once('\t').unwrap();
        assert_eq!(id, c.id);
        let expected = unhex(hex);
        let actual = run(&c);
        if c.op != "refine" {
            if expected[0] == 1 {
                some += 1
            } else {
                none += 1
            }
        }
        if expected != actual {
            failures.push(format!("{} ({})", c.id, c.op));
        }
    }
    eprintln!(
        "{} cylinder comparisons ({some} fitted, {none} refused, rest refine); mismatches: {failures:?}",
        rows.len()
    );
    assert!(failures.is_empty());
}
