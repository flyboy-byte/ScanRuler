// SPDX-License-Identifier: AGPL-3.0-only
//! Full source/native route, factory, caller and scheduling traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    import_queue::QueueError,
    scene_queue::{Route, SceneQueue},
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
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/scene-queue/cases.tsv");
struct Opaque;
struct Wait {
    ready: Rc<Cell<bool>>,
    value: Option<Result<Rc<Opaque>, Rc<Opaque>>>,
    token: Rc<Opaque>,
}
impl Future for Wait {
    type Output = Result<Rc<Opaque>, Rc<Opaque>>;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready.get() {
            Poll::Ready(self.value.take().unwrap())
        } else {
            Poll::Pending
        }
    }
}
type Run = Route<'static, Wait, Rc<Opaque>, Rc<Opaque>>;
struct World {
    queue: SceneQueue<Rc<Opaque>>,
    modes: [usize; 4],
    flags: Vec<Rc<Cell<bool>>>,
    created: Vec<Cell<bool>>,
    values: Vec<Rc<Opaque>>,
    faults: Vec<Rc<Opaque>>,
    tokens: Vec<Rc<Opaque>>,
    log: RefCell<Vec<String>>,
}
impl World {
    fn submit(
        self: &Rc<Self>,
        i: usize,
        scene: bool,
        states: &mut Vec<String>,
        identity: &mut Vec<String>,
        runs: &mut Vec<Option<Run>>,
    ) {
        let w = self.clone();
        let result = self.queue.run(scene, move || {
            w.log.borrow_mut().push(format!("start:{i}"));
            let mode = w.modes[i];
            if mode == 4 {
                return Err(w.faults[i].clone());
            }
            w.created[i].set(true);
            Ok(Wait {
                ready: w.flags[i].clone(),
                value: Some(if mode == 0 || mode == 2 {
                    Ok(w.values[i].clone())
                } else {
                    Err(w.faults[i].clone())
                }),
                token: w.tokens[i].clone(),
            })
        });
        match result {
            Ok(run) => {
                states.push("pending".into());
                identity.push(match &run {
                    Route::Direct(wait) => {
                        usize::from(Rc::ptr_eq(&wait.token, &self.tokens[i])).to_string()
                    }
                    Route::Queued(_) => "0".into(),
                });
                runs.push(Some(run));
            }
            Err(QueueError::Callback(e)) => {
                states.push(format!(
                    "throw:{}",
                    usize::from(Rc::ptr_eq(&e, &self.faults[i]))
                ));
                identity.push("-".into());
                runs.push(None)
            }
            _ => panic!("host refusal"),
        }
    }
    fn flush(&self, states: &mut [String], runs: &mut [Option<Run>]) {
        let mut cx = Context::from_waker(Waker::noop());
        for _ in 0..4 {
            for (i, slot) in runs.iter_mut().enumerate() {
                if let Some(run) = slot {
                    match Pin::new(run).poll(&mut cx) {
                        Poll::Pending => {}
                        Poll::Ready(Ok(v)) => {
                            states[i] =
                                format!("ok:{}", usize::from(Rc::ptr_eq(&v, &self.values[i])));
                            *slot = None;
                        }
                        Poll::Ready(Err(QueueError::Callback(e))) => {
                            states[i] =
                                format!("err:{}", usize::from(Rc::ptr_eq(&e, &self.faults[i])));
                            *slot = None;
                        }
                        _ => panic!("host error"),
                    }
                }
            }
        }
    }
    fn frame(&self, states: &[String], identity: &[String]) -> String {
        format!(
            "{}|{}|{}|0",
            states.join(","),
            identity.join(","),
            self.log.borrow().join(",")
        )
    }
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mask: usize = p[1].parse().unwrap();
    let pattern: usize = p[2].parse().unwrap();
    let order: usize = p[3].parse().unwrap();
    let modes = [pattern, (pattern + 1) % 5, (pattern + 3) % 5, 0];
    let make = || (0..4).map(|_| Rc::new(Opaque)).collect();
    let w = Rc::new(World {
        queue: SceneQueue::new(16),
        modes,
        flags: modes.iter().map(|m| Rc::new(Cell::new(*m <= 1))).collect(),
        created: (0..4).map(|_| Cell::new(false)).collect(),
        values: make(),
        faults: make(),
        tokens: make(),
        log: RefCell::new(Vec::new()),
    });
    let (mut states, mut identity, mut runs, mut frames) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..3 {
        w.submit(
            i,
            mask & (1 << i) != 0,
            &mut states,
            &mut identity,
            &mut runs,
        );
        frames.push(w.frame(&states, &identity));
    }
    w.flush(&mut states, &mut runs);
    frames.push(w.frame(&states, &identity));
    let orders = [[0, 1, 2], [2, 1, 0], [1, 2, 0]];
    let mut finished = [false; 4];
    for _ in 0..3 {
        for i in orders[order] {
            if w.created[i].get() && modes[i] >= 2 && modes[i] <= 3 && !finished[i] {
                finished[i] = true;
                w.flags[i].set(true);
                w.log.borrow_mut().push(format!("finish:{i}"));
                frames.push(w.frame(&states, &identity));
                w.flush(&mut states, &mut runs);
                frames.push(w.frame(&states, &identity));
            }
        }
    }
    w.flush(&mut states, &mut runs);
    frames.push(w.frame(&states, &identity));
    w.submit(3, true, &mut states, &mut identity, &mut runs);
    frames.push(w.frame(&states, &identity));
    w.flush(&mut states, &mut runs);
    frames.push(w.frame(&states, &identity));
    assert!(!states.iter().any(|s| s == "pending"));
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn complete_independent_schedules_execute() {
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
#[test]
fn direct_bypasses_full_or_abandoned_queue_and_retains_borrowed_nonclone() {
    struct NoClone<'a>(&'a str);
    let borrowed = String::from("borrowed");
    let queue = SceneQueue::new(0);
    let flag = Cell::new(false);
    let run = queue
        .run(false, || {
            flag.set(true);
            Ok::<_, ()>(std::future::ready(Ok::<_, ()>(NoClone(&borrowed))))
        })
        .unwrap();
    assert!(flag.get());
    let Route::Direct(mut direct) = run else {
        panic!("direct route changed")
    };
    let mut cx = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(value)) = Pin::new(&mut direct).poll(&mut cx) else {
        panic!("not ready")
    };
    assert_eq!(value.0.as_ptr(), borrowed.as_ptr());
    assert!(matches!(
        queue.run(true, || Ok(std::future::ready(Ok::<_, ()>(1)))),
        Err(QueueError::Limit)
    ));
    let queue = SceneQueue::new(1);
    let waiting = queue
        .run(true, || Ok(std::future::pending::<Result<(), ()>>()))
        .unwrap();
    drop(waiting);
    assert!(matches!(
        queue.run(true, || Ok(std::future::ready(Ok::<_, ()>(())))),
        Err(QueueError::Abandoned)
    ));
    assert!(
        queue
            .run(false, || Ok(std::future::ready(Ok::<_, ()>(()))))
            .is_ok()
    );
}
#[test]
#[ignore = "complete original scene queue requires Node22 and original graph"]
fn pinned_scene_queue_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-scene-queue.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/scene-queue.mjs"))
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
    assert_eq!(originals.len(), 120);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/scene-queue-diff/run-{run}"));
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
        "120 complete scene routing/factory/caller observations; {}",
        dir.display()
    );
}
