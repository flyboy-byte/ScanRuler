// SPDX-License-Identifier: AGPL-3.0-only
//! Own DEFLATE/ZIP reader against the original's `fflate` (`io.inflate`).
//!
//! Ordinary tests pin a few hand-built archives. The ignored comparison generates archives with
//! fflate itself, mutates them (bytes, and zip header fields set to edge values) and compares
//! `unzipSync` (as the 3MF reader calls it) with `unzip_filtered`: entries, names, bytes, and error
//! code and text. Host-limit refusals and allocation failures are counted apart from divergences.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]

use port_io::inflate::{InflateError, ZipLimits, inflate_into, is_threemf_part, unzip_filtered};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const LIMITS: ZipLimits = ZipLimits {
    max_entry_bytes: 64 * 1024 * 1024,
    max_entries: 1 << 20,
};

fn stored_zip(name: &str, data: &[u8]) -> Vec<u8> {
    let mut z = Vec::new();
    z.extend(0x0403_4b50u32.to_le_bytes());
    z.extend([20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    z.extend(0u32.to_le_bytes());
    z.extend((data.len() as u32).to_le_bytes());
    z.extend((data.len() as u32).to_le_bytes());
    z.extend((name.len() as u16).to_le_bytes());
    z.extend(0u16.to_le_bytes());
    z.extend(name.as_bytes());
    z.extend(data);
    let cd = z.len() as u32;
    z.extend(0x0201_4b50u32.to_le_bytes());
    z.extend([20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    z.extend(0u32.to_le_bytes());
    z.extend((data.len() as u32).to_le_bytes());
    z.extend((data.len() as u32).to_le_bytes());
    z.extend((name.len() as u16).to_le_bytes());
    z.extend([0u8; 12]);
    z.extend(0u32.to_le_bytes());
    z.extend(name.as_bytes());
    let cd_len = z.len() as u32 - cd;
    z.extend(0x0605_4b50u32.to_le_bytes());
    z.extend([0, 0, 0, 0, 1, 0, 1, 0]);
    z.extend(cd_len.to_le_bytes());
    z.extend(cd.to_le_bytes());
    z.extend(0u16.to_le_bytes());
    z
}

#[test]
fn stored_entry_is_read_and_filtered() {
    let z = stored_zip("3D/3dmodel.MODEL", b"<model/>");
    let got = unzip_filtered(&z, is_threemf_part, LIMITS).unwrap();
    assert_eq!(
        got,
        vec![("3D/3dmodel.MODEL".to_string(), b"<model/>".to_vec())]
    );
    let none = unzip_filtered(&stored_zip("a.png", b"x"), is_threemf_part, LIMITS).unwrap();
    assert!(none.is_empty());
}

#[test]
fn damaged_archives_fail_with_the_originals_codes() {
    let e = unzip_filtered(b"not a zip", is_threemf_part, LIMITS).unwrap_err();
    assert_eq!(
        e,
        InflateError::Fflate {
            code: 13,
            message: "invalid zip data".into()
        }
    );
    // A fixed-Huffman block that ends before its end-of-block code.
    assert!(matches!(
        inflate_into(&[0x03], 4),
        Err(InflateError::Fflate { code: 0, .. })
    ));
    // Block type 3 is invalid.
    assert!(matches!(
        inflate_into(&[0x07], 4),
        Err(InflateError::Fflate { code: 1, .. })
    ));
    // An empty stream yields the declared size of zeros, as the original does.
    assert_eq!(inflate_into(&[], 3).unwrap(), vec![0, 0, 0]);
}

#[test]
fn stored_deflate_block_and_overflow() {
    // BFINAL stored block, LEN 3: "abc".
    let raw = [0x01, 3, 0, 0xfc, 0xff, b'a', b'b', b'c'];
    assert_eq!(inflate_into(&raw, 3).unwrap(), b"abc");
    assert_eq!(inflate_into(&raw, 5).unwrap(), b"abc");
    assert_eq!(inflate_into(&raw, 2), Err(InflateError::OffsetOutOfBounds));
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const EDGES: [u32; 10] = [
    0,
    1,
    0xff,
    0xffff,
    0x7fff_ffff,
    0x8000_0000,
    0xffff_ffff,
    0x0001_0000,
    30,
    22,
];

fn mutate(rng: &mut Rng, mut b: Vec<u8>) -> Vec<u8> {
    for _ in 0..1 + rng.below(3) {
        if b.is_empty() {
            b.push(rng.below(256) as u8);
            continue;
        }
        // A third of the edits aim at the headers near the end, a third at the entry data
        // (the first 60% of the file), a third anywhere.
        let at = match rng.below(3) {
            0 => b.len() - 1 - rng.below(b.len().min(130)),
            1 => rng.below((b.len() * 6 / 10).max(1)),
            _ => rng.below(b.len()),
        };
        match rng.below(7) {
            0 => b.truncate(at),
            1 => b[at] = rng.below(256) as u8,
            2 => b.insert(at, rng.below(256) as u8),
            3 => {
                b.remove(at);
            }
            4 => b[at] ^= 1 << rng.below(8),
            5 => {
                let v = EDGES[rng.below(EDGES.len())].to_le_bytes();
                let n = if rng.below(2) == 0 { 2 } else { 4 };
                for (k, byte) in v.iter().take(n).enumerate() {
                    if at + k < b.len() {
                        b[at + k] = *byte;
                    }
                }
            }
            _ => {
                let other = rng.below(b.len());
                b.swap(at, other);
            }
        }
    }
    b
}

fn framed(inputs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = (inputs.len() as u32).to_le_bytes().to_vec();
    for i in inputs {
        out.extend((i.len() as u32).to_le_bytes());
        out.extend(i);
    }
    out
}

fn node(root: &Path, args: &[&str], stdin: Vec<u8>) -> Vec<u8> {
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/fflate-zip.cjs"))
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start node");
    let mut sin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        let _ = sin.write_all(&stdin);
    });
    let mut out = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut out).unwrap();
    writer.join().unwrap();
    assert!(child.wait().unwrap().success(), "original process failed");
    out
}

fn split(data: &[u8]) -> Vec<Vec<u8>> {
    let n = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    let mut at = 4;
    (0..n)
        .map(|_| {
            let l = u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize;
            at += 4;
            at += l;
            data[at - l..at].to_vec()
        })
        .collect()
}

#[derive(Debug, PartialEq)]
enum Original {
    Ok(Vec<(String, Vec<u8>)>),
    Err(u32, String),
    AllocationFailed,
    TooLarge,
}

fn parse_original(data: &[u8], count: usize) -> Vec<Original> {
    let mut at = 0;
    let u32_at = |at: &mut usize| {
        let v = u32::from_le_bytes(data[*at..*at + 4].try_into().unwrap());
        *at += 4;
        v as usize
    };
    (0..count)
        .map(|_| {
            let status = data[at];
            at += 1;
            match status {
                1 => {
                    let n = u32_at(&mut at);
                    let mut entries = Vec::new();
                    for _ in 0..n {
                        let nl = u32_at(&mut at);
                        let name = String::from_utf8(data[at..at + nl].to_vec()).unwrap();
                        at += nl;
                        let dl = u32_at(&mut at);
                        entries.push((name, data[at..at + dl].to_vec()));
                        at += dl;
                    }
                    Original::Ok(entries)
                }
                3 => {
                    u32_at(&mut at);
                    u32_at(&mut at);
                    Original::TooLarge
                }
                s => {
                    let code = u32_at(&mut at) as u32;
                    let ml = u32_at(&mut at);
                    let msg = String::from_utf8_lossy(&data[at..at + ml]).into_owned();
                    at += ml;
                    if s == 2 {
                        Original::AllocationFailed
                    } else {
                        Original::Err(code, msg)
                    }
                }
            }
        })
        .collect()
}

#[test]
#[ignore = "live comparison with the original's fflate; needs Node 22 and the pinned lockfile"]
fn pinned_zip_inflate_comparison() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let sabotage = std::env::var_os("SCANRULER_ZIP_SABOTAGE").is_some();
    let seeds: Vec<u64> = std::env::var("SCANRULER_ZIP_SEEDS")
        .unwrap_or_else(|_| "1,7,99".into())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let cases: usize = std::env::var("SCANRULER_ZIP_CASES")
        .unwrap_or_else(|_| "1500".into())
        .parse()
        .unwrap();
    let (mut same_ok, mut same_err, mut host, mut diverged) = (0, 0, 0, 0);
    let mut codes = std::collections::BTreeMap::<String, usize>::new();
    for seed in seeds {
        let base_blob = node(
            &root,
            &["gen", &seed.to_string(), &cases.to_string()],
            Vec::new(),
        );
        let bases = split(&base_blob);
        let mut rng = Rng(seed ^ 0xfeed_f00d);
        // Unmutated archives first (they must read), then mutated ones.
        let mut inputs: Vec<Vec<u8>> = bases.clone();
        inputs.extend(bases.iter().map(|b| mutate(&mut rng, b.clone())));
        let originals = parse_original(&node(&root, &["unzip"], framed(&inputs)), inputs.len());
        for (k, (input, orig)) in inputs.iter().zip(&originals).enumerate() {
            let mut rust = unzip_filtered(input, is_threemf_part, LIMITS);
            if sabotage
                && let Ok(entries) = &mut rust
                && let Some(e) = entries.first_mut()
            {
                e.1.push(0);
            }
            let matched = match (&rust, orig) {
                (Err(InflateError::ResourceLimit), _)
                | (_, Original::AllocationFailed | Original::TooLarge) => {
                    host += 1;
                    continue;
                }
                (Ok(r), Original::Ok(o)) => {
                    if r == o {
                        same_ok += 1;
                        true
                    } else {
                        false
                    }
                }
                (Err(InflateError::Fflate { code, message }), Original::Err(oc, om)) => {
                    if u32::from(*code) == *oc && message == om {
                        same_err += 1;
                        *codes.entry(message.clone()).or_default() += 1;
                        true
                    } else {
                        false
                    }
                }
                (Err(InflateError::OffsetOutOfBounds), Original::Err(255, om))
                    if om == "offset is out of bounds" =>
                {
                    same_err += 1;
                    true
                }
                _ => false,
            };
            if !matched {
                diverged += 1;
                let dir = std::env::temp_dir().join("scanruler-zip-fuzz");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(format!("seed{seed}-case{k}.zip")), input).unwrap();
                std::fs::write(
                    dir.join(format!("seed{seed}-case{k}.txt")),
                    format!("original: {orig:?}\nrust: {rust:?}\n"),
                )
                .unwrap();
            }
        }
    }
    println!("refusal kinds: {codes:?}");
    println!(
        "zip/inflate identical_success={same_ok} identical_refusal={same_err} host_limit={host} divergences={diverged}"
    );
    if sabotage {
        assert!(diverged > 0, "sabotaged Rust output was not reported");
    } else {
        assert_eq!(
            diverged, 0,
            "divergences; reproducers in the temp directory"
        );
        assert!(
            same_ok > 0,
            "no archive read successfully; the corpus is broken"
        );
    }
}
