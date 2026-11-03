// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original primitive category/string/input comparisons, no tolerance.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::js_fixed::fixed3;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/fixed3/cases.tsv");
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|h| u8::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect()
}
fn category(x: f64) -> &'static str {
    if x.is_nan() {
        "nan"
    } else if x == f64::INFINITY {
        "inf"
    } else if x == f64::NEG_INFINITY {
        "-inf"
    } else if x == 0. && x.is_sign_negative() {
        "-0"
    } else {
        "finite"
    }
}
#[test]
fn exact_ties_negative_zero_threshold_and_bounded_output() {
    for (n, s) in [
        (0.0625, "0.063"),
        (-0.0625, "-0.063"),
        (0.1875, "0.188"),
        (-0., "0.000"),
        (-f64::from_bits(1), "-0.000"),
        (1e21, "1e+21"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(fixed3(n), s)
    }
    for line in INPUT.lines() {
        let bits = u64::from_str_radix(line.split('\t').nth(1).unwrap(), 16).unwrap();
        assert!(fixed3(f64::from_bits(bits)).len() <= 26)
    }
}
#[test]
#[ignore = "original Number intrinsic and pinned application call sites require Node22"]
fn pinned_fixed3_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/fixed3.cjs"))
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
    assert_eq!(lines.len(), 8492);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/fixed-number-diff/run-{run}"));
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
    for (descriptor, line) in INPUT.lines().zip(lines) {
        let p = line.split('\t').collect::<Vec<_>>();
        let d = descriptor.split('\t').collect::<Vec<_>>();
        assert_eq!(p[0], d[0]);
        let bits = u64::from_str_radix(d[1], 16).unwrap();
        let value = f64::from_bits(bits);
        let input = bits.to_be_bytes();
        assert_eq!(input.as_slice(), unhex(p[1]));
        assert_eq!(p[1], p[4], "original input mutation");
        assert_eq!(category(value), p[2]);
        let actual = fixed3(value);
        let original = unhex(p[3]);
        let after = value.to_bits().to_be_bytes();
        assert_eq!(input, after);
        let case = dir.join(d[0]);
        std::fs::create_dir_all(&case).unwrap();
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("descriptor.tsv", descriptor.as_bytes()),
            ("original.bin", original.as_slice()),
            ("rust.bin", actual.as_bytes()),
            ("original.category.txt", p[2].as_bytes()),
            ("rust.category.txt", category(value).as_bytes()),
            ("original.after.bin", unhex(p[4]).as_slice()),
            ("rust.after.bin", after.as_slice()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap()
        }
        assert_eq!(
            actual.as_bytes(),
            original,
            "{} exact fixed3 for bits {}",
            d[0],
            d[1]
        );
    }
    println!(
        "8492 exact fixed3 primitive/category/string/immutability observations; {}",
        dir.display()
    );
}
