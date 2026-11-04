// SPDX-License-Identifier: AGPL-3.0-only
//! Complete execute orchestration using actual native plugin/settler providers.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    execute::{self, Body, Done, DoneWait, ExecuteError, Host},
    plugin_busy::{Limits as PluginLimits, Plugins},
    plugin_stop::Stop,
    reactions,
    settle::Work,
    settle_registry::{Limits as SettleLimits, Settlers},
    tasks::{Limits as TaskLimits, Owner},
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
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/execute/cases.tsv");
struct Fault {
    name: &'static str,
    code: &'static str,
    message: &'static str,
}
type E = Rc<Fault>;
fn fault(name: &'static str, code: &'static str, message: &'static str) -> E {
    Rc::new(Fault {
        name,
        code,
        message,
    })
}
struct Value;
struct Wait<T> {
    ready: Rc<Cell<bool>>,
    result: Option<Result<T, E>>,
    wake: Rc<RefCell<Option<Waker>>>,
}
impl<T: Unpin> Future for Wait<T> {
    type Output = Result<T, E>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready.get() {
            Poll::Ready(self.result.take().unwrap())
        } else {
            *self.wake.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
struct Job {
    ready: Rc<Cell<bool>>,
    wake: Rc<RefCell<Option<Waker>>>,
    done: bool,
    clear: bool,
}
struct Env {
    discard: usize,
    busy: usize,
    wait: usize,
    body: usize,
    cleared: Cell<bool>,
    log: RefCell<Vec<String>>,
    jobs: RefCell<Vec<Job>>,
    observers: RefCell<Vec<DoneWait>>,
    done_seen: Cell<usize>,
    plugins: Plugins<'static, usize, Stop<'static, Done, E>, E>,
    settlers: Settlers<'static, E>,
    value: Rc<Value>,
}
impl Env {
    fn guard(&self) -> Result<Option<Rc<[u16]>>, E> {
        self.log.borrow_mut().push("guard".into());
        if self.cleared.get() {
            return Ok(None);
        }
        match self.busy {
            0 => Ok(None),
            1 => Ok(Some(Rc::from(vec![]))),
            2 => Ok(Some(Rc::from("busy".encode_utf16().collect::<Vec<_>>()))),
            _ => Err(fault("Error", "", "guard")),
        }
    }
    fn wait(&self, stop: bool) -> Result<Option<Work<'static, E>>, E> {
        if !stop && self.wait == 5 {
            return Ok(None);
        }
        if self.wait == 4 {
            return Err(fault("Error", "", "wait"));
        }
        let ready = Rc::new(Cell::new(
            self.wait == 0 || self.wait == 1 || self.wait == 5,
        ));
        let wake = Rc::new(RefCell::new(None));
        if self.wait == 0 || self.wait == 5 {
            self.cleared.set(true)
        }
        if self.wait == 2 || self.wait == 3 {
            self.jobs.borrow_mut().push(Job {
                ready: ready.clone(),
                wake: wake.clone(),
                done: false,
                clear: self.wait == 2,
            })
        }
        Ok(Some(Box::pin(Wait {
            ready,
            wake,
            result: Some(if self.wait == 1 || self.wait == 3 {
                Err(fault("Error", "", "wait"))
            } else {
                Ok(())
            }),
        })))
    }
    fn observe_done(&self, cx: &mut Context<'_>) {
        let mut observers = self.observers.borrow_mut();
        let mut i = 0;
        while i < observers.len() {
            if Pin::new(&mut observers[i]).poll(cx).is_ready() {
                drop(observers.remove(i));
                self.done_seen.set(self.done_seen.get() + 1);
                self.log.borrow_mut().push("done".into());
            } else {
                i += 1
            }
        }
    }
    fn frame(&self, state: &str) -> String {
        format!(
            "{state}|{}|{}|0",
            self.done_seen.get(),
            self.log.borrow().join(",")
        )
    }
}
struct Provider(Rc<Env>);
impl Host<'static> for Provider {
    type Value = Rc<Value>;
    type Error = E;
    fn discards(&mut self) -> Result<bool, E> {
        if self.0.discard == 0 {
            return Ok(false);
        }
        self.0.log.borrow_mut().push("discards".into());
        if self.0.discard == 4 {
            Err(fault("Error", "", "discard"))
        } else {
            Ok(self.0.discard == 2)
        }
    }
    fn busy(&mut self) -> Result<Option<Rc<[u16]>>, E> {
        self.0.guard()
    }
    fn stop(&mut self, done: Done) -> Work<'static, E> {
        let run = self.0.plugins.stop_work(Rc::new(done)).unwrap();
        Box::pin(async move { run.await.map_err(|_| fault("HostError", "", "stop limit")) })
    }
    fn settle(&mut self) -> Result<Work<'static, E>, E> {
        let run = self
            .0
            .settlers
            .start(&"unit".encode_utf16().collect::<Vec<_>>(), false)
            .unwrap();
        Ok(Box::pin(async move {
            if let Some(run) = run {
                run.await
            }
            Ok(())
        }))
    }
    // A separately owned guarded-provider adapter. This comparator profile uses
    // the original history=false branch; its full activity/history/deadline
    // implementation is not claimed by the execute orchestration unit.
    fn guarded(&mut self, deferred: bool) -> Body<'static, Rc<Value>, E> {
        match self.0.guard() {
            Err(e) => return Box::pin(std::future::ready(Err(e))),
            Ok(Some(busy)) if !busy.is_empty() => {
                return Box::pin(std::future::ready(Err(fault(
                    "CommandError",
                    "busy",
                    "The session is busy — busy. Try again once it has finished.",
                ))));
            }
            _ => {}
        }
        self.0
            .log
            .borrow_mut()
            .push(format!("body:{}", usize::from(deferred)));
        let mode = self.0.body;
        let ready = Rc::new(Cell::new(mode <= 1 || mode == 4));
        let wake = Rc::new(RefCell::new(None));
        if mode == 2 || mode == 3 {
            self.0.jobs.borrow_mut().push(Job {
                ready: ready.clone(),
                wake: wake.clone(),
                done: false,
                clear: false,
            })
        }
        Box::pin(Wait {
            ready,
            wake,
            result: Some(if mode == 0 || mode == 2 {
                Ok(self.0.value.clone())
            } else {
                Err(fault("CommandError", "failed", "body"))
            }),
        })
    }
}
fn hex(s: &str) -> String {
    s.encode_utf16().map(|v| format!("{v:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let n: Vec<usize> = p[1..].iter().map(|s| s.parse().unwrap()).collect();
    let [discard, busy, deferred, wait, body] = n[..] else {
        panic!()
    };
    let env = Rc::new(Env {
        discard,
        busy,
        wait,
        body,
        cleared: Cell::new(false),
        log: RefCell::new(Vec::new()),
        jobs: RefCell::new(Vec::new()),
        observers: RefCell::new(Vec::new()),
        done_seen: Cell::new(0),
        plugins: Plugins::new(PluginLimits {
            slots: 32,
            visits: 32,
            id_units: 32,
            depth: 16,
        }),
        settlers: Settlers::new(SettleLimits {
            registrations: 32,
            depth: 16,
            name_units: 32,
        }),
        value: Rc::new(Value),
    });
    let weak = Rc::downgrade(&env);
    let check = Rc::new(move || {
        let e = weak.upgrade().unwrap();
        e.log.borrow_mut().push("plugin-check".into());
        Ok(Some(Rc::from("plugin".encode_utf16().collect::<Vec<_>>())))
    });
    let weak = Rc::downgrade(&env);
    let stop: Rc<Stop<'static, Done, E>> = Rc::new(move |done| {
        let e = weak.upgrade().unwrap();
        e.log.borrow_mut().push("stop".into());
        e.observers.borrow_mut().push(done.wait().unwrap());
        Ok(e.wait(true)?.unwrap())
    });
    drop(
        env.plugins
            .register(
                Rc::from("plugin".encode_utf16().collect::<Vec<_>>()),
                check,
                Rc::from(vec![]),
                Some(stop),
            )
            .unwrap(),
    );
    let weak = Rc::downgrade(&env);
    drop(
        env.settlers
            .register(Rc::new(move |_| {
                let e = weak.upgrade().unwrap();
                e.log.borrow_mut().push("settle".into());
                e.wait(false)
            }))
            .unwrap(),
    );
    let owner = Owner::new(TaskLimits {
        tasks: 8,
        depth: 8,
        polls_per_turn: 64,
    })
    .unwrap();
    let state = Rc::new(RefCell::new(String::from("pending")));
    let receiver = state.clone();
    let host = env.clone();
    let output_env = env.clone();
    // Drop the ticket before the first asynchronous continuation. Owner still drains execute.
    drop(
        reactions::retain(
            &owner,
            move || execute::execute(Provider(host), deferred != 0, 32),
            move |result| {
                *receiver.borrow_mut() = match result {
                    Ok(value) => {
                        format!("ok:{}", usize::from(Rc::ptr_eq(&value, &output_env.value)))
                    }
                    Err(ExecuteError::Callback(e)) => {
                        format!("err:{}:{}:{}", hex(e.name), hex(e.code), hex(e.message))
                    }
                    _ => panic!("host error"),
                };
            },
        )
        .unwrap(),
    );
    let mut frames = vec![env.frame(&state.borrow())];
    let mut cx = Context::from_waker(Waker::noop());
    let mut flush = || {
        let turn = owner.turn().unwrap();
        assert_eq!(
            turn.ready, 0,
            "fixture must reach reaction quiescence within its admitted turn"
        );
        env.observe_done(&mut cx);
    };
    flush();
    frames.push(env.frame(&state.borrow()));
    for _ in 0..3 {
        let mut j = 0;
        while j < env.jobs.borrow().len() {
            let job = {
                let mut jobs = env.jobs.borrow_mut();
                if jobs[j].done {
                    None
                } else {
                    jobs[j].done = true;
                    Some((jobs[j].ready.clone(), jobs[j].clear, jobs[j].wake.clone()))
                }
            };
            if let Some((ready, clear, wake)) = job {
                if clear {
                    env.cleared.set(true)
                }
                ready.set(true);
                if let Some(waker) = wake.borrow_mut().take() {
                    waker.wake();
                }
                env.log.borrow_mut().push(format!("finish:{j}"));
                frames.push(env.frame(&state.borrow()));
                flush();
                frames.push(env.frame(&state.borrow()));
            }
            j += 1;
        }
    }
    flush();
    frames.push(env.frame(&state.borrow()));
    assert_ne!(*state.borrow(), "pending");
    owner.finish().unwrap();
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn independent_matrix_executes() {
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
struct Borrowed<'a>(&'a str);
struct Local<'a> {
    value: &'a str,
    flag: Rc<Cell<bool>>,
    preflight_error: bool,
}
impl<'a> Host<'a> for Local<'a> {
    type Value = Borrowed<'a>;
    type Error = ();
    fn discards(&mut self) -> Result<bool, ()> {
        if self.preflight_error {
            Err(())
        } else {
            Ok(true)
        }
    }
    fn busy(&mut self) -> Result<Option<Rc<[u16]>>, ()> {
        Ok(Some(Rc::from(vec![])))
    }
    fn stop(&mut self, _: Done) -> Work<'a, ()> {
        let flag = self.flag.clone();
        Box::pin(std::future::poll_fn(move |_| {
            if flag.get() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }))
    }
    fn settle(&mut self) -> Result<Work<'a, ()>, ()> {
        panic!("unexpected settle")
    }
    fn guarded(&mut self, _: bool) -> Body<'a, Borrowed<'a>, ()> {
        Box::pin(std::future::ready(Ok(Borrowed(self.value))))
    }
}
#[test]
fn done_wakes_each_observer_once_and_preserves_borrowed_nonclone_values() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Count(AtomicUsize);
    impl std::task::Wake for Count {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let value = String::from("borrowed");
    let flag = Rc::new(Cell::new(false));
    let mut run = execute::execute(
        Local {
            value: &value,
            flag: flag.clone(),
            preflight_error: false,
        },
        false,
        2,
    );
    let done = run.done().unwrap();
    assert!(done == run.done().unwrap());
    assert!(!done.resolved());
    let mut first = done.wait().unwrap();
    let mut second = done.wait().unwrap();
    assert!(matches!(done.wait(), Err(execute::DoneError::Waiters)));
    let a = Arc::new(Count(AtomicUsize::new(0)));
    let b = Arc::new(Count(AtomicUsize::new(0)));
    let wa = Waker::from(a.clone());
    let wb = Waker::from(b.clone());
    assert!(
        Pin::new(&mut first)
            .poll(&mut Context::from_waker(&wa))
            .is_pending()
    );
    assert!(
        Pin::new(&mut second)
            .poll(&mut Context::from_waker(&wb))
            .is_pending()
    );
    let mut cx = Context::from_waker(Waker::noop());
    assert!(Pin::new(&mut run).poll(&mut cx).is_pending());
    flag.set(true);
    let Poll::Ready(Ok(result)) = Pin::new(&mut run).poll(&mut cx) else {
        panic!("unfinished")
    };
    assert_eq!(result.0.as_ptr(), value.as_ptr());
    assert!(done.resolved());
    assert_eq!(a.0.load(Ordering::SeqCst), 1);
    assert_eq!(b.0.load(Ordering::SeqCst), 1);
    assert!(Pin::new(&mut first).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut second).poll(&mut cx).is_ready());
    let mut late = done.wait().unwrap();
    assert!(Pin::new(&mut late).poll(&mut cx).is_ready());
    assert!(matches!(
        Pin::new(&mut run).poll(&mut cx),
        Poll::Ready(Err(ExecuteError::Completed))
    ));
    assert_eq!(a.0.load(Ordering::SeqCst), 1);
}
#[test]
fn preflight_failure_has_no_done_and_dropping_observers_or_run_does_not_resolve() {
    let flag = Rc::new(Cell::new(false));
    let mut failed = execute::execute(
        Local {
            value: "value",
            flag: flag.clone(),
            preflight_error: true,
        },
        false,
        1,
    );
    assert!(failed.done().is_none());
    let mut cx = Context::from_waker(Waker::noop());
    assert!(matches!(
        Pin::new(&mut failed).poll(&mut cx),
        Poll::Ready(Err(ExecuteError::Callback(())))
    ));
    let run = execute::execute(
        Local {
            value: "value",
            flag,
            preflight_error: false,
        },
        false,
        1,
    );
    let done = run.done().unwrap();
    drop(done.wait().unwrap());
    assert!(!done.resolved());
    assert!(matches!(done.wait(), Err(execute::DoneError::Waiters)));
    drop(run);
    assert!(!done.resolved());
}

#[test]
#[ignore = "complete original execute lifecycle requires Node22 and original graph"]
fn pinned_execute_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-execute.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/execute.mjs"))
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
    assert_eq!(originals.len(), 1200);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/execute-diff/run-{run}"));
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
        if id == INPUT.lines().next().unwrap().split('\t').next().unwrap() {
            let mut wrong = actual.clone().into_bytes();
            *wrong.last_mut().unwrap() ^= 1;
            assert_ne!(
                wrong,
                expected.as_bytes(),
                "wrong execution outcome must fail"
            );
            assert_ne!(
                &actual.as_bytes()[..actual.len() - 1],
                expected.as_bytes(),
                "missing execution results must fail"
            );
        }
    }
    println!(
        "1200 complete execute preflight/wait/done observations; {}",
        dir.display()
    );
}
