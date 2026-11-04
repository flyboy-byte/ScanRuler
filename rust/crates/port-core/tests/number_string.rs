// SPDX-License-Identifier: AGPL-3.0-only
//! Primitive String/JSON bytes from the actual pinned numeric error branch.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/number-string/cases.tsv");
fn observe(line: &str) -> String {
    let mut cols = line.split('\t');
    let id = cols.next().unwrap();
    let bits = cols.next().unwrap();
    let n = f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
    let category = if n.is_nan() {
        "nan"
    } else if n == f64::INFINITY {
        "inf"
    } else if n == f64::NEG_INFINITY {
        "-inf"
    } else if n == 0. && n.is_sign_negative() {
        "-0"
    } else {
        "finite"
    };
    let mut value = port_core::js_number::number_string(n);
    if std::env::var_os("SCANRULER_NUMBER_STRING_SABOTAGE").is_some() {
        value.push('9')
    }
    let json = if n.is_finite() {
        value.as_str()
    } else {
        "null"
    };
    format!("{id}\t{bits}\t{category}\t{value}\t{json}\n")
}
#[test]
fn exact_decimal_midpoints_choose_the_original_even_digit() {
    for (value, expected) in [
        (f64::from_bits(0x4061_a492_4000_0000), "141.14285278320312"),
        (f64::from_bits(0xc061_a492_4000_0000), "-141.14285278320312"),
        (f64::from_bits(0xc089_0e09_2000_0000), "-801.7544555664062"),
    ] {
        assert_eq!(port_core::js_number::number_string(value), expected);
    }
}
#[test]
fn native_number_profile_is_bounded() {
    assert_eq!(INPUT.lines().count(), 6241);
    for line in INPUT.lines() {
        assert!(observe(line).len() < 160)
    }
}
#[test]
#[ignore = "complete original numeric String/JSON reference needs Node22"]
fn pinned_number_string_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/number-string-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), &sha.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(dirty.status.success());
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let built = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-number-string.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("oracle-build.stdout.txt"), &built.stdout).unwrap();
    std::fs::write(dir.join("oracle-build.stderr.txt"), &built.stderr).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/number-string.mjs"))
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
    assert!(source.stderr.is_empty());
    let output = String::from_utf8(source.stdout).unwrap();
    let results = output.split_inclusive('\n').collect::<Vec<_>>();
    assert_eq!(results.len(), 6241);
    let mut mismatches = 0;
    for (line, original) in INPUT.lines().zip(results) {
        let id = line.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("input.bin"), line.as_bytes()).unwrap();
        std::fs::write(case.join("original.bin"), original.as_bytes()).unwrap();
        std::fs::write(case.join("original.stderr.txt"), &source.stderr).unwrap();
        let actual = observe(line);
        std::fs::write(case.join("rust.bin"), actual.as_bytes()).unwrap();
        if actual != original {
            mismatches += 1;
            if mismatches <= 5 {
                eprintln!("{id}: original {original:?}; rust {actual:?}")
            }
        }
        let mut wrong = actual.clone().into_bytes();
        *wrong.last_mut().unwrap() ^= 1;
        assert_ne!(wrong, original.as_bytes());
        assert_ne!(&actual.as_bytes()[..actual.len() - 1], original.as_bytes());
    }
    println!(
        "6241 primitive Number text comparisons; mismatches={mismatches}; raw {}",
        dir.display()
    );
    assert_eq!(mismatches, 0);
}
