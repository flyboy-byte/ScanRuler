// SPDX-License-Identifier: AGPL-3.0-only
//! Full original file classification, identity, associations and preserved bytes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "support/obj_files_wire.rs"]
mod wire;
use port_io::scan_files::*;
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/obj_files_cases.rs")
}
#[test]
fn host_guards_and_non_aliasing_copies() {
    let mut set = FileSet::new(wire::limits());
    let m = set.insert("a.obj", b"model").unwrap();
    let t = set.insert("a.png", b"original").unwrap();
    assert_eq!(set.bundle(&[t, m, t]).unwrap(), Some(m));
    set.bundle(&[m]).unwrap();
    assert_eq!(set.companions(m).unwrap(), [t, t]);
    let mut copies = set.companion_bytes(&[t, t]).unwrap();
    copies[0].bytes[0] = 0;
    assert_eq!(copies[1].bytes, b"original");
    assert_eq!(set.file(t).unwrap().bytes, b"original");
    assert_eq!(set.bundle(&vec![t; 65]), Err(FileError::Limit));
    assert_eq!(set.companions(m).unwrap(), [t, t]);
    let mut small = FileSet::new(FileLimits {
        max_links: 1,
        ..wire::limits()
    });
    let m = small.insert("a.obj", b"").unwrap();
    let t = small.insert("a.mtl", b"").unwrap();
    small.bundle(&[m, t]).unwrap();
    assert_eq!(small.bundle(&[m, t, t]), Err(FileError::Limit));
    assert_eq!(small.companions(m).unwrap(), [t]);
    for l in [
        FileLimits {
            max_files: 0,
            ..wire::limits()
        },
        FileLimits {
            max_name_bytes: 0,
            ..wire::limits()
        },
        FileLimits {
            max_bytes: 0,
            ..wire::limits()
        },
    ] {
        assert_eq!(FileSet::new(l).insert("a.obj", b"x"), Err(FileError::Limit));
    }
    assert_eq!(
        FileSet::new(FileLimits {
            max_work: 0,
            ..wire::limits()
        })
        .insert("a.obj", b""),
        Err(FileError::Work)
    );
    assert_eq!(
        classify(
            "obj",
            FileLimits {
                max_work: 0,
                ..wire::limits()
            }
        ),
        Err(FileError::Work)
    );
}
#[test]
#[ignore = "live pinned original comparison requires Node22"]
fn pinned_obj_files_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/obj-files-diff")
        .join(format!("run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(run.join("candidate.txt"), sha.stdout).unwrap();
    let mut identity = String::from(
        "contract: io.obj.files.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\nseeds: 1,7,99,1039\ncomparison: exact complete FLS1 classification, identity, links, names/bytes/order\n",
    );
    for (label, p, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let o = Command::new(p)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(o.status.success());
        identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&o.stdout).trim()
        ));
    }
    std::fs::write(run.join("identity.txt"), identity).unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 87);
    for c in cases {
        let rust = wire::run(c.input);
        let mut child = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/obj-files.cjs"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(c.input).unwrap();
        let original = child.wait_with_output().unwrap();
        let dir = run.join(c.id);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, data) in [
            ("input.bin", c.input),
            ("original.bin", original.stdout.as_slice()),
            ("rust.bin", rust.as_slice()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(dir.join(name), data).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert!(original.stderr.is_empty(), "{}", c.id);
        assert!(!original.stdout.is_empty(), "{}", c.id);
        assert_eq!(rust, original.stdout, "{}: complete state", c.id);
        let mut changed = rust.clone();
        let last = changed.len() - 1;
        changed[last] ^= 1;
        assert_ne!(changed, original.stdout, "changed state cannot pass");
        assert_ne!(&rust[..last], original.stdout, "missing state cannot pass");
        println!("{}: full source state match", c.id);
    }
    println!("87 exact original states; raw {}", run.display());
}
