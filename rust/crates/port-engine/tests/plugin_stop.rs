// SPDX-License-Identifier: AGPL-3.0-only
//! Complete source live stopping, full map/effect trace and concurrent waits.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    plugin_busy::{Check, Limits, Plugins, Release},
    plugin_stop::{Stop, StopError, StopRun},
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
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/plugin-stop/cases.tsv");
struct Wait {
    ready: Rc<Cell<bool>>,
    failed: bool,
}
impl Future for Wait {
    type Output = Result<(), ()>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready.get() {
            Poll::Ready(if self.failed { Err(()) } else { Ok(()) })
        } else {
            Poll::Pending
        }
    }
}
struct Job {
    ready: Rc<Cell<bool>>,
    done: bool,
}
type Registry = Plugins<'static, usize, Stop<'static, usize, ()>, ()>;
type Undo = Release<'static, usize, Stop<'static, usize, ()>, ()>;
struct World {
    registry: Registry,
    checks: RefCell<Vec<Check<'static, ()>>>,
    stops: RefCell<Vec<Rc<Stop<'static, usize, ()>>>>,
    releases: RefCell<Vec<Option<Undo>>>,
    stores: Vec<Rc<[Rc<usize>]>>,
    log: RefCell<Vec<String>>,
    jobs: RefCell<Vec<Job>>,
    done: Rc<usize>,
    mutated: Cell<bool>,
    action: usize,
}
impl World {
    fn install(&self, i: usize) {
        let check = self.checks.borrow()[i].clone();
        let stop = self.stops.borrow()[i].clone();
        let id = Rc::from(i.to_string().encode_utf16().collect::<Vec<_>>());
        let release = self
            .registry
            .register(
                id,
                check,
                self.stores[i].clone(),
                if self.action == 7 { None } else { Some(stop) },
            )
            .unwrap();
        self.releases.borrow_mut()[i] = Some(release)
    }
    fn release(&self, i: usize) {
        self.releases.borrow()[i].as_ref().unwrap().release()
    }
    fn mutate(&self) {
        if self.mutated.replace(true) {
            return;
        }
        match self.action {
            1 => self.release(1),
            2 => self.install(2),
            3 => {
                self.release(1);
                self.install(1)
            }
            4 => {
                self.release(0);
                self.install(0)
            }
            5 => self.release(0),
            6 => {
                self.release(0);
                let check = self.checks.borrow()[2].clone();
                let stop = self.stops.borrow()[2].clone();
                let release = self
                    .registry
                    .register(
                        Rc::from(vec![48]),
                        check,
                        self.stores[2].clone(),
                        Some(stop),
                    )
                    .unwrap();
                self.releases.borrow_mut()[0] = Some(release);
            }
            _ => {}
        }
    }
    fn frame(&self, status: &[&str]) -> String {
        let entries = self.registry.snapshot().unwrap();
        let ids = entries
            .iter()
            .map(|e| String::from_utf16(&e.id).unwrap())
            .collect::<Vec<_>>();
        let arrays = entries
            .iter()
            .zip(&ids)
            .map(|(e, id)| {
                format!(
                    "{id}:{}",
                    self.stores
                        .iter()
                        .position(|s| Rc::ptr_eq(s, &e.stores))
                        .unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let stops = self.stops.borrow();
        let functions = entries
            .iter()
            .zip(&ids)
            .filter_map(|(e, id)| {
                e.stop.as_ref().map(|stop| {
                    format!(
                        "{id}:{}",
                        stops.iter().position(|s| Rc::ptr_eq(s, stop)).unwrap()
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}|{}|{}|{arrays}|{functions}|0",
            status.join(","),
            self.log.borrow().join(","),
            ids.join(",")
        )
    }
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let n: Vec<usize> = p[1..].iter().map(|n| n.parse().unwrap()).collect();
    let [a, b, action, phase, s0, s1] = n[..] else {
        panic!()
    };
    let world = Rc::new(World {
        registry: Plugins::new(Limits {
            slots: 256,
            visits: 256,
            id_units: 10,
            depth: 16,
        }),
        checks: RefCell::new(Vec::new()),
        stops: RefCell::new(Vec::new()),
        releases: RefCell::new(vec![None, None, None]),
        stores: (0..3).map(|_| Rc::from(Vec::<Rc<usize>>::new())).collect(),
        log: RefCell::new(Vec::new()),
        jobs: RefCell::new(Vec::new()),
        done: Rc::new(777),
        mutated: Cell::new(false),
        action,
    });
    for i in 0..3 {
        let weak = Rc::downgrade(&world);
        world.checks.borrow_mut().push(Rc::new(move || {
            let w = weak.upgrade().unwrap();
            w.log.borrow_mut().push(format!("check:{i}"));
            if i == 0 && phase == 0 {
                w.mutate()
            }
            match if i == 0 {
                a
            } else if i == 1 {
                b
            } else {
                2
            } {
                0 => Ok(None),
                1 => Ok(Some(Rc::from(vec![]))),
                2 => Ok(Some(Rc::from("busy".encode_utf16().collect::<Vec<_>>()))),
                _ => Err(()),
            }
        }));
        let weak = Rc::downgrade(&world);
        world.stops.borrow_mut().push(Rc::new(move |done| {
            let w = weak.upgrade().unwrap();
            w.log.borrow_mut().push(format!(
                "stop:{i}:{}",
                usize::from(Rc::ptr_eq(&done, &w.done))
            ));
            if i == 0 && phase == 1 {
                w.mutate()
            }
            let n = if i == 0 {
                s0
            } else if i == 1 {
                s1
            } else {
                3
            };
            if n == 0 {
                return Err(());
            }
            let ready = Rc::new(Cell::new(n == 1 || n == 2));
            if n >= 3 {
                w.jobs.borrow_mut().push(Job {
                    ready: ready.clone(),
                    done: false,
                })
            }
            Ok(Box::pin(Wait {
                ready,
                failed: n == 2 || n == 4,
            }))
        }));
    }
    if action != 8 {
        world.install(0);
        world.install(1);
    }
    let mut frames = Vec::new();
    let mut status = Vec::new();
    let mut runs: Vec<StopRun<'_, ()>> = Vec::new();
    let flush = |status: &mut Vec<&str>, runs: &mut Vec<StopRun<'_, ()>>| {
        let mut cx = Context::from_waker(Waker::noop());
        for (i, run) in runs.iter_mut().enumerate() {
            if Pin::new(run).poll(&mut cx).is_ready() {
                status[i] = "done"
            }
        }
    };
    for round in 0..2 {
        runs.push(world.registry.stop_work(world.done.clone()).unwrap());
        status.push("pending");
        frames.push(world.frame(&status));
        flush(&mut status, &mut runs);
        frames.push(world.frame(&status));
        let count = world.jobs.borrow().len();
        for j in (0..count).rev() {
            let ready = {
                let mut jobs = world.jobs.borrow_mut();
                if jobs[j].done {
                    continue;
                }
                jobs[j].done = true;
                jobs[j].ready.clone()
            };
            ready.set(true);
            world.log.borrow_mut().push(format!("finish:{j}"));
            frames.push(world.frame(&status));
            flush(&mut status, &mut runs);
            frames.push(world.frame(&status));
        }
        flush(&mut status, &mut runs);
        frames.push(world.frame(&status));
        assert_eq!(status[round], "done");
    }
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn complete_independent_inputs_are_executable() {
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
#[test]
fn host_visit_refusal_retains_and_drains_started_wait() {
    let registry: Registry = Plugins::new(Limits {
        slots: 3,
        visits: 1,
        id_units: 3,
        depth: 1,
    });
    let ready = Rc::new(Cell::new(false));
    let flag = ready.clone();
    let stop: Rc<Stop<'_, usize, ()>> = Rc::new(move |_| {
        Ok(Box::pin(Wait {
            ready: flag.clone(),
            failed: true,
        }))
    });
    let check: Check<'_, ()> = Rc::new(|| Ok(Some(Rc::from(vec![1]))));
    for id in [vec![1], vec![2]] {
        drop(
            registry
                .register(
                    Rc::from(id),
                    check.clone(),
                    Rc::from(vec![]),
                    Some(stop.clone()),
                )
                .unwrap(),
        )
    }
    let mut run = registry.stop_work(Rc::new(0)).unwrap();
    let mut cx = Context::from_waker(Waker::noop());
    assert!(Pin::new(&mut run).poll(&mut cx).is_pending());
    ready.set(true);
    assert_eq!(
        Pin::new(&mut run).poll(&mut cx),
        Poll::Ready(Err(StopError::Visits))
    );
}
#[test]
#[ignore = "complete original live plugin stopping requires Node22 and original graph"]
fn pinned_plugin_stop_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-plugin-stop.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/plugin-stop.mjs"))
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
    assert_eq!(originals.len(), 227);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/plugin-stop-diff/run-{run}"));
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
        "227 complete live plugin start/wait observations; {}",
        dir.display()
    );
}
