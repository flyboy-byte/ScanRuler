// SPDX-License-Identifier: AGPL-3.0-only
//! Black-box native filesystem/argument/presentation checks. Engine values are
//! separately paired with the pinned original in port-engine/native_import.rs.
#![allow(clippy::unwrap_used, clippy::panic)]
use std::{path::PathBuf, process::Command};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("scanruler-info-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn ply(encoding: &str) -> Vec<u8> {
    let mut bytes=format!("ply\nformat {encoding} 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n").into_bytes();
    if encoding == "ascii" {
        bytes.extend_from_slice(b"0 0 0\n1 0 0\n0 1 0\n1 1 0\n3 0 1 2\n3 1 3 2\n");
        return bytes;
    }
    let le = encoding == "binary_little_endian";
    for v in [0_f32, 0., 0., 1., 0., 0., 0., 1., 0., 1., 1., 0.] {
        bytes.extend_from_slice(&if le { v.to_le_bytes() } else { v.to_be_bytes() })
    }
    for face in [[0_i32, 1, 2], [1, 3, 2]] {
        bytes.push(3);
        for v in face {
            bytes.extend_from_slice(&if le { v.to_le_bytes() } else { v.to_be_bytes() })
        }
    }
    bytes
}
#[test]
fn info_reads_all_ply_encodings_and_formats_the_engine_summary() {
    let scratch = Scratch::new();
    let path = scratch.0.join("scan.ply");
    for encoding in ["ascii", "binary_little_endian", "binary_big_endian"] {
        std::fs::write(&path, ply(encoding)).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
            .args(["info", path.to_str().unwrap(), "--units", "cm"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stderr.is_empty());
        // Exact original scan.open summary for these independent controls, plus the
        // engine's already-computed connected-component count.
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            "{\"fileName\":\"scan.ply\",\"units\":\"cm\",\"vertices\":4,\"triangles\":2,\"bounds\":{\"min\":[0,0,0],\"max\":[10,10,0]},\"center\":[5,5,0],\"size\":14.142135623730951,\"components\":1}\n"
        );
    }
}
#[test]
fn info_keeps_failures_off_stdout_and_exits_nonzero() {
    let scratch = Scratch::new();
    let path = scratch.0.join("broken.ply");
    std::fs::write(&path, b"ply\nnot a header\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["info", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .starts_with("failed:")
    );
    std::fs::write(&path, ply("ascii")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["info", path.to_str().unwrap(), "--units", "furlongs"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .starts_with("invalid_input:")
    );
}
