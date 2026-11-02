// SPDX-License-Identifier: AGPL-3.0-only
//! Full original file classification, identity, associations and preserved bytes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "support/drop_files_wire.rs"]
mod wire;

struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/drop_files_cases.rs")
}
#[test]
fn pending_callbacks_and_resource_errors_are_distinct() {
    use port_io::drop_files::*;
    let case = cases()
        .into_iter()
        .find(|c| c.id == "one-directory")
        .unwrap();
    let (data, mut source) = wire::parse(case.input);
    source.pending = true;
    assert_eq!(
        wire::block_on(dropped_files(data.as_ref(), &mut source, wire::limits())).unwrap(),
        [1, 2]
    );
    for limits in [
        DropLimits {
            max_items: 0,
            ..wire::limits()
        },
        DropLimits {
            max_entries: 0,
            ..wire::limits()
        },
        DropLimits {
            max_files: 0,
            ..wire::limits()
        },
        DropLimits {
            max_batches: 0,
            ..wire::limits()
        },
        DropLimits {
            max_calls: 0,
            ..wire::limits()
        },
    ] {
        let (data, mut source) = wire::parse(case.input);
        assert_eq!(
            wire::block_on(dropped_files(data.as_ref(), &mut source, limits)),
            Err(DropError::Limit)
        );
    }
    let (data, mut source) = wire::parse(case.input);
    source.infinite = true;
    assert_eq!(
        wire::block_on(dropped_files(
            data.as_ref(),
            &mut source,
            DropLimits {
                max_batches: 2,
                ..wire::limits()
            }
        )),
        Err(DropError::Limit)
    );
    assert_eq!(
        source
            .log
            .iter()
            .filter(|s| s.starts_with("batch:"))
            .count(),
        2
    );
    assert!(!source.log.iter().any(|s| s.starts_with("file:")));
}
#[test]
#[ignore = "live pinned original comparison requires Node22"]
fn pinned_drop_files_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/drop-files-diff")
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
        "contract: drop.files.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\nseeds: 1,7,99,1039\ncomparison: exact complete FLS1 classification, identity, links, names/bytes/order\n",
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
    assert_eq!(cases.len(), 40);
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
    println!("40 exact original states; raw {}", run.display());
}
