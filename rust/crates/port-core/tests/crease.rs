// SPDX-License-Identifier: AGPL-3.0-only
//! Independent full prepared-array controls; no reference output enters native geometry.
#![allow(clippy::unwrap_used, clippy::panic)]
use port_core::{
    adjacency::build_adjacency,
    crease::{self, Limits, Mode, Report, Skipped, Split},
    mesh_graph::MeshGraph,
    normals::compute_vertex_normals,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/scan-render/cases.tsv");
fn mesh(kind: usize, seed: u32) -> (Vec<f32>, Vec<u32>) {
    let (mut p, mut ix) = (Vec::<f64>::new(), vec![]);
    let mut state = seed;
    let mut random = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        f64::from(state) / 4294967296.
    };
    match kind {
        0 => {
            p = vec![
                -1., -1., -1., 1., -1., -1., 1., 1., -1., -1., 1., -1., -1., -1., 1., 1., -1., 1.,
                1., 1., 1., -1., 1., 1.,
            ];
            ix = vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2,
                7, 6, 3, 0, 4, 3, 4, 7,
            ]
        }
        1 | 2 => {
            p = vec![
                0.,
                0.,
                0.,
                1.,
                0.,
                0.,
                0.,
                1.,
                0.,
                1.,
                1.,
                if kind == 1 { 0. } else { 1. },
            ];
            ix = vec![0, 1, 2, 1, 3, 2]
        }
        3 | 4 => {
            let n = if kind == 3 { 7 } else { 129 };
            p.extend([0., 0., 0.]);
            for k in 0..n {
                p.extend([(k % 11) as f64, (k / 11) as f64, (k % 3) as f64]);
                ix.extend([0, k + 1, (k + 1) % n + 1])
            }
        }
        5 => {
            p = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 2., 2., 2.];
            ix = vec![0, 0, 1, 0, 1, 2, 2, 1, 0, 0, 1, 2]
        }
        6 => {
            p = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., -1., 1., 0., 1., 1.];
            ix = vec![0, 1, 2, 1, 0, 3, 0, 1, 4, 0, 1, 2]
        }
        8 => {
            for k in 0..9 {
                p.extend([(k % 3) as f64, (k / 3) as f64, (k % 2) as f64]);
            }
            for a in 0..9 {
                for b in a + 1..9 {
                    for c in b + 1..9 {
                        ix.extend([a, b, c]);
                    }
                }
            }
        }
        11 => {
            for t in 0..40000 {
                let a = t * 4;
                let x = f64::from(t * 3);
                p.extend([x, 0., 0., x + 1., 0., 0., x, 1., 0., x, 0., 1.]);
                ix.extend([
                    a,
                    a + 2,
                    a + 1,
                    a,
                    a + 1,
                    a + 3,
                    a,
                    a + 3,
                    a + 2,
                    a + 1,
                    a + 2,
                    a + 3,
                ]);
            }
        }
        9 => {}
        10 => {
            p = vec![1., 2., 3.];
        }
        _ => {
            let n = 9;
            for y in 0..n {
                for x in 0..n {
                    p.extend([x as f64, y as f64, (random() - 0.5) * 0.2])
                }
            }
            for y in 0..n - 1 {
                for x in 0..n - 1 {
                    let a = y * n + x;
                    ix.extend([a, a + 1, a + n, a + 1, a + n + 1, a + n])
                }
            }
        }
    }
    (p.into_iter().map(|x| x as f32).collect(), ix)
}
fn u(out: &mut Vec<u8>, x: usize) {
    out.extend((x as u32).to_le_bytes())
}
fn floats(out: &mut Vec<u8>, v: &[f32]) {
    u(out, v.len());
    for x in v {
        out.extend(x.to_le_bytes())
    }
}
fn words(out: &mut Vec<u8>, v: &[u32]) {
    u(out, v.len());
    for x in v {
        out.extend(x.to_le_bytes())
    }
}
fn split(out: &mut Vec<u8>, s: Option<&Split>) {
    u(out, usize::from(s.is_some()));
    if let Some(s) = s {
        floats(out, &s.positions);
        words(out, &s.indices);
        floats(out, &s.normals);
        words(
            out,
            &s.wire_slots
                .iter()
                .map(|&x| u32::from(x))
                .collect::<Vec<_>>(),
        );
        words(out, &s.copy_of)
    }
}
fn report(out: &mut Vec<u8>, r: &Report) {
    u(out, r.added);
    u(
        out,
        match r.skipped {
            None => 0,
            Some(Skipped::Off) => 1,
            Some(Skipped::Scan) => 2,
            Some(Skipped::Budget) => 3,
        },
    )
}
fn observe(row: &str) -> Vec<u8> {
    let c = row.split('\t').collect::<Vec<_>>();
    let (p, ix) = mesh(c[1].parse().unwrap(), c[2].parse().unwrap());
    let n = p.len() / 3;
    let limits = Limits::default();
    let adj = build_adjacency(&ix, n, limits.vertices, limits.indices * 2).unwrap();
    let slots = crease::wire_slots(&adj, n, limits).unwrap();
    let mut out = vec![];
    words(
        &mut out,
        &slots.iter().map(|&x| u32::from(x)).collect::<Vec<_>>(),
    );
    u(&mut out, crease::wire_conflicts(&ix, &slots).unwrap());
    let share = crease::coplanar_share(&p, &ix, limits).unwrap();
    out.extend(share.to_le_bytes());
    u(&mut out, usize::from(share >= 0.1));
    u(&mut out, crease::crease_budget(n));
    u(&mut out, crease::crease_budget(2_000_000));
    let angle = c[4].parse().unwrap();
    let s = crease::split_creases(&p, &ix, &slots, angle, c[5].parse().unwrap(), limits).unwrap();
    split(&mut out, s.as_ref());
    let mode = match c[3] {
        "on" => Mode::On,
        "off" => Mode::Off,
        _ => Mode::Auto,
    };
    let (s, r) = crease::crease_for(mode, &p, &ix, &slots, angle, limits).unwrap();
    split(&mut out, s.as_ref());
    report(&mut out, &r);
    let normals = compute_vertex_normals(&p, &ix, limits.vertices).unwrap();
    let g = MeshGraph {
        positions: p,
        indices: ix,
        normals,
        adjacency: adj,
        vertex_count: n,
        triangle_count: 0,
        bbox_diag: 0.,
    };
    let (s, r) = crease::scan_render(&g, mode, angle, limits).unwrap();
    split(&mut out, Some(&s));
    report(&mut out, &r);
    if std::env::var_os("SCANRULER_CREASE_SABOTAGE").is_some() {
        out[4] ^= 1
    }
    out
}
#[test]
fn native_complete_prepared_render_profiles() {
    assert_eq!(INPUT.lines().count(), 1309);
    for row in INPUT.lines() {
        assert!(!observe(row).is_empty())
    }
}
#[test]
#[ignore = "whole original modules require Node/TypeScript oracle tooling"]
fn pinned_scan_render_comparison() {
    use std::{
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root.join(format!("rust/target/scan-render-diff/run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(run.join("candidate.txt"), &sha.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(
        run.join("identity.txt"),
        format!(
            "reference: {}\ncandidate: {}dirty: {}",
            port_core::REFERENCE_SHA,
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/scan-render.cjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(INPUT.as_bytes()).unwrap());
    let source = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    let output = String::from_utf8(source.stdout).unwrap();
    assert_eq!(output.lines().count(), 1309);
    let mut mismatches = 0;
    for (row, original) in INPUT.lines().zip(output.lines()) {
        let (id, hex) = original.split_once('\t').unwrap();
        assert_eq!(id, row.split('\t').next().unwrap());
        let expected = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let actual = observe(row);
        let d = run.join(id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, bytes) in [
            ("input.bin", row.as_bytes()),
            ("original.bin", &expected),
            ("rust.bin", &actual),
            ("original.stderr.txt", &source.stderr),
        ] {
            std::fs::write(d.join(name), bytes).unwrap()
        }
        if expected != actual {
            mismatches += 1;
            if mismatches <= 4 {
                eprintln!("mismatch {id}")
            }
        }
        let mut wrong = expected.clone();
        wrong[4] ^= 1;
        assert_ne!(expected, wrong, "wrong actual corner slot must fail");
        assert_ne!(
            expected,
            &actual[..actual.len() - 1],
            "missing byte must fail"
        )
    }
    eprintln!(
        "1309 complete render controls; mismatches={mismatches}; raw {}",
        run.display()
    );
    assert_eq!(mismatches, 0)
}
#[test]
fn native_admission_keeps_source_budget_refusal_distinct() {
    let limits = Limits::default();
    let (p, ix) = mesh(0, 7);
    let adj = build_adjacency(&ix, p.len() / 3, limits.vertices, limits.indices * 2).unwrap();
    let slots = crease::wire_slots(&adj, p.len() / 3, limits).unwrap();
    assert!(
        crease::split_creases(&p, &ix, &slots, 30., 0, limits)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        crease::split_creases(
            &p,
            &ix,
            &slots,
            30.,
            250000,
            Limits {
                render_vertices: p.len() / 3,
                ..limits
            }
        ),
        Err(crease::Error::Limit)
    ));
    let bad = port_core::adjacency::Adjacency {
        offsets: vec![0, 1],
        list: vec![2],
    };
    assert!(matches!(
        crease::wire_slots(&bad, 1, limits),
        Err(crease::Error::Invalid)
    ));
    assert!(matches!(
        crease::split_creases(&p, &[99, 0, 1], &slots, 30., 1, limits),
        Err(crease::Error::Invalid)
    ));
}
