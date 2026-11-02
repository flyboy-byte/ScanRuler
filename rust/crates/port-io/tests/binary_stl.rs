// SPDX-License-Identifier: AGPL-3.0-only
//! Named deterministic cases shared independently with the original helper.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use port_io::stl::{BinaryStlError, read_binary_triangles};

fn fixture(count: usize, seed: u32) -> Vec<u8> {
    let mut bytes = vec![0; 84 + count * 50];
    bytes[..5].copy_from_slice(b"solid");
    // Deliberately unrelated to the explicit count: the helper does not read it.
    bytes[80..84].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut state = seed;
    for record in bytes[84..].as_chunks_mut::<50>().0.iter_mut() {
        record[..12].fill(0xff);
        record[48..].fill(0xff);
        for coordinate in record[12..48].as_chunks_mut::<4>().0.iter_mut() {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            // Finite binary32 values, varied sign/magnitude/mantissa.
            let bits = (state & 0x807f_ffff) | (((state >> 23) % 250 + 1) << 23);
            coordinate.copy_from_slice(&bits.to_le_bytes());
        }
    }
    bytes
}

#[test]
fn bounds_and_budget_fail_before_progress() {
    for (bytes, count, limit, expected) in [
        (vec![0; 131], 1, 1, BinaryStlError::Truncated),
        (vec![0; 84], 1, 0, BinaryStlError::TriangleLimit),
        (
            vec![0; 84],
            usize::MAX,
            usize::MAX,
            BinaryStlError::Truncated,
        ),
    ] {
        let mut called = false;
        assert_eq!(
            read_binary_triangles(&bytes, count, limit, |_| called = true),
            Err(expected)
        );
        assert!(!called);
    }
    assert_eq!(read_binary_triangles(&[0; 84], 0, 0, |_| {}), Ok(vec![]));
}

#[test]
fn coordinates_order_and_trailing_bytes() {
    let mut bytes = fixture(2, 7);
    bytes.extend_from_slice(b"trailing bytes");
    let positions = read_binary_triangles(&bytes, 2, 2, |_| {}).unwrap();
    let expected: Vec<u32> = bytes[84..184]
        .as_chunks::<50>()
        .0
        .iter()
        .flat_map(|record| {
            record[12..48]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u32::from_le_bytes(*b))
        })
        .collect();
    assert_eq!(
        positions.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        expected
    );
}

// Opt-in live source comparison. The ordinary tests do not require Node or Git.
// Run: cargo test -p port-io --test binary_stl pinned_original_comparison -- --ignored --nocapture
#[test]
#[ignore = "live oracle comparison requires Node and the pinned Git object"]
fn pinned_original_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let reports = root
        .join("rust/target/stl-diff")
        .join(format!("run-{run_id}"));
    println!("raw comparison evidence: {}", reports.display());
    std::fs::create_dir_all(&reports).unwrap();
    let candidate = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(candidate.status.success());
    std::fs::write(reports.join("identity.txt"), format!(
        "contract: io.stl.binary.bits.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}\nfixture-generator: binary_stl.rs fixture LCG 1664525/1013904223, seeds 0,1,7,99,42,43,17,5\ncomparison: exact LE coordinate bytes and UTF-8 progress, no tolerance\n",
        String::from_utf8_lossy(&candidate.stdout).trim()
    )).unwrap();
    let mut cases = vec![("empty", fixture(0, 0), 0)];
    for (name, count, seed) in [
        ("one", 1, 1),
        ("seed_7", 41, 7),
        ("seed_99", 257, 99),
        ("progress", 262146, 42),
        ("progress_twice", 524290, 43),
    ] {
        cases.push((name, fixture(count, seed), count));
    }
    let mut special = fixture(1, 0);
    for (slot, bits) in [
        0u32, 0x80000000, 0x7f800000, 0xff800000, 1, 0x007fffff, 0x7fc12345, 0x7f812345, 0xff812345,
    ]
    .into_iter()
    .enumerate()
    {
        special[96 + slot * 4..100 + slot * 4].copy_from_slice(&bits.to_le_bytes());
    }
    cases.push(("ieee754", special, 1));
    let mut extra = fixture(2, 17);
    extra.extend_from_slice(b"ignored trailer");
    cases.push(("trailer", extra, 2));
    let mut no_attributes = fixture(1, 5);
    no_attributes.truncate(132);
    cases.push(("missing_final_attributes", no_attributes, 1));
    cases.push(("empty_no_header", vec![], 0));
    cases.push(("truncated", vec![0; 131], 1));
    let mut successful = 0;
    for (id, input, count) in cases {
        let mut progress = String::new();
        let candidate = read_binary_triangles(&input, count, count, |text| {
            progress.push_str(text);
            progress.push('\n');
        });
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/binary-stl.cjs"))
            .arg(count.to_string())
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node is required for the requested live oracle comparison");
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let original = child.wait_with_output().unwrap();
        let case_dir = reports.join(id);
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("input.bin"), &input).unwrap();
        std::fs::write(case_dir.join("original.bin"), &original.stdout).unwrap();
        std::fs::write(
            case_dir.join("original.progress-or-error.txt"),
            &original.stderr,
        )
        .unwrap();
        std::fs::write(case_dir.join("rust.progress.txt"), &progress).unwrap();
        std::fs::write(case_dir.join("count.txt"), count.to_string()).unwrap();
        match candidate {
            Ok(positions) => {
                assert!(
                    original.status.success(),
                    "{id}: oracle failed: {}",
                    String::from_utf8_lossy(&original.stderr)
                );
                let raw: Vec<u8> = positions
                    .iter()
                    .flat_map(|v| v.to_bits().to_le_bytes())
                    .collect();
                std::fs::write(case_dir.join("rust.bin"), &raw).unwrap();
                assert!(raw == original.stdout, "{id}: coordinate bytes differ");
                assert_eq!(
                    progress.as_bytes(),
                    original.stderr,
                    "{id}: progress differs"
                );
                successful += 1;
                println!(
                    "{id}: {} coordinate bytes + progress compared exactly",
                    raw.len()
                );
            }
            Err(BinaryStlError::Truncated) => {
                assert_eq!(original.status.code(), Some(2), "{id}");
                assert_eq!(original.stderr, b"RangeError\n", "{id}");
                assert!(original.stdout.is_empty(), "{id}");
                println!(
                    "{id}: Rust truncation / original RangeError (not successful value parity)"
                );
            }
            Err(error) => panic!("{id}: unexpected {error:?}"),
        }
    }
    assert_eq!(successful, 10, "missing successful comparison cases");
}
