// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original guard-object, busy-return and cleanup state traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::activity_guard::{Busy, Guard, Guards, Reset};
use std::{
    cell::{Cell, RefCell},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/activity-guard/cases.tsv");
struct Fault(Vec<u16>);
type ProbeGuard = Rc<Guard<'static, Rc<Fault>>>;
type ProbeReset = Option<Reset<'static, Rc<Fault>>>;
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let modes: Vec<usize> = p[1].split(',').map(|n| n.parse().unwrap()).collect();
    let action: usize = p[2].parse().unwrap();
    let text: Vec<_> = p[3]
        .as_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|h| u16::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
        .collect();
    let slot = Guards::new();
    let free = slot.current();
    let log = Rc::new(RefCell::new(Vec::new()));
    let guards: Rc<RefCell<Vec<ProbeGuard>>> = Rc::new(RefCell::new(Vec::new()));
    let resets: Rc<RefCell<Vec<ProbeReset>>> = Rc::new(RefCell::new(Vec::new()));
    let mut fault_text: Vec<_> = "fault:".encode_utf16().collect();
    fault_text.extend(&text);
    let fault = Rc::new(Fault(fault_text));
    let mut functions: Vec<Busy<'static, Rc<Fault>>> = Vec::new();
    for mode in 0..4 {
        let weak = slot.downgrade();
        let guards = Rc::downgrade(&guards);
        let resets = Rc::downgrade(&resets);
        let log = log.clone();
        let fault = fault.clone();
        let text = text.clone();
        let mutated = Cell::new(false);
        functions.push(Rc::new(move || {
            log.borrow_mut().push(format!("busy:{mode}"));
            if !mutated.replace(true) {
                let slot = weak.upgrade().unwrap();
                if action == 1 {
                    drop(slot.configure(guards.upgrade().unwrap().borrow()[(mode + 1) % 3].clone()))
                } else if action == 2
                    && let Some(reset) = resets
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .first()
                        .and_then(|r| r.as_ref())
                {
                    reset.reset()
                }
            }
            match mode {
                0 => Ok(None),
                1 => Ok(Some(vec![])),
                2 => Ok(Some(text.clone())),
                _ => Err(fault.clone()),
            }
        }));
    }
    for mode in modes {
        guards
            .borrow_mut()
            .push(Rc::new(Guard::new(functions[mode].clone())))
    }
    let mut frames = Vec::new();
    for op in [
        "get", "c0", "busy", "c1", "busy", "r0", "get", "c2", "busy", "drop1", "get", "c0", "r2",
        "busy", "r0", "get",
    ] {
        let mut result = "-".to_string();
        if let Some(i) = op.strip_prefix('c') {
            let next = guards.borrow()[i.parse::<usize>().unwrap()].clone();
            resets.borrow_mut().push(Some(slot.configure(next)))
        } else if let Some(i) = op.strip_prefix('r') {
            if let Some(reset) = &resets.borrow()[i.parse::<usize>().unwrap()] {
                reset.reset()
            }
        } else if op == "drop1" {
            resets.borrow_mut()[1] = None
        } else if op == "busy" {
            result = match slot.current().busy() {
                Ok(None) => "null".to_string(),
                Ok(Some(text)) => format!("text:{}", hex(&text)),
                Err(e) => format!(
                    "throw:Error:{}:{}",
                    hex(&e.0),
                    usize::from(Rc::ptr_eq(&e, &fault))
                ),
            }
        }
        let current = slot.current();
        let active = if Rc::ptr_eq(&current, &free) {
            0
        } else {
            guards
                .borrow()
                .iter()
                .position(|g| Rc::ptr_eq(g, &current))
                .unwrap()
                + 1
        };
        frames.push(format!("{active}|{result}|{}", log.borrow().join(",")));
    }
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn borrowed_non_clone_fault_and_unconditional_old_reset() {
    #[derive(Debug, PartialEq, Eq)]
    struct Borrowed<'a>(&'a [u16]);
    let data = [0xd800];
    let slot = Guards::new();
    let free = slot.current();
    let callback: Busy<'_, Borrowed<'_>> = Rc::new(|| Err(Borrowed(&data)));
    let first = Rc::new(Guard::new(callback.clone()));
    let second = Rc::new(Guard::new(callback));
    assert!(!Rc::ptr_eq(&first, &second));
    let old = slot.configure(first.clone());
    drop(slot.configure(second.clone()));
    assert!(Rc::ptr_eq(&slot.current(), &second));
    assert_eq!(slot.current().busy(), Err(Borrowed(&data)));
    old.reset();
    assert!(Rc::ptr_eq(&slot.current(), &free));
    assert_eq!(slot.current().busy(), Ok(None));
    drop(slot.configure(first.clone()));
    assert!(Rc::ptr_eq(&slot.current(), &first));
    old.reset();
    assert!(Rc::ptr_eq(&slot.current(), &free));
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "full original guard module requires Node22 and original graph"]
fn pinned_activity_guard_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-guard.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/activity-guard.mjs"))
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
    assert_eq!(originals.len(), 192);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/guard-diff/run-{run}"));
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
        "192 complete guard identity/callback/reset observations; {}",
        dir.display()
    );
}
