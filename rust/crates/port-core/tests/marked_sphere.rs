// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original marked-sphere output, region identity and exact FitErrors.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::marked_sphere::{Error, MarkedSphere, Settings, fit_sphere_on_selection};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};
#[path = "common/sphere_input.rs"]
mod inputs;
use inputs::{Model, packet};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/marked-sphere/cases.tsv");
fn model(line: &str) -> Model {
    let mut m = inputs::model(line);
    let f = line.split('\t').collect::<Vec<_>>();
    let mode: usize = f[1].parse().unwrap();
    if mode >= 16 {
        let r = if mode == 17 {
            0.03125
        } else if mode == 18 {
            1.03125
        } else {
            10.
        };
        m.p = vec![
            r, 0., 0., -r, 0., 0., 0., r, 0., 0., -r, 0., 0., 0., r, 0., 0., -r,
        ];
        m.idx = vec![0, 1, 2, 3, 4, 5]
    }
    if f[5] != "default" {
        let previous = m.idx;
        let n: usize = f[5].parse().unwrap();
        m.idx = (0..n).map(|i| previous[i % previous.len()]).collect()
    }
    m
}
fn text(s: &str, out: &mut Vec<u8>) {
    out.extend((s.len() as u32).to_le_bytes());
    out.extend(s.as_bytes())
}
fn output(result: Result<MarkedSphere, Error>, selection: &Arc<[u32]>) -> Vec<u8> {
    let mut out = vec![];
    match result {
        Ok(r) => {
            out.push(0);
            text("sphere", &mut out);
            for n in r.center.into_iter().chain([
                r.radius,
                r.sigma,
                r.form_error,
                r.residual_min,
                r.residual_max,
            ]) {
                out.extend(n.to_le_bytes())
            }
            out.extend((r.used_points as u32).to_le_bytes());
            out.extend((r.region_size as u32).to_le_bytes());
            out.push(u8::from(r.held.is_some()));
            if let Some(h) = r.held {
                out.extend(h.free_diameter.to_le_bytes());
                out.extend(h.shift.to_le_bytes())
            }
            out.extend((r.region.len() as u32).to_le_bytes());
            for &i in r.region.iter() {
                out.extend(i.to_le_bytes())
            }
            out.push(u8::from(Arc::ptr_eq(&r.region, selection)))
        }
        Err(Error::Source(message)) => {
            out.push(1);
            text("Error", &mut out);
            out.push(1);
            text("FitError", &mut out);
            text(&message, &mut out)
        }
        other => panic!("unexpected host refusal: {other:?}"),
    }
    out
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|h| u8::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect()
}
fn scalar(a: &[u8], b: &[u8], id: &str) {
    let a = f64::from_le_bytes(a.try_into().unwrap());
    let b = f64::from_le_bytes(b.try_into().unwrap());
    if b.is_nan() {
        assert!(a.is_nan(), "{id}")
    } else if !b.is_finite() {
        assert_eq!(a, b, "{id}")
    } else if a == 0. && b == 0. {
        assert_eq!(a.to_bits(), b.to_bits(), "{id}: zero sign")
    } else {
        assert!(
            a.is_finite() && (a - b).abs() <= 1e-10 + 1e-10 * b.abs(),
            "{id}: {a} vs {b}"
        )
    }
}
fn compare(actual: &[u8], original: &[u8], id: &str) {
    assert_eq!(actual.len(), original.len(), "{id}: full shape");
    assert_eq!(actual[0], original[0], "{id}: success/refusal");
    if original[0] == 1 {
        assert_eq!(actual, original, "{id}: full FitError brand/name/message");
        return;
    }
    let prefix = 5 + u32::from_le_bytes(original[1..5].try_into().unwrap()) as usize;
    assert_eq!(&actual[..prefix], &original[..prefix], "{id}: kind");
    for i in 0..8 {
        let at = prefix + i * 8;
        scalar(&actual[at..at + 8], &original[at..at + 8], id)
    }
    let counts = prefix + 64;
    assert_eq!(
        &actual[counts..counts + 9],
        &original[counts..counts + 9],
        "{id}: used/region counts and held presence"
    );
    let mut at = counts + 9;
    if original[at - 1] == 1 {
        for _ in 0..2 {
            scalar(&actual[at..at + 8], &original[at..at + 8], id);
            at += 8
        }
    }
    assert_eq!(
        &actual[at..],
        &original[at..],
        "{id}: complete original region and identity"
    );
    assert_eq!(actual.last(), Some(&1));
}
#[test]
fn held_refusal_ties_original_region_and_guard_order() {
    let m = model("probe\t16\t1\t3\tnone\t24");
    let region: Arc<[u32]> = m.idx.clone().into();
    let settings = Settings {
        sigma: 3.,
        known_diameter: None,
    };
    let r = fit_sphere_on_selection(&m.p, region.clone(), settings, 100).unwrap();
    assert_eq!(r.radius, 10.);
    assert_eq!(r.used_points, 24);
    assert!(r.held.is_none());
    assert!(Arc::ptr_eq(&r.region, &region));
    let held = fit_sphere_on_selection(
        &m.p,
        region.clone(),
        Settings {
            known_diameter: Some(20.),
            ..settings
        },
        100,
    )
    .unwrap();
    assert_eq!(held.held.unwrap().free_diameter, 20.);
    let refusal = fit_sphere_on_selection(
        &m.p,
        region.clone(),
        Settings {
            known_diameter: Some(0.0625),
            ..settings
        },
        100,
    )
    .unwrap_err();
    assert!(matches!(refusal,Error::Source(s)if s.contains("known Ø 0.063 mm")));
    let tiny = model("probe\t17\t1\t3\t2\t24");
    let refusal = fit_sphere_on_selection(
        &tiny.p,
        tiny.idx.into(),
        Settings {
            known_diameter: Some(2.),
            ..settings
        },
        100,
    )
    .unwrap_err();
    assert!(matches!(refusal,Error::Source(s)if s.contains("sphere of Ø 0.063 mm")));
    let early = fit_sphere_on_selection(&[0.], vec![u32::MAX; 19].into(), settings, 0).unwrap_err();
    assert!(matches!(early,Error::Source(s)if s.ends_with("selection has 19.")));
    assert_eq!(
        fit_sphere_on_selection(&m.p, region, settings, 23).unwrap_err(),
        Error::InvalidInput
    );
}
#[test]
#[ignore = "complete original marked-sphere graph requires Node22/typescript"]
fn pinned_marked_sphere_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/marked-sphere.cjs"))
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
    assert_eq!(lines.len(), 2220);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/marked-sphere-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &sha.stdout).unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let mut successes = 0;
    for (descriptor, line) in INPUT.lines().zip(lines) {
        let p = line.split('\t').collect::<Vec<_>>();
        let m = model(descriptor);
        assert_eq!(p[0], m.id);
        let region: Arc<[u32]> = m.idx.clone().into();
        assert_eq!(region.as_ref(), m.idx);
        let input = packet(&m);
        assert_eq!(input, unhex(p[1]), "{}: independent actual inputs", m.id);
        assert_eq!(p[1], p[3], "{}: original mutation", m.id);
        let original = unhex(p[2]);
        let actual = output(
            fit_sphere_on_selection(
                &m.p,
                region.clone(),
                Settings {
                    sigma: m.k,
                    known_diameter: m.held,
                },
                10000,
            ),
            &region,
        );
        let after = packet(&Model {
            id: m.id.clone(),
            p: m.p.clone(),
            idx: region.to_vec(),
            k: m.k,
            held: m.held,
        });
        assert_eq!(input, after, "{}: native mutation", m.id);
        let case = dir.join(&m.id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("descriptor.tsv", descriptor.as_bytes()),
            ("original.bin", original.as_slice()),
            ("rust.bin", actual.as_slice()),
            ("original.after.bin", unhex(p[3]).as_slice()),
            ("rust.after.bin", after.as_slice()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap()
        }
        compare(&actual, &original, &m.id);
        successes += usize::from(original[0] == 0);
    }
    assert!(successes > 0);
    println!(
        "2220 complete marked-sphere values/held/used/region/identity/FitError/input comparisons; {successes} successful outputs; {}",
        dir.display()
    );
}

#[test]
fn clipping_keeps_the_entire_original_recipe_region() {
    let m = model("probe\t9\t1\t1\tnone\tdefault");
    let region: Arc<[u32]> = m.idx.clone().into();
    let fit = fit_sphere_on_selection(
        &m.p,
        region.clone(),
        Settings {
            sigma: 1.,
            known_diameter: None,
        },
        10000,
    )
    .unwrap();
    assert!(fit.used_points < fit.region_size);
    assert_eq!(fit.region_size, 128);
    assert!(Arc::ptr_eq(&fit.region, &region));
    assert_eq!(fit.region.as_ref(), m.idx);
}
