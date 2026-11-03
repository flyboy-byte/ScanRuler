// SPDX-License-Identifier: AGPL-3.0-only
//! Full settle traces over stable source callbacks and independent native futures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::settle::{Next, SettleError, Settler, settle};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    io::Write,
    path::Path,
    pin::Pin,
    process::{Command, Stdio},
    rc::Rc,
    task::{Context, Poll, Waker},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/settle-run/cases.tsv");
struct Wait {
    ready: Rc<Cell<bool>>,
    failed: bool,
}
impl Future for Wait {
    type Output = Result<(), u16>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready.get() {
            Poll::Ready(if self.failed { Err(0xd800) } else { Ok(()) })
        } else {
            Poll::Pending
        }
    }
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let modes: Vec<usize> = if p[1].is_empty() {
        vec![]
    } else {
        p[1].split(',').map(|n| n.parse().unwrap()).collect()
    };
    let name: Vec<_> = p[4]
        .as_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|h| u16::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect();
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut pending = Vec::new();
    let mut callbacks: Vec<Box<Settler<'_, u16>>> = Vec::new();
    for (i, &mode) in modes.iter().enumerate() {
        let log = log.clone();
        let flag = Rc::new(Cell::new(mode == 2 || mode == 3));
        if mode >= 4 {
            pending.push(flag.clone())
        }
        callbacks.push(Box::new(move |next| {
            log.borrow_mut().push(format!(
                "{i}:{}:{}",
                hex(next.name),
                usize::from(next.read_only)
            ));
            if mode == 0 {
                return Ok(None);
            }
            if mode == 1 {
                return Err(0xdfff);
            }
            Ok(Some(Box::pin(Wait {
                ready: flag.clone(),
                failed: mode == 3 || mode == 5,
            })))
        }));
    }
    let mut refs: Vec<_> = callbacks.iter_mut().map(|c| c.as_mut()).collect();
    let mut run = settle(
        Next {
            name: &name,
            read_only: p[2] == "1",
        },
        &mut refs,
        4096,
        4096,
    )
    .unwrap();
    let mut status = if run.is_none() { "null" } else { "pending" };
    let mut frames = Vec::new();
    let frame = |status: &str| format!("{status}|{}|0", log.borrow().join(","));
    let mut flush = |status: &mut &str| {
        if let Some(r) = &mut run {
            let mut cx = Context::from_waker(Waker::noop());
            if Pin::new(r).poll(&mut cx).is_ready() {
                *status = "done"
            }
        }
    };
    frames.push(frame(status));
    flush(&mut status);
    frames.push(frame(status));
    if p[3] == "reverse" {
        pending.reverse()
    }
    if p[3] == "all" {
        for flag in &pending {
            flag.set(true)
        }
        frames.push(frame(status));
        flush(&mut status);
        frames.push(frame(status));
    } else {
        for flag in &pending {
            flag.set(true);
            frames.push(frame(status));
            flush(&mut status);
            frames.push(frame(status));
        }
    }
    flush(&mut status);
    frames.push(frame(status));
    assert_ne!(status, "pending");
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn pre_admission_and_opaque_faults() {
    let name = [0xd800];
    let mut callback =
        |_: Next<'_>| -> Result<Option<port_engine::settle::Work<'_, Vec<u16>>>, Vec<u16>> {
            panic!("host limit before callback")
        };
    let mut refs: Vec<&mut Settler<'_, Vec<u16>>> = vec![&mut callback];
    assert!(matches!(
        settle(
            Next {
                name: &name,
                read_only: false
            },
            &mut refs,
            0,
            1
        ),
        Err(SettleError::Limit)
    ));
    assert!(matches!(
        settle(
            Next {
                name: &name,
                read_only: false
            },
            &mut refs,
            1,
            0
        ),
        Err(SettleError::Limit)
    ));
    let mut callback =
        |_: Next<'_>| -> Result<Option<port_engine::settle::Work<'_, Vec<u16>>>, Vec<u16>> {
            Err(vec![0xdfff])
        };
    let mut refs: Vec<&mut Settler<'_, Vec<u16>>> = vec![&mut callback];
    assert!(
        settle(
            Next {
                name: &name,
                read_only: true
            },
            &mut refs,
            1,
            1
        )
        .unwrap()
        .is_none()
    );
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "complete pinned settlement requires Node22 and original graph"]
fn pinned_settle_run_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-settle.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/settle-run.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let original = child.wait_with_output().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let output = String::from_utf8(original.stdout).unwrap();
    let originals: Vec<_> = output.split_inclusive('\n').collect();
    assert_eq!(originals.len(), 64);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/settle-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let candidate = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &candidate.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&candidate.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    for (input, expected) in INPUT.lines().zip(originals) {
        let id = input.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        let actual = observe(input);
        for (name, data) in [
            ("input.bin", input.as_bytes()),
            ("original.bin", expected.as_bytes()),
            ("rust.bin", actual.as_bytes()),
            ("original.stderr.txt", b"".as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert_eq!(actual, expected, "{id}");
    }
    println!(
        "64 complete concurrent wait/state observations; {}",
        dir.display()
    );
}
