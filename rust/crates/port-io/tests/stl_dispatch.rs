// SPDX-License-Identifier: AGPL-3.0-only
//! Whole-parser cases: binary precedence, fallback, text decoding and numeric grammar.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use port_io::stl_dispatch::{StlError, StlLimits, read_stl};

fn limits() -> StlLimits {
    StlLimits {
        max_bytes: 32 * 1024 * 1024,
        max_triangles: 1_000_000,
    }
}

fn binary(count: usize, solid: bool, header_count: u32) -> Vec<u8> {
    let mut bytes = vec![0; 84 + count * 50];
    if solid {
        bytes[..5].copy_from_slice(b"solid");
    }
    bytes[80..84].copy_from_slice(&header_count.to_le_bytes());
    for (t, record) in bytes[84..].as_chunks_mut::<50>().0.iter_mut().enumerate() {
        record[..12].fill(0xff);
        record[48..].fill(0xff);
        for (i, coordinate) in record[12..48].as_chunks_mut::<4>().0.iter_mut().enumerate() {
            coordinate.copy_from_slice(&((t * 9 + i) as f32 - 2.5).to_le_bytes());
        }
    }
    bytes
}

fn ascii(coords: &str) -> Vec<u8> {
    format!("solid sample\nvertex {coords}\nvertex {coords}\nvertex {coords}\nendsolid")
        .into_bytes()
}

struct Case {
    id: String,
    input: Vec<u8>,
    success: bool,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    let mut add = |id: &str, input: Vec<u8>, success| {
        out.push(Case {
            id: id.into(),
            input,
            success,
        })
    };
    add("empty", vec![], false);
    add("short_junk", b"not an STL".to_vec(), false);
    add("empty_ascii", b"solid\nendsolid".to_vec(), false);
    add("one_vertex", b"solid vertex 1 2 3".to_vec(), false);
    add(
        "two_vertices",
        b"solid vertex 1 2 3 vertex 4 5 6".to_vec(),
        false,
    );
    add("ascii_triangle", ascii("0 1 -2"), true);
    add("binary_solid_precedence", binary(2, true, 2), true);
    add("binary_plain", binary(3, false, 3), true);
    add("binary_zero_solid", binary(0, true, 0), true);
    add("binary_count_mismatch", binary(2, false, u32::MAX), true);
    add("binary_solid_count_mismatch", binary(2, true, 3), false);
    let mut trailer = binary(1, false, 10);
    trailer.extend_from_slice(b"ignored tail");
    add("binary_partial_trailer", trailer, true);
    let mut missing_attributes = binary(1, false, 1);
    missing_attributes.truncate(132);
    add(
        "binary_short_record_falls_back_to_zero",
        missing_attributes,
        true,
    );
    add("binary_minimum_mismatch", binary(0, false, u32::MAX), true);
    add(
        "uppercase_solid",
        b"SOLID vertex 1 2 3 vertex 1 2 3 vertex 1 2 3".to_vec(),
        false,
    );
    add(
        "uppercase_vertex",
        b"solid VERTEX 1 2 3 VERTEX 1 2 3 VERTEX 1 2 3".to_vec(),
        false,
    );
    add(
        "solid_suffix",
        b"solidfoo vertex 1 2 3 vertex 1 2 3 vertex 1 2 3".to_vec(),
        true,
    );
    add(
        "substring_keyword",
        b"solid notvertex 1 2 3 vertexvertex 4 5 6 vertex 7 8 9".to_vec(),
        true,
    );
    add(
        "failed_keyword",
        b"solid vertex1 2 3 vertex 0 0 0 vertex 1 0 0 vertex 0 1 0".to_vec(),
        true,
    );
    add(
        "token_consumption",
        b"solid vertex xvertex 1 2 3 vertex 4 5 6 vertex 7 8 9".to_vec(),
        true,
    );
    add(
        "trailing_incomplete_match",
        [ascii("1 2 3"), b" vertex 1 2".to_vec()].concat(),
        true,
    );
    add(
        "second_complete_triangle",
        [
            ascii("1 2 3"),
            b" vertex 4 5 6 vertex 7 8 9 vertex 10 11 12".to_vec(),
        ]
        .concat(),
        true,
    );
    for (id, coords) in [
        ("decimal_prefix", "12junk -3.25mm .5suffix"),
        ("unfinished_exponent", "1e+ 2E- 3.e"),
        ("exponent", "+1.e2 -.25E+2 6.022e23"),
        ("nondecimal_prefix", "0x10 -0x20 0b101"),
        ("infinity", "Infinityjunk -Infinity +Infinity"),
        ("nan", "NaN -NaN +junk"),
        ("invalid_numeric", ". +.e2 --1"),
        ("zero_signs", "-0 -0.0e5 +0"),
        ("overflow_underflow", "1e400 -1e-400 -1e400"),
        (
            "f32_rounding",
            "1.0000000596046448 1.0000001788139343 16777217",
        ),
        (
            "f64_rounding",
            "9007199254740993 4.9406564584124654e-324 -2.4703282292062327e-324",
        ),
        ("nul_suffix", "1\0junk 2\0 3"),
        ("unicode_suffix", "1é 2٣ 3∞"),
    ] {
        add(id, ascii(coords), true);
    }
    add(
        "huge_exponent",
        ascii(&format!("1e{} -1e-{} .5", "9".repeat(400), "9".repeat(400))),
        true,
    );
    add(
        "long_decimal",
        ascii(&format!(
            "{}.0 -0.{}1 +1",
            "9".repeat(1000),
            "0".repeat(1000)
        )),
        true,
    );
    for (n, c) in [
        '\t', '\n', '\u{000b}', '\u{000c}', '\r', ' ', '\u{00a0}', '\u{1680}', '\u{2000}',
        '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}',
        '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}',
        '\u{3000}', '\u{feff}',
    ]
    .into_iter()
    .enumerate()
    {
        add(
            &format!("js_space_{n:02}"),
            format!("{c}solid{c}vertex{c}1{c}2{c}3{c}vertex{c}4{c}5{c}6{c}vertex{c}7{c}8{c}9")
                .into_bytes(),
            true,
        );
    }
    for (n, c) in ['\u{0085}', '\u{180e}', '\u{200b}', '\u{001c}']
        .into_iter()
        .enumerate()
    {
        add(
            &format!("not_js_space_{n}"),
            format!("solid vertex{c}1{c}2{c}3 vertex{c}1{c}2{c}3 vertex{c}1{c}2{c}3").into_bytes(),
            false,
        );
    }
    add(
        "bom",
        [vec![0xef, 0xbb, 0xbf], ascii("1 2 3")].concat(),
        true,
    );
    for (n, bad) in [
        vec![0xff],
        vec![0xc0, 0xaf],
        vec![0xe0, 0x80, 0x80],
        vec![0xed, 0xa0, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
        vec![0xe2, 0x82],
    ]
    .into_iter()
    .enumerate()
    {
        let mut text = b"solid vertex 1".to_vec();
        text.extend_from_slice(&bad);
        text.extend_from_slice(b" 2 3 vertex 4 5 6 vertex 7 8 9");
        add(&format!("utf8_replacement_{n}"), text, true);
    }
    add(
        "utf8_incomplete_tail",
        [ascii("1 2 3"), vec![0xf0, 0x9f, 0x92]].concat(),
        true,
    );
    add(
        "utf8_invalid_prefix",
        [vec![0xff], ascii("1 2 3")].concat(),
        false,
    );
    for n in [507, 508, 512] {
        let mut text = vec![b' '; n];
        text.extend_from_slice(&ascii("1 2 3"));
        add(&format!("head_boundary_{n}"), text, true);
    }
    for seed in [1u32, 7, 99, 1039] {
        let mut text = String::from("solid seeded\n");
        let mut state = seed;
        for _ in 0..57 {
            text.push_str("vertex");
            for _ in 0..3 {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                text.push_str(&format!(" {:.17e}mm", (state as i32) as f64 / 7919.0));
            }
            text.push('\n');
        }
        add(&format!("ascii_seed_{seed}"), text.into_bytes(), true);
    }
    out
}

#[test]
fn source_contract_and_resource_guards() {
    for case in cases() {
        assert_eq!(
            read_stl(&case.input, limits(), |_| {}).is_ok(),
            case.success,
            "{}",
            case.id
        );
    }
    let mut called = false;
    assert_eq!(
        read_stl(
            &ascii("1 2 3"),
            StlLimits {
                max_bytes: 1,
                max_triangles: 1
            },
            |_| called = true
        ),
        Err(StlError::InputLimit)
    );
    assert!(!called);
    assert_eq!(
        read_stl(
            &ascii("1 2 3"),
            StlLimits {
                max_bytes: 1024,
                max_triangles: 0
            },
            |_| called = true
        ),
        Err(StlError::TriangleLimit)
    );
    assert!(called); // original ASCII progress occurs before extraction
    called = false;
    assert_eq!(
        read_stl(
            &binary(1, true, 1),
            StlLimits {
                max_bytes: 1024,
                max_triangles: 0
            },
            |_| called = true
        ),
        Err(StlError::TriangleLimit)
    );
    assert!(!called);
}

#[test]
fn numeric_prefixes_and_triangle_order() {
    let values = read_stl(&ascii("1e+ -0x1 +.5junk"), limits(), |_| {}).unwrap();
    assert_eq!(values.len(), 9);
    for xyz in values.as_chunks::<3>().0 {
        assert_eq!(
            xyz.map(f32::to_bits),
            [1.0f32.to_bits(), (-0.0f32).to_bits(), 0.5f32.to_bits()]
        );
    }
}

#[test]
#[ignore = "live full-parser oracle comparison requires Node and the pinned Git object"]
fn pinned_stl_dispatch_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let run_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let reports = root
        .join("rust/target/stl-dispatch-diff")
        .join(format!("run-{run_id}"));
    std::fs::create_dir_all(&reports).unwrap();
    println!("raw full-parser evidence: {}", reports.display());
    let mut identity = String::from(
        "contract: io.stl.dispatch.bits.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: exact little-endian float32 bytes, progress and source error messages\nseeds: 1,7,99,1039\n",
    );
    for (name, program, args) in [
        ("candidate", "git", vec!["rev-parse", "HEAD"]),
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["-V"]),
    ] {
        let output = Command::new(program)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(output.status.success());
        identity.push_str(&format!(
            "{name}: {}\n",
            String::from_utf8_lossy(&output.stdout).trim()
        ));
    }
    std::fs::write(reports.join("identity.txt"), identity).unwrap();
    let cases = cases();
    let expected_count = cases.len();
    assert_eq!(
        expected_count, 82,
        "case schedule changed; review input manifest"
    );
    let mut passed = 0;
    let mut refused = 0;
    for case in cases {
        let mut progress = String::new();
        let candidate = read_stl(&case.input, limits(), |text| {
            progress.push_str(text);
            progress.push('\n');
        });
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/stl-dispatch.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node required for explicit live comparison");
        child.stdin.take().unwrap().write_all(&case.input).unwrap();
        let original = child.wait_with_output().unwrap();
        let dir = reports.join(&case.id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("input.bin"), &case.input).unwrap();
        std::fs::write(dir.join("original.bin"), &original.stdout).unwrap();
        std::fs::write(dir.join("original.progress-or-error.txt"), &original.stderr).unwrap();
        assert_eq!(candidate.is_ok(), case.success, "{}: Rust result", case.id);
        assert_eq!(
            original.status.success(),
            case.success,
            "{}: original status: {}",
            case.id,
            String::from_utf8_lossy(&original.stderr)
        );
        match candidate {
            Ok(positions) => {
                let raw: Vec<u8> = positions
                    .iter()
                    .flat_map(|v| v.to_bits().to_le_bytes())
                    .collect();
                std::fs::write(dir.join("rust.bin"), &raw).unwrap();
                assert!(
                    raw == original.stdout,
                    "{}: coordinate bytes differ",
                    case.id
                );
                passed += 1;
                println!(
                    "{}: {} coordinate bytes compared exactly",
                    case.id,
                    raw.len()
                );
            }
            Err(error @ (StlError::InvalidFile | StlError::InvalidAscii)) => {
                assert_eq!(original.status.code(), Some(2), "{}", case.id);
                assert!(original.stdout.is_empty(), "{}", case.id);
                progress.push_str(&format!("{error}\n"));
                refused += 1;
                println!("{}: source refusal compared", case.id);
            }
            Err(error) => panic!("{}: unexpected host error {error}", case.id),
        }
        std::fs::write(dir.join("rust.progress-or-error.txt"), &progress).unwrap();
        assert_eq!(
            progress.as_bytes(),
            original.stderr,
            "{}: progress or error differs",
            case.id
        );
    }
    assert_eq!(passed + refused, expected_count);
    assert!(passed > 0 && refused > 0);
    println!("full-parser result: {passed} successful value cases, {refused} refusals");
}
