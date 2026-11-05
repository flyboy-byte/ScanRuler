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

#[test]
fn fit_plane_executes_registered_fit_and_formats_result() {
    let scratch = Scratch::new();
    let path = scratch.0.join("grid.ply");
    let bytes = grid_ply();
    std::fs::write(&path, bytes).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["fit-plane", path.to_str().unwrap(), "--sigma", "0"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stderr.is_empty());
    let json = String::from_utf8(out.stdout).unwrap();
    assert!(json.contains("\"center\":[2,2,0],\"normal\":[0,0,1]"));
    assert!(json.contains("\"sigmaCutoff\":0,\"cutoff\":\"all points\",\"markedPoints\":25"));
}

fn grid_ply() -> String {
    let mut bytes = String::from(
        "ply\nformat ascii 1.0\nelement vertex 25\nproperty float x\nproperty float y\nproperty float z\nelement face 32\nproperty list uchar int vertex_indices\nend_header\n",
    );
    for y in 0..5 {
        for x in 0..5 {
            bytes += &format!("{x} {y} 0\n")
        }
    }
    for y in 0..4 {
        for x in 0..4 {
            let a = y * 5 + x;
            bytes += &format!(
                "3 {a} {} {}\n3 {} {} {}\n",
                a + 1,
                a + 5,
                a + 1,
                a + 6,
                a + 5
            )
        }
    }
    bytes
}

#[test]
fn box_and_marked_cli_paths_use_real_selection_and_finishing() {
    let scratch = Scratch::new();
    let path = scratch.0.join("part.ply");
    std::fs::write(&path, grid_ply()).unwrap();
    for marked in [false, true] {
        let mut args = vec![
            "fit-plane",
            path.to_str().unwrap(),
            "--box",
            "0",
            "0",
            "-1",
            "3",
            "4",
            "1",
            "--normal",
            "0",
            "0",
            "1",
            "--orient",
            "XY",
            "--extend",
            "1",
            "2",
            "3",
            "4",
        ];
        if marked {
            args.push("--marked");
        }
        let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let json = String::from_utf8(out.stdout).unwrap();
        assert!(json.contains("\"center\":[1.5,2,0]"), "{json}");
        assert!(json.contains("\"markedPoints\":20"), "{json}");
        assert!(
            json.contains("\"orient\":{\"ref\":-1,\"relation\":\"normal\"}"),
            "{json}"
        );
        assert!(
            json.contains(
                "\"extend\":{\"kind\":\"plane\",\"uMin\":1,\"uMax\":2,\"vMin\":3,\"vMax\":4}"
            ),
            "{json}"
        );
        if marked {
            assert!(json.contains("\"selection\":{\"matched\":20"), "{json}");
            assert!(
                json.contains(
                    "\"marked\":{\"points\":20,\"from\":\"scan.query box facing a way\"}"
                ),
                "{json}"
            );
        } else {
            assert!(json.contains("\"points\":20"), "{json}");
        }
    }
    let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["fit-plane", path.to_str().unwrap(), "--marked"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("\"markedPoints\":25")
    );
}
#[test]
fn cli_selection_guards_and_engine_refusals_produce_no_partial_stdout() {
    let scratch = Scratch::new();
    let path = scratch.0.join("part.ply");
    std::fs::write(&path, grid_ply()).unwrap();
    for (flags, message) in [
        (vec!["--box", "0", "0"], "--box requires 6 numbers"),
        (
            vec!["--extend", "a", "2", "3", "4"],
            "--extend requires numbers",
        ),
        (vec!["--in-plane"], "--in-plane requires --orient"),
        (vec!["--normal", "0", "0", "1"], "--normal requires --box"),
        (vec!["--within-deg", "30"], "--within-deg requires --normal"),
        (
            vec!["--box", "3", "0", "-1", "0", "4", "1"],
            "invalid_input: The box’s min lies past its max.",
        ),
        (
            vec![
                "--box", "0", "0", "-1", "3", "4", "1", "--normal", "0", "0", "0",
            ],
            "invalid_input: normal.dir: give a direction, not a zero vector.",
        ),
        (
            vec!["--orient", "Missing"],
            "not_found: There is no element named \"Missing\".",
        ),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
            .args(["fit-plane", path.to_str().unwrap()])
            .args(flags)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert_eq!(String::from_utf8(out.stderr).unwrap().trim(), message);
    }
}

#[test]
fn standalone_obj_reaches_info_and_plane_fit_and_reports_parser_refusal() {
    let scratch = Scratch::new();
    let path = scratch.0.join("scan.obj");
    let mut obj = String::new();
    for y in 0..5 {
        for x in 0..5 {
            obj.push_str(&format!("v {x} {y} 0\n"));
        }
    }
    for y in 0..4 {
        for x in 0..4 {
            let a = y * 5 + x + 1;
            obj.push_str(&format!("f {a} {} {} {}\n", a + 1, a + 6, a + 5));
        }
    }
    std::fs::write(&path, obj).unwrap();
    let info = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["info", path.to_str().unwrap(), "--units", "cm"])
        .output()
        .unwrap();
    assert!(info.status.success(), "{:?}", info.stderr);
    assert!(info.stderr.is_empty());
    assert_eq!(
        String::from_utf8(info.stdout).unwrap(),
        "{\"fileName\":\"scan.obj\",\"units\":\"cm\",\"vertices\":25,\"triangles\":32,\"bounds\":{\"min\":[0,0,0],\"max\":[40,40,0]},\"center\":[20,20,0],\"size\":56.568542494923804,\"components\":1}\n"
    );
    let fit = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["fit-plane", path.to_str().unwrap(), "--sigma", "0"])
        .output()
        .unwrap();
    assert!(fit.status.success(), "{:?}", fit.stderr);
    assert!(fit.stderr.is_empty());
    let result = String::from_utf8(fit.stdout).unwrap();
    assert!(result.contains("\"kind\":\"plane\""), "{result}");
    assert!(result.contains("\"status\":\"done\""), "{result}");
    std::fs::write(&path, b"v 0 0 0\n").unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_scanruler-rs"))
        .args(["info", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(2));
    assert!(refused.stdout.is_empty());
    assert_eq!(
        String::from_utf8(refused.stderr).unwrap(),
        "failed: This OBJ contains no faces — point clouds are not supported yet.\n"
    );
}
