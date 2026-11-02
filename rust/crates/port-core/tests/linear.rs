// SPDX-License-Identifier: AGPL-3.0-only
//! Solution and mutated-matrix comparisons for both frozen dense solvers.
#![allow(clippy::unwrap_used)]
use port_core::linear::{DEFAULT_CHOLESKY_TINY, SolveError, cholesky_solve, solve_linear};
struct Case {
    id: String,
    n: usize,
    a: Vec<f64>,
    b: Vec<f64>,
}
fn cases() -> Vec<Case> {
    let mut out = vec![
        Case {
            id: "empty".into(),
            n: 0,
            a: vec![],
            b: vec![],
        },
        Case {
            id: "pivot".into(),
            n: 2,
            a: vec![0., 2., 1., 3.],
            b: vec![4., 7.],
        },
        Case {
            id: "spd".into(),
            n: 2,
            a: vec![4., 1., 1., 3.],
            b: vec![1., 2.],
        },
        Case {
            id: "singular".into(),
            n: 2,
            a: vec![1., 2., 2., 4.],
            b: vec![1., 2.],
        },
        Case {
            id: "tiny".into(),
            n: 1,
            a: vec![1e-14],
            b: vec![1.],
        },
        Case {
            id: "threshold".into(),
            n: 1,
            a: vec![1e-12],
            b: vec![1.],
        },
        Case {
            id: "lower_only".into(),
            n: 2,
            a: vec![4., 1039., 1., 3.],
            b: vec![1., 2.],
        },
    ];
    for seed in [1u32, 7, 99, 1039] {
        let n = 5;
        let mut state = seed;
        let mut a = vec![0.; n * n];
        for r in 0..n {
            for c in r..n {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                let v = f64::from(state % 100) / 100.;
                a[r * n + c] = v;
                a[c * n + r] = v;
            }
            a[r * n + r] += 10.;
        }
        out.push(Case {
            id: format!("seed_{seed}"),
            n,
            a,
            b: vec![1., 2., 3., 4., 5.],
        });
    }
    out
}
#[test]
fn shape_and_input_contract() {
    let mut a = vec![0., 2., 1., 3.];
    let b = vec![4., 7.];
    assert_eq!(solve_linear(2, &mut a, &b, 2).unwrap(), Some(vec![1., 2.]));
    assert_eq!(b, [4., 7.]);
    assert_eq!(
        cholesky_solve(1, &[1e-12], &[1.], DEFAULT_CHOLESKY_TINY, 1).unwrap(),
        None
    );
    let before = a.clone();
    assert_eq!(
        solve_linear(2, &mut a, &b, 1),
        Err(SolveError::ShapeOrLimit)
    );
    assert_eq!(a, before);
}
#[test]
#[ignore = "requires frozen live Node oracle"]
fn pinned_linear_comparison() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/linear-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    println!("raw linear evidence: {}", dir.display());
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
    let fixtures = cases();
    assert_eq!(fixtures.len(), 11);
    for case in fixtures {
        for mode in [0u8, 1] {
            let id = format!("{}_{}", case.id, mode);
            let mut input = (case.n as u32).to_le_bytes().to_vec();
            input.push(mode);
            input.extend(DEFAULT_CHOLESKY_TINY.to_le_bytes());
            input.extend(case.a.iter().chain(&case.b).flat_map(|v| v.to_le_bytes()));
            let mut a = case.a.clone();
            let x = if mode == 0 {
                solve_linear(case.n, &mut a, &case.b, 10).unwrap()
            } else {
                cholesky_solve(case.n, &a, &case.b, DEFAULT_CHOLESKY_TINY, 10).unwrap()
            };
            let mut raw = vec![u8::from(x.is_some())];
            if let Some(x) = &x {
                raw.extend(x.iter().flat_map(|v| v.to_le_bytes()));
            }
            raw.extend(a.iter().flat_map(|v| v.to_le_bytes()));
            let mut child = Command::new("node")
                .arg(root.join("docs/port-plan/oracle/linear-solvers.cjs"))
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
            assert!(
                raw == original.stdout,
                "{id}: solution/refusal/matrix differs"
            );
            println!(
                "{id}: success={}, full solution + matrix bytes compared",
                x.is_some()
            );
        }
    }
}
