// SPDX-License-Identifier: AGPL-3.0-only
//! Standing seeded mutation fuzz against the frozen original (`cargo xtask fuzz`).
//!
//! Generates deterministic inputs from a seed, runs the original (one Node process per batch,
//! `docs/port-plan/oracle/fuzz-batch.cjs`) and the Rust parser on independent copies, and compares
//! success/failure, progress text, error text and output bytes. Rust host-limit refusals are
//! counted as their own class, never as divergences and never as matches.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use port_io::ply::{PlyLimits, read_ply};
use port_io::stl_dispatch::{StlError, StlLimits, read_stl};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    fn float(&mut self) -> f64 {
        (self.next() % 2_000_001) as f64 / 1000.0 - 1000.0
    }
}

const DICT: &[&str] = &[
    "ascii",
    "binary_little_endian",
    "binary_big_endian",
    "float",
    "double",
    "uchar",
    "char",
    "short",
    "ushort",
    "int",
    "uint",
    "float32",
    "uint8",
    "int8",
    "list",
    "property",
    "element",
    "vertex",
    "face",
    "x",
    "y",
    "z",
    "end_header",
    "comment ScanRuler: keep winding",
    "format",
    "ply",
    "NaN",
    "Infinity",
    "-Infinity",
    "-0",
    "1e400",
    "0x10",
    "1e",
    "solid",
    "endsolid",
    "facet",
    " ",
    "\n",
    "\r\n",
    "\t",
    "\u{00a0}",
    "\u{feff}",
    "0",
    "1",
    "-1",
    "3",
    "4",
];
const NUMBERS: &[&str] = &[
    "0",
    "1",
    "-1",
    "2",
    "3",
    "255",
    "256",
    "65535",
    "2147483647",
    "2147483648",
    "4294967295",
    "4294967296",
    "1e21",
    "-1e21",
    "99999999999999999999",
    "0.5",
    "NaN",
];

fn ply_model(rng: &mut Rng) -> Vec<u8> {
    let format = ["ascii", "binary_little_endian", "binary_big_endian"][rng.below(3)];
    let types = [
        "float", "double", "uchar", "char", "short", "ushort", "int", "uint",
    ];
    let vtype = types[rng.below(types.len())];
    let nv = 3 + rng.below(6);
    let nf = 1 + rng.below(4);
    let mut head = format!("ply\nformat {format} 1.0\n");
    if rng.below(4) == 0 {
        head.push_str("comment ScanRuler: keep winding\n");
    }
    head.push_str(&format!(
        "element vertex {nv}\nproperty {vtype} x\nproperty {vtype} y\nproperty {vtype} z\n"
    ));
    head.push_str(&format!(
        "element face {nf}\nproperty list uchar int vertex_indices\nend_header\n"
    ));
    let mut out = head.into_bytes();
    let big = format == "binary_big_endian";
    let scalar = |rng: &mut Rng, out: &mut Vec<u8>| {
        let v = rng.float();
        if format == "ascii" {
            let s = match vtype {
                "float" | "double" => format!("{v} "),
                _ => format!("{} ", (v as i64).rem_euclid(120)),
            };
            out.extend_from_slice(s.as_bytes());
            return;
        }
        let bytes: Vec<u8> = match vtype {
            "float" => (v as f32).to_le_bytes().to_vec(),
            "double" => v.to_le_bytes().to_vec(),
            "uchar" | "char" => vec![rng.below(256) as u8],
            "short" | "ushort" => (rng.below(65536) as u16).to_le_bytes().to_vec(),
            _ => (rng.next() as u32).to_le_bytes().to_vec(),
        };
        if big {
            out.extend(bytes.iter().rev());
        } else {
            out.extend(bytes);
        }
    };
    for _ in 0..nv {
        for _ in 0..3 {
            scalar(rng, &mut out);
        }
        if format == "ascii" {
            out.push(b'\n');
        }
    }
    for _ in 0..nf {
        let k = 3 + rng.below(3);
        if format == "ascii" {
            out.extend_from_slice(format!("{k}").as_bytes());
            for _ in 0..k {
                out.extend_from_slice(format!(" {}", rng.below(nv)).as_bytes());
            }
            out.push(b'\n');
        } else {
            out.push(k as u8);
            for _ in 0..k {
                let idx = (rng.below(nv) as u32).to_le_bytes();
                if big {
                    out.extend(idx.iter().rev());
                } else {
                    out.extend(idx);
                }
            }
        }
    }
    out
}

fn stl_model(rng: &mut Rng) -> Vec<u8> {
    let n = 1 + rng.below(4);
    if rng.below(2) == 0 {
        let mut s = String::from("solid fuzz\n");
        for _ in 0..n {
            s.push_str("facet normal 0 0 1\nouter loop\n");
            for _ in 0..3 {
                s.push_str(&format!(
                    "vertex {} {} {}\n",
                    rng.float(),
                    rng.float(),
                    rng.float()
                ));
            }
            s.push_str("endloop\nendfacet\n");
        }
        s.push_str("endsolid fuzz\n");
        s.into_bytes()
    } else {
        let mut b = vec![0u8; 80];
        if rng.below(3) == 0 {
            b[..5].copy_from_slice(b"solid");
        }
        b.extend((n as u32).to_le_bytes());
        for _ in 0..n {
            for _ in 0..12 {
                b.extend((rng.float() as f32).to_le_bytes());
            }
            b.extend([0, 0]);
        }
        b
    }
}

fn mutate(rng: &mut Rng, mut b: Vec<u8>) -> Vec<u8> {
    for _ in 0..1 + rng.below(3) {
        if b.is_empty() {
            b.push(rng.below(256) as u8);
            continue;
        }
        let at = rng.below(b.len());
        match rng.below(7) {
            0 => b.truncate(at),
            1 => b[at] = rng.below(256) as u8,
            2 => b.insert(at, rng.below(256) as u8),
            3 => {
                b.remove(at);
            }
            4 => {
                let w = DICT[rng.below(DICT.len())].as_bytes();
                let end = (at + 1 + rng.below(8)).min(b.len());
                b.splice(at..end, w.iter().copied());
            }
            5 => {
                // replace the digit run nearest `at` with an edge number
                let mut s = at;
                while s < b.len() && !b[s].is_ascii_digit() {
                    s += 1;
                }
                let mut e = s;
                while e < b.len() && b[e].is_ascii_digit() {
                    e += 1;
                }
                if s < b.len() {
                    b.splice(s..e, NUMBERS[rng.below(NUMBERS.len())].bytes());
                }
            }
            _ => b[at] ^= 1 << rng.below(8),
        }
    }
    b
}

struct Outcome {
    timed_out: bool,
    ok: bool,
    progress: String,
    message: String,
    output: Vec<u8>,
}

fn ply_limits() -> PlyLimits {
    PlyLimits {
        max_bytes: 1 << 20,
        max_vertices: 100_000,
        max_triangles: 200_000,
        max_work: 5_000_000,
    }
}

/// None means the Rust side hit a host-limit refusal (a separate class).
fn rust(unit: &str, input: &[u8], sabotage: bool) -> Option<Outcome> {
    let mut lines: Vec<String> = Vec::new();
    let (ok, message, mut output) = if unit == "ply" {
        match read_ply(input, ply_limits(), |s| lines.push(s.to_string())) {
            Ok(m) => {
                let mut o = b"PLY1".to_vec();
                o.extend(u32::from(m.keep_winding).to_le_bytes());
                o.extend((m.positions.len() as u32).to_le_bytes());
                o.extend((m.indices.len() as u32).to_le_bytes());
                for p in &m.positions {
                    o.extend(p.to_le_bytes());
                }
                for i in &m.indices {
                    o.extend(i.to_le_bytes());
                }
                (true, String::new(), o)
            }
            Err(port_io::ply::PlyError::ResourceLimit | port_io::ply::PlyError::Allocation) => {
                return None;
            }
            Err(e) => (false, e.to_string(), Vec::new()),
        }
    } else {
        let limits = StlLimits {
            max_bytes: 1 << 20,
            max_triangles: 200_000,
        };
        match read_stl(input, limits, |s| lines.push(s.to_string())) {
            Ok(v) => (
                true,
                String::new(),
                v.iter().flat_map(|f| f.to_le_bytes()).collect(),
            ),
            Err(StlError::InputLimit | StlError::TriangleLimit | StlError::Allocation) => {
                return None;
            }
            Err(e) => (false, e.to_string(), Vec::new()),
        }
    };
    if sabotage && let Some(last) = output.last_mut() {
        *last ^= 1;
    }
    Some(Outcome {
        timed_out: false,
        ok,
        progress: lines.join("\n"),
        message,
        output,
    })
}

fn original(unit: &str, root: &Path, inputs: &[Vec<u8>]) -> Vec<Outcome> {
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/fuzz-batch.cjs"))
        .arg(unit)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start node");
    let mut stdin = child.stdin.take().unwrap();
    let mut framed = (inputs.len() as u32).to_le_bytes().to_vec();
    for i in inputs {
        framed.extend((i.len() as u32).to_le_bytes());
        framed.extend(i);
    }
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&framed);
    });
    let mut stdout = child.stdout.take().unwrap();
    let mut data = Vec::new();
    stdout.read_to_end(&mut data).unwrap();
    writer.join().unwrap();
    assert!(
        child.wait().unwrap().success(),
        "original batch process failed"
    );
    let mut at = 0;
    let mut take = |n: usize| {
        let s = &data[at..at + n];
        at += n;
        s.to_vec()
    };
    let mut out = Vec::new();
    for _ in inputs {
        let status = take(1)[0];
        let ok = status == 1;
        let field = |take: &mut dyn FnMut(usize) -> Vec<u8>| {
            let n = u32::from_le_bytes(take(4).try_into().unwrap()) as usize;
            take(n)
        };
        let progress = String::from_utf8_lossy(&field(&mut take)).into_owned();
        let message = String::from_utf8_lossy(&field(&mut take)).into_owned();
        let output = field(&mut take);
        out.push(Outcome {
            timed_out: status == 2,
            ok,
            progress,
            message,
            output,
        });
    }
    out
}

// Match categories and success/refusal must survive reduction. Host limits and
// reference timeouts are never evidence that a reduced input preserves a bug.
fn signature(o: &Outcome, r: &Outcome) -> Option<(bool, bool, u8)> {
    if o.timed_out || r.timed_out {
        return None;
    }
    let bits = u8::from(o.ok != r.ok)
        | (u8::from(o.progress != r.progress) << 1)
        | (u8::from(o.message != r.message) << 2)
        | (u8::from(o.output != r.output) << 3);
    (bits != 0).then_some((o.ok, r.ok, bits))
}

// Deterministic chunk deletion, with a hard evaluation cap. Every accepted
// candidate is independently evaluated; no reference answer enters Rust.
fn reduce(
    mut input: Vec<u8>,
    budget: usize,
    mut preserves: impl FnMut(&[u8]) -> bool,
) -> (Vec<u8>, usize) {
    let mut chunk = input.len().max(1);
    let mut attempts = 0;
    while chunk > 0 && !input.is_empty() && attempts < budget {
        let mut offset = 0;
        while offset < input.len() && attempts < budget {
            let end = (offset + chunk).min(input.len());
            let mut candidate = input[..offset].to_vec();
            candidate.extend_from_slice(&input[end..]);
            attempts += 1;
            if preserves(&candidate) {
                input = candidate;
            } else {
                offset = end;
            }
        }
        chunk /= 2;
    }
    (input, attempts)
}

fn save_outcome(dir: &Path, name: &str, value: &Outcome) {
    std::fs::write(dir.join(format!("{name}.bin")), &value.output).unwrap();
    std::fs::write(dir.join(format!("{name}.progress.txt")), &value.progress).unwrap();
    std::fs::write(dir.join(format!("{name}.message.txt")), &value.message).unwrap();
    std::fs::write(
        dir.join(format!("{name}.status.txt")),
        format!("ok={} timeout={}\n", value.ok, value.timed_out),
    )
    .unwrap();
}

#[test]
fn reducer_preserves_predicate_and_budget() {
    let input = b"prefix BUG suffix".to_vec();
    let (reduced, calls) = reduce(input.clone(), 128, |b| b.windows(3).any(|w| w == b"BUG"));
    assert_eq!(reduced, b"BUG");
    assert!(calls <= 128);
    let mut observed = 0;
    let (same, calls) = reduce(input.clone(), 3, |_| {
        observed += 1;
        false
    });
    assert_eq!(same, input);
    assert_eq!((calls, observed), (3, 3));
    assert_eq!(
        reduce(Vec::new(), 10, |_| panic!("empty input")),
        (Vec::new(), 0)
    );
    assert_eq!(reduce(vec![1], 0, |_| panic!("zero budget")), (vec![1], 0));
}

#[test]
fn mismatch_signature_rejects_timeouts_and_changed_failure_class() {
    let make = || Outcome {
        timed_out: false,
        ok: true,
        progress: String::new(),
        message: String::new(),
        output: vec![1],
    };
    let a = make();
    let mut b = make();
    assert_eq!(signature(&a, &b), None);
    b.output[0] = 2;
    let wanted = signature(&a, &b);
    assert_eq!(wanted, Some((true, true, 8)));
    b.ok = false;
    assert_ne!(signature(&a, &b), wanted);
    b.timed_out = true;
    assert_eq!(signature(&a, &b), None);
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[test]
#[ignore = "standing fuzz: run through `cargo xtask fuzz`; needs Node 22 and the pinned Git object"]
fn fuzz_against_original() {
    let unit = env_or("SCANRULER_FUZZ_UNIT", "ply");
    let seed: u64 = env_or("SCANRULER_FUZZ_SEED", "1").parse().unwrap();
    let cases: usize = env_or("SCANRULER_FUZZ_CASES", "500").parse().unwrap();
    let sabotage = std::env::var_os("SCANRULER_FUZZ_SABOTAGE").is_some();
    assert!(unit == "ply" || unit == "stl", "unit must be ply or stl");
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let out_dir = PathBuf::from(env_or(
        "SCANRULER_FUZZ_OUT",
        &std::env::temp_dir()
            .join("scanruler-fuzz")
            .to_string_lossy(),
    ));
    let mut rng = Rng(seed ^ 0x5eed_5eed_5eed_5eed);
    let (mut identical_ok, mut identical_err, mut host_limit, mut divergences) =
        (0usize, 0usize, 0usize, 0usize);
    let mut original_timeout = 0usize;
    let minimize = std::env::var_os("SCANRULER_FUZZ_MINIMIZE").is_some();
    let mut minimized = false;
    let mut done = 0;
    while done < cases {
        let batch = (cases - done).min(1000);
        let inputs: Vec<Vec<u8>> = (0..batch)
            .map(|_| {
                let base = if unit == "ply" {
                    ply_model(&mut rng)
                } else {
                    stl_model(&mut rng)
                };
                if rng.below(10) == 0 {
                    base
                } else {
                    mutate(&mut rng, base)
                }
            })
            .collect();
        let originals = original(&unit, &root, &inputs);
        for (k, (input, o)) in inputs.iter().zip(&originals).enumerate() {
            if o.timed_out {
                original_timeout += 1;
                continue;
            }
            let Some(r) = rust(&unit, input, sabotage) else {
                host_limit += 1;
                continue;
            };
            if r.ok == o.ok
                && r.progress == o.progress
                && r.message == o.message
                && r.output == o.output
            {
                if r.ok {
                    identical_ok += 1
                } else {
                    identical_err += 1
                }
            } else {
                divergences += 1;
                std::fs::create_dir_all(&out_dir).unwrap();
                let name = format!("{unit}-seed{seed}-case{}", done + k);
                std::fs::write(out_dir.join(format!("{name}.bin")), input).unwrap();
                if minimize && !minimized {
                    minimized = true;
                    let wanted = signature(o, &r).unwrap();
                    let (small, attempts) = reduce(input.clone(), 128, |candidate| {
                        let Some(candidate_rust) = rust(&unit, candidate, sabotage) else {
                            return false;
                        };
                        let candidate_original = original(&unit, &root, &[candidate.to_vec()]);
                        signature(&candidate_original[0], &candidate_rust) == Some(wanted)
                    });
                    // Recheck before writing a reduced reproducer. Nondeterminism
                    // is a failed reduction, never an accepted match.
                    let small_original = original(&unit, &root, std::slice::from_ref(&small));
                    let small_rust = rust(&unit, &small, sabotage).unwrap();
                    assert_eq!(signature(&small_original[0], &small_rust), Some(wanted));
                    let dir = out_dir.join(format!("{unit}-seed{seed}-case{}-reduced", done + k));
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(dir.join("input.bin"), &small).unwrap();
                    save_outcome(&dir, "original", &small_original[0]);
                    save_outcome(&dir, "rust", &small_rust);
                    std::fs::write(dir.join("reduction.txt"), format!("seed={seed} case={} original_bytes={} reduced_bytes={} attempts={attempts} budget=128 signature={wanted:?} sabotage={sabotage}\n", done+k, input.len(), small.len())).unwrap();
                    println!(
                        "reduced reproducer {} -> {} bytes in {attempts} attempts: {}",
                        input.len(),
                        small.len(),
                        dir.display()
                    );
                }
                std::fs::write(
                    out_dir.join(format!("{name}.txt")),
                    format!(
                        "original ok={} message={:?} progress={:?} out_len={}\nrust     ok={} message={:?} progress={:?} out_len={}\n",
                        o.ok, o.message, o.progress, o.output.len(), r.ok, r.message, r.progress, r.output.len()
                    ),
                )
                .unwrap();
            }
        }
        done += batch;
    }
    println!(
        "fuzz unit={unit} seed={seed} cases={cases} identical_success={identical_ok} identical_refusal={identical_err} host_limit={host_limit} original_timeout={original_timeout} divergences={divergences} reproducers={}",
        out_dir.display()
    );
    if sabotage {
        assert!(
            divergences > 0,
            "sabotaged Rust output was not reported as a divergence"
        );
    } else {
        assert_eq!(
            divergences,
            0,
            "divergences found; reproducers in {}",
            out_dir.display()
        );
    }
}
