// SPDX-License-Identifier: AGPL-3.0-only
//! PNG decoding against actual Chrome outputs of the original's `decodeImage`.
//!
//! Offline: the committed batches in `docs/port-plan/fixtures/png-chrome` and the 2026-11-02 image
//! audit capture, both Chrome 156 results. Live (ignored): fresh seeded batches from the same
//! generator run through Chrome now (`docs/port-plan/oracle/image-decode.mjs`).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use port_io::inflate::inflate_into;
use port_io::png::{PngError, PngLimits, canvas_round_trip, decode_png};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// One result: `None` when the original leaves the image out.
type Outcome = Option<(u32, u32, Vec<u8>)>;

fn rust_outcome(bytes: &[u8]) -> Result<Outcome, PngError> {
    match decode_png(bytes, PngLimits::default()) {
        Ok(img) => Ok(Some((img.width, img.height, img.rgba))),
        Err(PngError::Omitted) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Tally of one comparison. A host limit counts as its own class, and only where Chrome also
/// produced nothing; anywhere else it is a mismatch.
#[derive(Default, Debug)]
struct Tally {
    decoded: usize,
    omitted: usize,
    host_limit: usize,
    mismatches: Vec<usize>,
}

fn compare(cases: &[Vec<u8>], chrome: &[Outcome], sabotage: bool) -> Tally {
    assert_eq!(
        cases.len(),
        chrome.len(),
        "batch and results differ in length"
    );
    let mut t = Tally::default();
    for (i, (bytes, expected)) in cases.iter().zip(chrome).enumerate() {
        match rust_outcome(bytes) {
            Ok(mut got) => {
                if sabotage && let Some((_, _, px)) = got.as_mut() {
                    px[0] ^= 1;
                }
                if &got != expected {
                    t.mismatches.push(i);
                } else if got.is_some() {
                    t.decoded += 1;
                } else {
                    t.omitted += 1;
                }
            }
            Err(PngError::Limit) if expected.is_none() => t.host_limit += 1,
            Err(_) => t.mismatches.push(i),
        }
    }
    t
}

fn u32_at(b: &[u8], at: &mut usize) -> u32 {
    let v = b
        .get(*at..*at + 4)
        .map_or(0, |s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]));
    *at += 4;
    v
}

fn read_batch(b: &[u8]) -> Vec<Vec<u8>> {
    let mut at = 0;
    let n = u32_at(b, &mut at);
    (0..n)
        .map(|_| {
            let len = u32_at(b, &mut at) as usize;
            at += len;
            b[at - len..at].to_vec()
        })
        .collect()
}

fn read_results(b: &[u8]) -> Vec<Outcome> {
    let mut at = 0;
    let n = u32_at(b, &mut at);
    (0..n)
        .map(|_| {
            let s = b[at];
            at += 1;
            if s != 1 {
                return None;
            }
            let w = u32_at(b, &mut at);
            let h = u32_at(b, &mut at);
            let len = u32_at(b, &mut at) as usize;
            at += len;
            Some((w, h, b[at - len..at].to_vec()))
        })
        .collect()
}

/// A gzip member written by `capture.py` (no optional header fields).
fn gunzip(gz: &[u8]) -> Vec<u8> {
    assert!(
        gz.len() > 18 && gz[0] == 0x1f && gz[1] == 0x8b && gz[3] == 0,
        "plain gzip expected"
    );
    let n = gz.len();
    let size = u32::from_le_bytes([gz[n - 4], gz[n - 3], gz[n - 2], gz[n - 1]]) as usize;
    inflate_into(&gz[10..n - 8], size).unwrap_or_default()
}

fn fixture(name: &str) -> Vec<u8> {
    gunzip(
        &std::fs::read(root().join("docs/port-plan/fixtures/png-chrome").join(name))
            .unwrap_or_default(),
    )
}

#[test]
fn committed_chrome_batches_match() {
    let mut report = Vec::new();
    for set in ["named", "sweep-s1", "struct-s11"] {
        let cases = read_batch(&fixture(&format!("{set}.bin.gz")));
        let chrome = read_results(&fixture(&format!("{set}.chrome.bin.gz")));
        assert!(!cases.is_empty(), "{set}: no cases");
        let t = compare(&cases, &chrome, false);
        report.push(format!(
            "{set}: {} decoded, {} omitted, {} host limit, {} mismatches {:?}",
            t.decoded,
            t.omitted,
            t.host_limit,
            t.mismatches.len(),
            &t.mismatches[..t.mismatches.len().min(10)]
        ));
        assert!(t.mismatches.is_empty(), "{}", report.join("\n"));
    }
    println!("{}", report.join("\n"));
}

#[test]
fn comparison_rejects_a_wrong_pixel() {
    let cases = read_batch(&fixture("named.bin.gz"));
    let chrome = read_results(&fixture("named.chrome.bin.gz"));
    let t = compare(&cases, &chrome, true);
    assert!(
        t.mismatches.len() > 100,
        "sabotage caught only {} cases",
        t.mismatches.len()
    );
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(0))
        .collect()
}

/// The committed source-only Chrome 156 capture of the image audit (no Rust output in it).
fn audit_capture() -> Vec<(String, Outcome)> {
    let text = std::fs::read_to_string(
        root().join("docs/port-plan/evidence/2026-11-02-image-source.json"),
    )
    .unwrap_or_default();
    let mut out = Vec::new();
    for part in text.split("{\n      \"id\": \"").skip(1) {
        let id = part[..part.find('"').unwrap_or(0)].to_string();
        let field = |name: &str| -> Option<&str> {
            let at = part.find(&format!("\"{name}\": "))? + name.len() + 4;
            let rest = &part[at..];
            Some(rest[..rest.find([',', '\n']).unwrap_or(rest.len())].trim_matches('"'))
        };
        let outcome = if part.contains("\"source\": null") {
            None
        } else {
            let w = field("width").and_then(|v| v.parse().ok()).unwrap_or(0);
            let h = field("height").and_then(|v| v.parse().ok()).unwrap_or(0);
            Some((w, h, hex(field("rgba_hex").unwrap_or(""))))
        };
        out.push((id, outcome));
    }
    out
}

#[test]
fn image_audit_png_fixtures_match_chrome() {
    let capture = audit_capture();
    let pngs: Vec<_> = capture
        .iter()
        .filter(|(id, _)| id.contains("png"))
        .collect();
    assert!(pngs.len() >= 13, "capture has {} PNG cases", pngs.len());
    let mut bad = Vec::new();
    for (id, expected) in pngs {
        let bytes =
            std::fs::read(root().join(format!("docs/port-plan/fixtures/image-audit/{id}.bin")))
                .unwrap_or_default();
        if rust_outcome(&bytes).as_ref() != Ok(expected) {
            bad.push(id.clone());
        }
    }
    assert!(bad.is_empty(), "mismatched against Chrome: {bad:?}");
}

#[test]
fn canvas_round_trip_matches_the_alpha_atlas() {
    // The atlas fixture holds all 65,536 colour/alpha pairs and decodes to Chrome's bytes in the
    // audit test above; these two pairs are read from Chrome's atlas output.
    let mut px = [219, 31, 0, 6];
    canvas_round_trip(&mut px);
    assert_eq!(px, [212, 42, 0, 6]);
    let mut opaque = [1, 2, 3, 255];
    canvas_round_trip(&mut opaque);
    assert_eq!(opaque, [1, 2, 3, 255]);
    let mut clear = [9, 9, 9, 0];
    canvas_round_trip(&mut clear);
    assert_eq!(clear, [0, 0, 0, 0]);
}

#[test]
fn host_limits_are_not_omissions() {
    let bytes = std::fs::read(root().join("docs/port-plan/fixtures/image-audit/png-rgba.bin"))
        .unwrap_or_default();
    assert_eq!(
        decode_png(
            &bytes,
            PngLimits {
                max_input: 1 << 20,
                max_pixels: 10
            }
        ),
        Err(PngError::Limit)
    );
    assert_eq!(
        decode_png(
            &bytes,
            PngLimits {
                max_input: 10,
                max_pixels: 1 << 20
            }
        ),
        Err(PngError::Limit)
    );
    assert!(decode_png(&bytes, PngLimits::default()).is_ok());
}

fn run(cmd: &mut Command, input: &[u8]) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut child = cmd
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start the process");
    let mut sin = child.stdin.take().expect("stdin");
    let input = input.to_vec();
    // Write from another thread so a large batch cannot deadlock against the output pipe.
    let writer = std::thread::spawn(move || {
        let _ = sin.write_all(&input);
    });
    let mut out = Vec::new();
    child
        .stdout
        .take()
        .expect("stdout")
        .read_to_end(&mut out)
        .expect("read output");
    writer.join().expect("writer");
    assert!(child.wait().expect("wait").success(), "{cmd:?} failed");
    out
}

/// Fresh seeded batches through actual Chrome. `SCANRULER_PNG_SEEDS` (default `2,3,4`),
/// `SCANRULER_PNG_FLIPS` per sweep base (default 1500), `SCANRULER_PNG_IMAGES` per structural
/// batch (default 2000), `SCANRULER_PNG_SABOTAGE=1` to prove a wrong pixel is caught. Each
/// mismatching case is written to the system temp directory.
#[test]
#[ignore = "runs Chrome and Node 22 (fnm)"]
fn pinned_png_comparison() {
    let seeds = std::env::var("SCANRULER_PNG_SEEDS").unwrap_or_else(|_| "2,3,4".into());
    let flips = std::env::var("SCANRULER_PNG_FLIPS").unwrap_or_else(|_| "1500".into());
    let images = std::env::var("SCANRULER_PNG_IMAGES").unwrap_or_else(|_| "2000".into());
    let sabotage = std::env::var("SCANRULER_PNG_SABOTAGE").is_ok_and(|v| v == "1");
    let tmp = std::env::temp_dir().join("scanruler-png");
    let _ = std::fs::create_dir_all(&tmp);
    let generator = root().join("docs/port-plan/fixtures/png-chrome/generate.py");
    let mut total = Tally::default();
    for seed in seeds.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        for (kind, n) in [("sweep", &flips), ("struct", &images)] {
            let path = tmp.join(format!("{kind}-{seed}.bin"));
            run(
                Command::new("python3")
                    .arg(&generator)
                    .arg(kind)
                    .arg(&path)
                    .arg(seed)
                    .arg(n),
                &[],
            );
            let batch = std::fs::read(&path).unwrap_or_default();
            let chrome = run(
                Command::new("fnm").args([
                    "exec",
                    "--using=22",
                    "--",
                    "node",
                    "docs/port-plan/oracle/image-decode.mjs",
                ]),
                &batch,
            );
            let cases = read_batch(&batch);
            let t = compare(&cases, &read_results(&chrome), sabotage);
            println!(
                "{kind} seed {seed}: {} cases, {} decoded, {} omitted, {} host limit, {} mismatches",
                cases.len(),
                t.decoded,
                t.omitted,
                t.host_limit,
                t.mismatches.len()
            );
            for &i in &t.mismatches {
                let _ = std::fs::write(
                    tmp.join(format!("mismatch-{kind}-{seed}-{i}.png")),
                    &cases[i],
                );
            }
            total.decoded += t.decoded;
            total.omitted += t.omitted;
            total.host_limit += t.host_limit;
            total.mismatches.extend(t.mismatches);
        }
    }
    println!(
        "total: {} decoded, {} omitted, {} host limit, {} mismatches",
        total.decoded,
        total.omitted,
        total.host_limit,
        total.mismatches.len()
    );
    if sabotage {
        assert!(!total.mismatches.is_empty(), "sabotage was not detected");
    } else {
        assert!(
            total.mismatches.is_empty(),
            "reproducers in {}",
            tmp.display()
        );
    }
}

/// Compare one batch (u32 count, u32 length + bytes each) with the oracle's results for it, from
/// `SCANRULER_PNG_BATCH` and `SCANRULER_PNG_RESULTS`; for reproducers and new probe sets.
#[test]
#[ignore = "needs a batch and its Chrome results"]
fn png_batch_file() {
    let batch =
        std::fs::read(std::env::var("SCANRULER_PNG_BATCH").unwrap_or_default()).unwrap_or_default();
    let results = std::fs::read(std::env::var("SCANRULER_PNG_RESULTS").unwrap_or_default())
        .unwrap_or_default();
    let t = compare(&read_batch(&batch), &read_results(&results), false);
    println!(
        "{} decoded, {} omitted, {} host limit, mismatches {:?}",
        t.decoded, t.omitted, t.host_limit, t.mismatches
    );
    assert!(t.mismatches.is_empty());
}
