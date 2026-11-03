// SPDX-License-Identifier: AGPL-3.0-only
//! Pre-registered complete input/value/used-point/immutability comparisons.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::{
    clip::ClipError,
    sphere::{ClippedFit, Sphere, fit_sphere_clipped},
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/sphere/cases.tsv");
#[path = "common/sphere_input.rs"]
mod inputs;
use inputs::{model, packet};

fn sphere(s: Sphere, out: &mut Vec<u8>) {
    for n in [s.cx, s.cy, s.cz, s.r] {
        out.extend(n.to_le_bytes())
    }
}
fn output(r: Option<ClippedFit>) -> Vec<u8> {
    let Some(r) = r else { return vec![0] };
    let mut b = vec![1];
    sphere(r.sphere, &mut b);
    for n in [r.sigma, r.span, r.min, r.max] {
        b.extend(n.to_le_bytes())
    }
    b.extend((r.used.len() as u32).to_le_bytes());
    for i in &r.used {
        b.extend(i.to_le_bytes())
    }
    b.push(u8::from(r.free.is_some()));
    if let Some(free) = r.free {
        sphere(free, &mut b)
    }
    b
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn float(a: &[u8], b: &[u8], field: &str, id: &str) {
    let a = f64::from_le_bytes(a.try_into().unwrap());
    let b = f64::from_le_bytes(b.try_into().unwrap());
    if b.is_nan() {
        assert!(a.is_nan(), "{id}/{field}: original NaN vs {a}")
    } else if !b.is_finite() {
        assert_eq!(a, b, "{id}/{field}")
    } else if b == 0. && a == 0. {
        assert_eq!(a.to_bits(), b.to_bits(), "{id}/{field}: signed zero")
    } else {
        assert!(
            a.is_finite() && (a - b).abs() <= 1e-10 + 1e-10 * b.abs(),
            "{id}/{field}: {a} vs {b}"
        )
    }
}
fn compare(actual: &[u8], original: &[u8], id: &str) {
    assert_eq!(actual[0], original[0], "{id}: optional result");
    if original[0] == 0 {
        assert_eq!(actual, original);
        return;
    }
    assert_eq!(actual.len(), original.len(), "{id}: complete shape");
    for i in 0..8 {
        let start = 1 + i * 8;
        float(
            &actual[start..start + 8],
            &original[start..start + 8],
            &format!("field{i}"),
            id,
        )
    }
    let count = u32::from_le_bytes(original[65..69].try_into().unwrap()) as usize;
    let end = 69 + count * 4;
    assert_eq!(
        &actual[65..=end],
        &original[65..=end],
        "{id}: ordered used IDs/companion presence"
    );
    if original[end] == 1 {
        for i in 0..4 {
            let at = end + 1 + i * 8;
            float(
                &actual[at..at + 8],
                &original[at..at + 8],
                &format!("free{i}"),
                id,
            )
        }
    }
}
#[test]
fn analytic_sphere_and_host_guards() {
    let m = model("probe\t5\t1\t0\tnone");
    let before = packet(&m);
    let r = fit_sphere_clipped(&m.p, &m.idx, m.k, m.held, 100)
        .unwrap()
        .unwrap();
    assert!(r.sphere.cx.abs() < 1e-12);
    assert!((r.sphere.r - 3f64.sqrt()).abs() < 1e-12);
    assert_eq!(r.used, m.idx);
    assert!(r.free.is_none());
    assert_eq!(packet(&m), before);
    assert_eq!(
        fit_sphere_clipped(&[0.], &[], 0., None, 10).unwrap_err(),
        ClipError::InvalidInput
    );
    assert_eq!(
        fit_sphere_clipped(&[0.; 3], &[1], 0., None, 10).unwrap_err(),
        ClipError::InvalidInput
    );
    assert_eq!(
        fit_sphere_clipped(&m.p, &m.idx, 0., None, 3).unwrap_err(),
        ClipError::InvalidInput
    );
    let held = fit_sphere_clipped(&m.p, &m.idx, 0., Some(10.), 100)
        .unwrap()
        .unwrap();
    assert_eq!(held.sphere.r, 10.);
    assert!(held.free.is_some());
}
#[test]
#[ignore = "complete original sphere modules require Node22/typescript"]
fn pinned_sphere_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/sphere.cjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let source = child.wait_with_output().unwrap();
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    assert!(source.stderr.is_empty());
    let source = String::from_utf8(source.stdout).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 1560);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/sphere-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let candidate = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &candidate.stdout).unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&candidate.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let mut success = 0;
    for (descriptor, line) in INPUT.lines().zip(lines) {
        let p = line.split('\t').collect::<Vec<_>>();
        let m = model(descriptor);
        assert_eq!(p[0], m.id);
        let input = packet(&m);
        assert_eq!(input, unhex(p[1]), "{}: independent input model", m.id);
        assert_eq!(p[1], p[3], "{}: original mutation", m.id);
        let original = unhex(p[2]);
        let actual = output(fit_sphere_clipped(&m.p, &m.idx, m.k, m.held, 10000).unwrap());
        let after = packet(&m);
        assert_eq!(input, after, "{}: native mutation", m.id);
        let case = dir.join(&m.id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, bytes) in [
            ("input.bin", input.as_slice()),
            ("descriptor.tsv", descriptor.as_bytes()),
            ("original.bin", original.as_slice()),
            ("rust.bin", actual.as_slice()),
            ("original.after.bin", unhex(p[3]).as_slice()),
            ("rust.after.bin", after.as_slice()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), bytes).unwrap()
        }
        compare(&actual, &original, &m.id);
        success += usize::from(original[0] == 1);
    }
    assert!(success > 0);
    println!(
        "1560 complete sphere input/value/statistic/used/free/refusal comparisons; {success} successful models; {}",
        dir.display()
    );
}

#[test]
fn comparator_rejects_changed_values_used_ids_presence_and_signed_zero() {
    let original = output(Some(ClippedFit {
        sphere: Sphere {
            cx: 0.,
            cy: 0.,
            cz: 0.,
            r: 1.,
        },
        sigma: 0.,
        span: 0.,
        min: 0.,
        max: 0.,
        used: vec![0, 1, 2, 3],
        free: None,
    }));
    for kind in 0..4 {
        let mut changed = original.clone();
        match kind {
            0 => changed[1..9].copy_from_slice(&1e-3f64.to_le_bytes()),
            1 => changed[69..73].copy_from_slice(&37u32.to_le_bytes()),
            2 => changed[0] = 0,
            _ => changed[1..9].copy_from_slice(&(-0.0f64).to_le_bytes()),
        }
        assert!(std::panic::catch_unwind(|| compare(&changed, &original, "canary")).is_err());
    }
}
