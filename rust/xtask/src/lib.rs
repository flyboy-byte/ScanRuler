// SPDX-License-Identifier: AGPL-3.0-only
//! Milestone observation adapters; the source test comparators remain authoritative.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::{
    path::{Path, PathBuf},
    process::Command,
};
/// Repository containing this tool.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
struct Recording {
    path: PathBuf,
}
impl Recording {
    fn new(directory: &str) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = root().join(format!("rust/target/{directory}/run-{stamp}"));
        std::fs::create_dir_all(&path).unwrap();
        let head = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root())
            .output()
            .unwrap();
        assert!(head.status.success());
        std::fs::write(path.join("candidate.txt"), head.stdout).unwrap();
        Self { path }
    }
    fn case(&self, id: &str, input: &[u8], original: &[u8], rust: &[u8]) {
        let dir = self.path.join(id);
        std::fs::create_dir(&dir).unwrap();
        for (name, bytes) in [
            ("input.bin", input),
            ("original.bin", original),
            ("rust.bin", rust),
        ] {
            std::fs::write(dir.join(name), bytes).unwrap();
        }
    }
}
#[allow(dead_code, clippy::all)]
mod zip {
    include!(concat!(env!("OUT_DIR"), "/zip.rs"));
}
#[allow(dead_code, clippy::all)]
mod threemf {
    include!(concat!(env!("OUT_DIR"), "/threemf.rs"));
}
#[allow(dead_code, clippy::all)]
mod png {
    include!(concat!(env!("OUT_DIR"), "/png.rs"));
}
/// Capture the unchanged ZIP comparison corpus.
pub fn capture_zip() {
    zip::capture();
}
/// Capture the unchanged 3MF comparison corpus.
pub fn capture_threemf() {
    threemf::capture();
}
/// Compare and capture committed Chrome PNG outputs.
pub fn capture_png() {
    png::capture();
}

/// The project suite already compares its full JSONL and writes each native state.
/// Materialize those same observations for the inventory, without rerunning semantics.
pub fn capture_project() {
    let parent = root().join("rust/target/native-project-diff");
    let latest = std::fs::read_dir(&parent)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
        .max_by_key(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .strip_prefix("run-")
                .unwrap()
                .parse::<u128>()
                .unwrap()
        })
        .unwrap();
    let cases = std::fs::read_to_string(latest.join("cases.tsv")).unwrap();
    let original = std::fs::read_to_string(latest.join("original.jsonl")).unwrap();
    let rust = std::fs::read_to_string(latest.join("rust.jsonl")).unwrap();
    assert_eq!(cases.lines().count(), original.lines().count());
    assert_eq!(cases.lines().count(), rust.lines().count());
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(head.status.success());
    std::fs::write(latest.join("candidate.txt"), head.stdout).unwrap();
    for ((input, original), rust) in cases.lines().zip(original.lines()).zip(rust.lines()) {
        let id = input.split('\t').next().unwrap();
        let dir = latest.join(id);
        assert!(dir.is_dir());
        for (name, bytes) in [
            ("input.bin", input),
            ("original.bin", original),
            ("rust.bin", rust),
        ] {
            std::fs::write(dir.join(name), bytes).unwrap();
        }
    }
}
