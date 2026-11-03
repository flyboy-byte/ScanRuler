// SPDX-License-Identifier: AGPL-3.0-only
//! Complete source callback Set mutation and wait traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    settle::SettleRun,
    settle_registry::{Callback, Limits, RegistryError, Settlers},
};
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
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/settle-registry/cases.tsv");
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
struct Job {
    ready: Rc<Cell<bool>>,
    done: bool,
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let ints = |s: &str| {
        s.split(',')
            .map(|n| n.parse::<usize>().unwrap())
            .collect::<Vec<_>>()
    };
    let initial = ints(p[1]);
    let modes = ints(p[2]);
    let actions = ints(p[3]);
    let targets = ints(p[4]);
    let registry = Settlers::new(Limits {
        registrations: 256,
        depth: 16,
        name_units: 256,
    });
    let callbacks: Rc<RefCell<Vec<Callback<'static, u16>>>> = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::new(RefCell::new(Vec::new()));
    let jobs: Rc<RefCell<Vec<Job>>> = Rc::new(RefCell::new(Vec::new()));
    let mutated: Rc<Vec<Cell<bool>>> = Rc::new((0..3).map(|_| Cell::new(false)).collect());
    for i in 0..3 {
        let mode = modes[i];
        let action = actions[i];
        let target = targets[i];
        let weak = registry.downgrade();
        let functions = Rc::downgrade(&callbacks);
        let log = log.clone();
        let jobs = jobs.clone();
        let mutated = mutated.clone();
        callbacks.borrow_mut().push(Rc::new(move |next| {
            log.borrow_mut().push(format!(
                "call:{i}:{}:{}",
                hex(&next.name),
                usize::from(next.read_only)
            ));
            if !mutated[i].replace(true) {
                let reg = weak.upgrade().unwrap();
                let functions = functions.upgrade().unwrap();
                let add = |j: usize| reg.register(functions.borrow()[j].clone()).unwrap();
                match action {
                    1 => add(i).unsubscribe(),
                    2 => add(target).unsubscribe(),
                    3 => {
                        drop(add(target));
                    }
                    4 => {
                        add(i).unsubscribe();
                        drop(add(i));
                    }
                    5 => {
                        add(target).unsubscribe();
                        drop(add(target));
                    }
                    6 => {
                        drop(add(target));
                        drop(add(target));
                    }
                    _ => {}
                }
            }
            if mode == 0 {
                return Ok(None);
            }
            if mode == 1 {
                return Err(0xdfff);
            }
            let flag = Rc::new(Cell::new(mode == 2 || mode == 3));
            if mode >= 4 {
                jobs.borrow_mut().push(Job {
                    ready: flag.clone(),
                    done: false,
                })
            }
            Ok(Some(Box::pin(Wait {
                ready: flag,
                failed: mode == 3 || mode == 5,
            })))
        }));
    }
    let mut subscriptions: Vec<_> = (0..3).map(|_| None).collect();
    for i in initial {
        subscriptions[i] = Some(registry.register(callbacks.borrow()[i].clone()).unwrap())
    }
    let mut frames = Vec::new();
    let mut status = Vec::new();
    let mut runs: Vec<Option<SettleRun<'static, u16>>> = Vec::new();
    let frame = |status: &Vec<&str>| format!("{}|{}|0", status.join(","), log.borrow().join(","));
    let flush = |status: &mut Vec<&str>, runs: &mut Vec<Option<SettleRun<'static, u16>>>| {
        let mut cx = Context::from_waker(Waker::noop());
        for (i, run) in runs.iter_mut().enumerate() {
            if let Some(run) = run
                && Pin::new(run).poll(&mut cx).is_ready()
            {
                status[i] = "done"
            }
        }
    };
    for round in 0..2 {
        let name: Vec<_> = if round == 0 {
            "mesh.read"
        } else {
            "mesh.write?"
        }
        .encode_utf16()
        .collect();
        let mut name = name;
        if round == 1 {
            *name.last_mut().unwrap() = 0xd800;
        }
        let run = registry.start(&name, round == 1).unwrap();
        status.push(if run.is_none() { "null" } else { "pending" });
        runs.push(run);
        frames.push(frame(&status));
        flush(&mut status, &mut runs);
        frames.push(frame(&status));
        let count = jobs.borrow().len();
        for j in (0..count).rev() {
            let ready = {
                let mut jobs = jobs.borrow_mut();
                if jobs[j].done {
                    continue;
                }
                jobs[j].done = true;
                jobs[j].ready.clone()
            };
            ready.set(true);
            log.borrow_mut().push(format!("finish:{j}"));
            frames.push(frame(&status));
            flush(&mut status, &mut runs);
            frames.push(frame(&status));
        }
        flush(&mut status, &mut runs);
        frames.push(frame(&status));
        assert_ne!(status[round], "pending");
        if round == 0 {
            if p[5] == "reregister0" {
                drop(registry.register(callbacks.borrow()[0].clone()).unwrap())
            }
            if p[5] != "none" {
                subscriptions[0].as_ref().unwrap().unsubscribe()
            }
            log.borrow_mut().push(format!("after:{}", p[5]));
            frames.push(frame(&status));
        }
    }
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn bounded_metadata_and_explicit_subscription_lifetime() {
    let registry = Settlers::new(Limits {
        registrations: 1,
        depth: 1,
        name_units: 1,
    });
    let calls = Rc::new(Cell::new(0));
    let c = calls.clone();
    let callback: Callback<'_, Vec<u16>> = Rc::new(move |_| {
        c.set(c.get() + 1);
        Err(vec![0xd800])
    });
    let old = registry.register(callback.clone()).unwrap();
    drop(registry.register(callback.clone()).unwrap());
    assert!(registry.start(&[65], false).unwrap().is_none());
    assert_eq!(calls.get(), 1);
    assert!(matches!(
        registry.start(&[65, 66], false),
        Err(RegistryError::Name)
    ));
    old.unsubscribe();
    assert!(registry.start(&[65], true).unwrap().is_none());
    assert_eq!(calls.get(), 1);
    assert!(matches!(
        registry.register(callback),
        Err(RegistryError::Registrations)
    ));
    let no_depth: Settlers<'_, ()> = Settlers::new(Limits {
        registrations: 0,
        depth: 0,
        name_units: 0,
    });
    assert!(matches!(
        no_depth.start(&[], false),
        Err(RegistryError::Depth)
    ));
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
fn borrowed_work_non_clone_errors_and_reentrant_depth() {
    #[derive(Debug)]
    struct Fault<'a>(&'a [u16]);
    let text = [0xd800];
    let registry = Settlers::new(Limits {
        registrations: 4,
        depth: 1,
        name_units: 16,
    });
    let weak = registry.downgrade();
    let callback: Callback<'_, Fault<'_>> = Rc::new(|_| {
        Ok(Some(Box::pin(async {
            let fault = Fault(&text);
            assert_eq!(fault.0, &[0xd800]);
            Err(fault)
        })))
    });
    drop(registry.register(callback).unwrap());
    let callback: Callback<'_, Fault<'_>> = Rc::new(move |_| {
        assert!(matches!(
            weak.upgrade().unwrap().start(&[], false),
            Err(RegistryError::Depth)
        ));
        Ok(None)
    });
    drop(registry.register(callback).unwrap());
    let mut run = registry.start(&[], false).unwrap().unwrap();
    let mut cx = Context::from_waker(Waker::noop());
    assert!(Pin::new(&mut run).poll(&mut cx).is_ready());
}
#[test]
#[ignore = "complete pinned live Set comparison requires Node22 and original graph"]
fn pinned_settle_registry_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-settle-registry.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/settle-registry.mjs"))
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
    assert_eq!(originals.len(), 116);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/settle-registry-diff/run-{run}"));
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
        "116 complete live mutation/identity/wait observations; {}",
        dir.display()
    );
}
