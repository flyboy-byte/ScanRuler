// SPDX-License-Identifier: AGPL-3.0-only
//! Fixed independent source/native parser comparisons, not graph/session acceptance.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use port_io::ply_ascii::{PlyError, PlyLimits, PlyMesh, read_ply_ascii};

struct Case {
    id: &'static str,
    input: &'static [u8],
    success: bool,
}
fn cases() -> Vec<Case> {
    include!("support/ply_ascii_cases.rs")
}
fn limits() -> PlyLimits {
    PlyLimits {
        max_bytes: 2_000_000,
        max_vertices: 100_000,
        max_triangles: 100_000,
        max_work: 1_000_000,
    }
}
fn payload(mesh: &PlyMesh) -> Vec<u8> {
    let mut out = b"PLY1".to_vec();
    out.extend(u32::from(mesh.keep_winding).to_le_bytes());
    out.extend((mesh.positions.len() as u32).to_le_bytes());
    out.extend((mesh.indices.len() as u32).to_le_bytes());
    out.extend(
        mesh.positions
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes()),
    );
    out.extend(mesh.indices.iter().flat_map(|v| v.to_le_bytes()));
    out
}

#[test]
fn ascii_ply_cases_and_host_guards() {
    for c in cases() {
        assert_eq!(
            read_ply_ascii(c.input, limits(), |_| {}).is_ok(),
            c.success,
            "{}",
            c.id
        );
    }
    let triangle = cases().remove(0);
    for l in [
        PlyLimits {
            max_bytes: 1,
            ..limits()
        },
        PlyLimits {
            max_vertices: 2,
            ..limits()
        },
        PlyLimits {
            max_triangles: 0,
            ..limits()
        },
        PlyLimits {
            max_work: 0,
            ..limits()
        },
    ] {
        assert_eq!(
            read_ply_ascii(triangle.input, l, |_| {}),
            Err(PlyError::ResourceLimit)
        );
    }
    let huge = b"ply\nformat ascii 1.0\nelement vertex 999999999999\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nend_header\n";
    assert_eq!(
        read_ply_ascii(huge, limits(), |_| {}),
        Err(PlyError::ResourceLimit)
    );
}

#[test]
fn binary_ply_remains_explicitly_unported() {
    for format in ["binary_little_endian", "binary_big_endian"] {
        let input =
            format!("ply\nformat {format} 1.0\nelement vertex 3\nelement face 1\nend_header\n");
        let mut progress = Vec::new();
        assert_eq!(
            read_ply_ascii(input.as_bytes(), limits(), |s| progress.push(s.to_owned())),
            Err(PlyError::BinaryNotPorted)
        );
        assert!(progress.is_empty());
    }
}

#[test]
#[ignore = "live PLY oracle requires Node 22, original TypeScript dependency and pinned Git object"]
fn pinned_ply_ascii_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let reports = root
        .join("rust/target/ply-ascii-diff")
        .join(format!("run-{stamp}"));
    std::fs::create_dir_all(&reports).unwrap();
    let identity = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(identity.status.success());
    std::fs::write(reports.join("candidate.txt"), &identity.stdout).unwrap();
    let mut run_identity = String::from(
        "contract: io.ply.ascii.bits.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: exact complete PLY1 bytes, progress and source refusals\nseeds: 1,7,99,1039\n",
    );
    for (label, program, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let r = Command::new(program)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(r.status.success());
        run_identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&r.stdout).trim()
        ));
    }
    std::fs::write(reports.join("identity.txt"), run_identity).unwrap();
    let cases = cases();
    assert_eq!(
        cases.len(),
        60,
        "review fixed schedule before changing case count"
    );
    let mut values = 0;
    let mut refusals = 0;
    for c in cases {
        let mut progress = String::new();
        let candidate = read_ply_ascii(c.input, limits(), |s| {
            progress.push_str(s);
            progress.push('\n');
        });
        // The new all-encoding dispatcher must preserve every existing ASCII
        // value and refusal, not merely delegate to a branch with a matching status.
        let mut dispatcher_progress = String::new();
        let dispatcher = port_io::ply::read_ply(c.input, limits(), |s| {
            dispatcher_progress.push_str(s);
            dispatcher_progress.push('\n');
        });
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/ply-ascii.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(c.input).unwrap();
        let original = child.wait_with_output().unwrap();
        let dir = reports.join(c.id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("input.bin"), c.input).unwrap();
        std::fs::write(dir.join("original.bin"), &original.stdout).unwrap();
        std::fs::write(dir.join("original.progress-or-error.txt"), &original.stderr).unwrap();
        let raw = match &candidate {
            Ok(mesh) => payload(mesh),
            Err(e) => {
                progress.push_str(&format!("{e}\n"));
                Vec::new()
            }
        };
        // Retain both outputs before any assertion, including an unexpected host failure.
        std::fs::write(dir.join("rust.bin"), &raw).unwrap();
        std::fs::write(dir.join("rust.progress-or-error.txt"), &progress).unwrap();
        let dispatched = match &dispatcher {
            Ok(mesh) => payload(mesh),
            Err(e) => {
                dispatcher_progress.push_str(&format!("{e}\n"));
                Vec::new()
            }
        };
        std::fs::write(dir.join("dispatcher.bin"), &dispatched).unwrap();
        std::fs::write(
            dir.join("dispatcher.progress-or-error.txt"),
            &dispatcher_progress,
        )
        .unwrap();
        assert_eq!(
            dispatcher.is_ok(),
            c.success,
            "{}: dispatcher outcome",
            c.id
        );
        assert_eq!(
            dispatched, original.stdout,
            "{}: dispatcher full values",
            c.id
        );
        assert_eq!(
            dispatcher_progress.as_bytes(),
            original.stderr,
            "{}: dispatcher progress/refusal",
            c.id
        );
        assert_eq!(
            candidate.is_ok(),
            c.success,
            "{}: Rust outcome: {candidate:?}",
            c.id
        );
        assert_eq!(
            original.status.success(),
            c.success,
            "{}: original outcome: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(
            raw, original.stdout,
            "{}: full values/kind/counts/winding",
            c.id
        );
        assert_eq!(
            progress.as_bytes(),
            original.stderr,
            "{}: progress/refusal",
            c.id
        );
        if c.success {
            assert!(raw.len() >= 16);
            values += 1;
            if c.id == "triangle" {
                let mut wrong_value = raw.clone();
                wrong_value[16] ^= 1;
                assert_ne!(
                    wrong_value, original.stdout,
                    "one changed coordinate bit must fail exact comparison"
                );
                assert_ne!(
                    &raw[..raw.len() - 4],
                    original.stdout,
                    "missing compared data must fail"
                );
            }
        } else {
            assert_eq!(original.status.code(), Some(2));
            refusals += 1;
        }
        println!("{}: exact values/state and progress/error", c.id);
    }
    assert_eq!((values, refusals), (46, 14));
    println!(
        "ASCII PLY: {values} complete value cases / {refusals} source refusals; raw {}",
        reports.display()
    );
}
