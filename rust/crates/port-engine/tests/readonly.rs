// SPDX-License-Identifier: AGPL-3.0-only
//! Actual native registered read-only/job dispatch; no original output is a native input.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    guarded::{self, Callbacks, Context, Fault, Host, Runtime, SubmitError},
    history_record::Snapshot as HistorySnapshot,
    recent_store::{ResultValue, Snapshot, State},
    schema_value::Value as V,
    tasks::{Limits as TaskLimits, Owner},
};
use std::{
    cell::{Cell, RefCell},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
    sync::Arc,
    task::{Poll, Waker},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/readonly/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn obj(fields: Vec<(&str, V)>) -> V {
    V::Object(fields.into_iter().map(|(k, v)| (text(k), v)).collect())
}
fn wire(value: &V) -> String {
    fn count(out: &mut Vec<u8>, n: usize) {
        out.extend_from_slice(&u32::try_from(n).unwrap().to_le_bytes())
    }
    fn txt(out: &mut Vec<u8>, s: &[u16]) {
        count(out, s.len());
        for c in s {
            out.extend_from_slice(&c.to_le_bytes())
        }
    }
    fn walk(out: &mut Vec<u8>, v: &V) {
        match v {
            V::Undefined => out.push(0),
            V::Null => out.push(1),
            V::Boolean(b) => out.push(if *b { 3 } else { 2 }),
            V::Number(n) => {
                out.push(4);
                out.extend_from_slice(&n.to_bits().to_le_bytes())
            }
            V::String(s) => {
                out.push(5);
                txt(out, s)
            }
            V::Array(items) => {
                out.push(6);
                count(out, items.len());
                for v in items {
                    walk(out, v)
                }
            }
            V::Object(fields) => {
                out.push(7);
                count(out, fields.len());
                for (k, v) in fields {
                    txt(out, k);
                    walk(out, v)
                }
            }
            V::Bytes(b) => {
                out.push(8);
                count(out, b.len());
                out.extend_from_slice(b)
            }
        }
    }
    let mut out = Vec::new();
    walk(&mut out, value);
    out.iter().map(|n| format!("{n:02x}")).collect()
}

fn snapshot_value(s: &Snapshot) -> V {
    let mut fields = vec![
        (text("job"), V::String(s.job.to_vec())),
        (text("name"), V::String(s.name.to_vec())),
        (
            text("state"),
            V::String(text(match s.state {
                State::Running => "running",
                State::Done => "done",
                State::Failed => "failed",
            })),
        ),
        (text("started"), V::String(s.started.to_vec())),
        (text("ms"), s.ms.map_or(V::Null, V::Number)),
    ];
    if let Some(result) = &s.result {
        fields.push((
            text("result"),
            match result {
                ResultValue::Primitive(v) => v.clone(),
                ResultValue::Array(items) => V::Array(items.borrow().clone()),
                ResultValue::Object(fields) => V::Object(fields.borrow().clone()),
            },
        ))
    }
    if let Some(message) = &s.result_omitted {
        fields.push((text("resultOmitted"), V::String(message.to_vec())))
    }
    if let Some(error) = &s.error {
        let error = error.borrow();
        fields.push((
            text("error"),
            V::Object(vec![
                (text("code"), V::String(error.code.to_vec())),
                (text("message"), V::String(error.message.to_vec())),
            ]),
        ));
    }
    V::Object(fields)
}

struct Timer {
    callback: Rc<dyn Fn() -> Result<(), Fault>>,
    active: bool,
}
struct Env {
    settlers: port_engine::settle_registry::Settlers<'static, Fault>,
    extra: usize,
    settle_mode: usize,
    body_mode: usize,
    busy: usize,
    now: Cell<f64>,
    doc: Cell<usize>,
    log: RefCell<Vec<V>>,
    timers: RefCell<Vec<Timer>>,
    released: Cell<bool>,
    waiters: RefCell<Vec<Waker>>,
    bodies: RefCell<Vec<Context>>,
    blocks: RefCell<Vec<Context>>,
    reason: RefCell<Option<Arc<port_engine::error::CommandError>>>,
    typed: Arc<port_engine::error::CommandError>,
    caller: RefCell<V>,
    unhandled: RefCell<Vec<V>>,
    weak: RefCell<Option<std::rc::Weak<Env>>>,
}
impl Env {
    fn log(&self, s: impl AsRef<str>) {
        self.log.borrow_mut().push(V::String(text(s.as_ref())))
    }
    fn error(&self, e: &Fault) -> Vec<V> {
        let (name, code, message, identity, reason) = match e {
            Fault::Command(e) => (
                text(e.name()),
                V::String(text(e.code().as_str())),
                e.message_units().to_vec(),
                Arc::ptr_eq(e, &self.typed),
                self.reason
                    .borrow()
                    .as_ref()
                    .is_some_and(|r| Arc::ptr_eq(r, e)),
            ),
            Fault::Error { name, message } => {
                (name.clone(), V::Undefined, message.clone(), false, false)
            }
            e => panic!("unexpected host error {e:?}"),
        };
        vec![
            V::String(name),
            code,
            V::String(message),
            V::Boolean(identity),
            V::Boolean(reason),
        ]
    }
    fn settling(
        &self,
        name: &[u16],
        read_only: bool,
    ) -> Option<port_engine::settle::Work<'static, Fault>> {
        self.settlers.start(name, read_only).unwrap().map(|run| {
            Box::pin(async move {
                run.await;
                Ok(())
            }) as port_engine::settle::Work<'static, Fault>
        })
    }
    fn fire(&self) {
        let calls = {
            let mut timers = self.timers.borrow_mut();
            timers
                .iter_mut()
                .filter_map(|t| {
                    if t.active {
                        t.active = false;
                        Some(t.callback.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        for f in calls {
            self.log("timer:fire");
            f().unwrap();
        }
    }
    fn release(&self) {
        self.released.set(true);
        for w in self.waiters.borrow_mut().drain(..) {
            w.wake()
        }
    }
    fn run(self: &Rc<Self>, ctx: Context) -> Result<(), Fault> {
        self.log(format!(
            "body:{}",
            usize::from(ctx.deferred() == Some(true))
        ));
        let env = self.clone();
        ctx.on_abort(Rc::new(move |ctx| {
            *env.reason.borrow_mut() = ctx.abort_reason();
            env.log("abort")
        }))?;
        self.doc.set(
            self.doc.get()
                + if std::env::var_os("SCANRULER_READONLY_SABOTAGE").is_some() {
                    9
                } else {
                    1
                },
        );
        let mut first = text("first ");
        first.push(0xd800);
        ctx.progress(&first)?;
        match self.body_mode {
            1 => {
                ctx.complete(Err(Fault::error("body failed")));
            }
            2 => return Err(Fault::Command(self.typed.clone())),
            3 | 4 => self.bodies.borrow_mut().push(ctx),
            9 => {
                ctx.complete(Err(Fault::error(
                    "Failed to fetch dynamically imported module: fixture",
                )));
            }
            mode => {
                ctx.complete(Ok(match mode {
                    5 => V::Undefined,
                    7 => V::Bytes(vec![0; 23]),
                    8 => V::String(vec![120; 16000]),
                    11 => V::Number(self.doc.get() as f64),
                    _ => V::String(text("ok")),
                }));
            }
        };
        Ok(())
    }
    fn frame(&self, runtime: &Runtime) -> String {
        let a = runtime.activity().current();
        let running = a.running.as_ref().map_or(V::Null, |r| {
            obj(vec![
                ("name", V::String(r.name.to_vec())),
                ("title", V::String(r.title.to_vec())),
                ("since", V::Number(r.since)),
            ])
        });
        let current = runtime.current().map_or(V::Null, |r| {
            obj(vec![
                ("name", V::String(r.name.to_vec())),
                ("deferred", V::Boolean(r.deferred)),
            ])
        });
        let history = runtime.history();
        let recent = |job: Option<&[u16]>| {
            V::Array(
                runtime
                    .recent_outcomes(job)
                    .unwrap()
                    .iter()
                    .map(snapshot_value)
                    .collect(),
            )
        };
        wire(&V::Array(vec![
            self.caller.borrow().clone(),
            V::Number(self.doc.get() as f64),
            V::Number(0.),
            running,
            a.progress
                .as_ref()
                .map_or(V::Null, |p| V::String(p.to_vec())),
            current,
            V::Number(history.past.len() as f64),
            V::Number(history.future.len() as f64),
            V::Number(history.revision as f64),
            V::Boolean(runtime.history_paused()),
            V::Number(self.timers.borrow().iter().filter(|t| t.active).count() as f64),
            V::Array(self.log.borrow().clone()),
            recent(None),
            recent(Some(&[])),
            recent(Some(&text("job"))),
            runtime.page_stale().map_or(V::Null, V::String),
            V::Array(self.unhandled.borrow().clone()),
        ]))
    }
}
impl Host for Env {
    fn busy(&self) -> Result<Option<Vec<u16>>, Fault> {
        self.log("busy");
        match self.busy {
            0 => Ok(None),
            1 => Ok(Some(text("working"))),
            _ => Err(Fault::error("busy fault")),
        }
    }
    fn workspace(&self) -> Result<Vec<u16>, Fault> {
        Ok(text("elements"))
    }
    fn capture(&self) -> Result<HistorySnapshot, Fault> {
        panic!("history:false/read-only must not capture")
    }
    fn stale_now(&self) -> Result<f64, Fault> {
        Ok(self.now.get())
    }
    fn now(&self) -> Result<f64, Fault> {
        if self.extra == 3 || self.extra == 5 && self.now.get() != 1234. {
            Err(Fault::error("clock"))
        } else if self.extra == 4 {
            Ok(f64::NAN)
        } else {
            Ok(self.now.get())
        }
    }
    fn timer(&self, ms: f64, callback: Rc<dyn Fn() -> Result<(), Fault>>) -> u64 {
        assert_eq!(ms, 10.);
        self.log("timer:set");
        let mut t = self.timers.borrow_mut();
        t.push(Timer {
            callback,
            active: true,
        });
        t.len() as u64
    }
    fn clear_timer(&self, id: u64) {
        self.timers.borrow_mut()[id as usize - 1].active = false;
        self.log("timer:clear")
    }
    fn settle(&self, name: &[u16]) -> Result<port_engine::settle::Work<'static, Fault>, Fault> {
        Ok(self
            .settling(name, false)
            .unwrap_or_else(|| Box::pin(std::future::ready(Ok(())))))
    }
    fn settle_readonly(
        &self,
        name: &[u16],
    ) -> Result<Option<port_engine::settle::Work<'static, Fault>>, Fault> {
        Ok(self.settling(name, true))
    }
    fn observer_fault(&self, error: Fault) {
        self.unhandled
            .borrow_mut()
            .push(V::Array(self.error(&error)))
    }
    fn stop(&self, _: port_engine::execute::Done) -> port_engine::settle::Work<'static, Fault> {
        panic!("no discards")
    }
}
fn observe(line: &str) -> String {
    let mut columns = line.split('\t');
    let id = columns.next().unwrap();
    let f = columns
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let [route, deferred, settle_mode, body_mode, job_mode, busy] = f[..6] else {
        panic!("fields")
    };
    let extra = f.get(6).copied().unwrap_or(0);
    let env = Rc::new(Env {
        settlers: port_engine::settle_registry::Settlers::new(
            port_engine::settle_registry::Limits {
                registrations: 8,
                depth: 8,
                name_units: 1000,
            },
        ),
        extra,
        settle_mode,
        body_mode,
        busy,
        now: Cell::new(1234.),
        doc: Cell::new(0),
        log: RefCell::new(vec![]),
        timers: RefCell::new(vec![]),
        released: Cell::new(false),
        waiters: RefCell::new(vec![]),
        bodies: RefCell::new(vec![]),
        blocks: RefCell::new(vec![]),
        reason: RefCell::new(None),
        typed: Arc::new(
            port_engine::error::CommandError::from_utf16(
                port_engine::error::CommandErrorCode::InvalidState,
                &{
                    let mut s = text("body failed ");
                    s.push(0xd800);
                    s
                },
                1000,
            )
            .unwrap(),
        ),
        caller: RefCell::new(V::String(text("pending"))),
        unhandled: RefCell::new(vec![]),
        weak: RefCell::new(None),
    });
    *env.weak.borrow_mut() = Some(Rc::downgrade(&env));
    let callback_env = Rc::downgrade(&env);
    env.settlers
        .register(Rc::new(move |next| {
            let env = callback_env.upgrade().unwrap();
            env.log(format!("settle:{}", usize::from(next.read_only)));
            match env.settle_mode {
                0 => Ok(None),
                1 => Ok(Some(Box::pin(std::future::ready(Ok(()))))),
                3 => Err(Fault::error("ignored settle")),
                _ => Ok(Some(Box::pin(std::future::poll_fn(move |cx| {
                    if env.released.get() {
                        Poll::Ready(if env.settle_mode == 4 {
                            Err(Fault::error("ignored async settle"))
                        } else {
                            Ok(())
                        })
                    } else {
                        env.waiters.borrow_mut().push(cx.waker().clone());
                        Poll::Pending
                    }
                })))),
            }
        }))
        .unwrap();
    let owner = Owner::new(TaskLimits {
        tasks: 1024,
        depth: 32,
        polls_per_turn: 10000,
    })
    .unwrap();
    let runtime = Runtime::new(owner.clone(), env.clone(), guarded::Limits::default());
    let watcher = env.clone();
    runtime
        .activity()
        .subscribe(Rc::new(move |_, s, _| {
            watcher.log(if s.running.is_some() {
                "activity:start"
            } else {
                "activity:clear"
            });
            Ok(())
        }))
        .unwrap();
    let body = env.clone();
    runtime
        .register(
            &text("unit"),
            obj(vec![
                ("name", V::String(text("run"))),
                ("title", V::String(text("Read"))),
                ("description", V::String(text("fixture"))),
                (
                    "input",
                    obj(vec![
                        ("type", V::String(text("object"))),
                        ("properties", obj(vec![])),
                    ]),
                ),
                ("readOnly", V::Boolean(route < 4)),
                (
                    "interrupts",
                    V::Boolean(route == 1 || route == 3 || route == 5),
                ),
                ("scene", V::Boolean(route == 2 || route == 3)),
                ("history", V::Boolean(false)),
                ("timeoutMs", V::Number(10.)),
            ]),
            Callbacks {
                run: Rc::new(move |_, ctx| body.run(ctx)),
                label: None,
                discards: None,
            },
        )
        .unwrap();
    let block_env = env.clone();
    runtime
        .register(
            &text("unit"),
            obj(vec![
                ("name", V::String(text("block"))),
                ("title", V::String(text("Block"))),
                ("description", V::String(text("scene blocker"))),
                (
                    "input",
                    obj(vec![
                        ("type", V::String(text("object"))),
                        ("properties", obj(vec![])),
                    ]),
                ),
                ("readOnly", V::Boolean(true)),
                ("scene", V::Boolean(true)),
                ("timeoutMs", V::Number(10.)),
            ]),
            Callbacks {
                run: Rc::new(move |_, ctx| {
                    block_env.log("block:start");
                    block_env.blocks.borrow_mut().push(ctx);
                    Ok(())
                }),
                label: None,
                discards: None,
            },
        )
        .unwrap();
    if extra == 1 {
        let receive = env.clone();
        drop(
            runtime
                .submit_job(
                    text("unit.block"),
                    obj(vec![]),
                    None,
                    None,
                    true,
                    move |result| {
                        receive.log(if result.is_ok() {
                            "reply:block-ok"
                        } else {
                            "reply:block-error"
                        });
                    },
                )
                .unwrap(),
        );
    }
    for receiver_index in 0..if extra == 2 { 101 } else { 1 } {
        let progress_env = env.clone();
        let receive_env = env.clone();
        let job = match job_mode {
            0 => None,
            1 => Some(vec![]),
            2 => Some(text("job")),
            _ => Some({
                let mut s = text("job");
                s.push(0xd800);
                s
            }),
        };
        let submitted = runtime.submit_job(
            text(if route == 6 { "missing" } else { "unit.run" }),
            if route == 7 {
                V::Number(42.)
            } else {
                obj(vec![])
            },
            Some(Rc::new(move |s| {
                let mut m = text("progress:");
                m.extend(s);
                progress_env.log.borrow_mut().push(V::String(m));
                Ok(())
            })),
            job,
            deferred != 0,
            move |result| {
                let value = match result {
                    Ok(v) => {
                        if extra == 2 {
                            let V::Number(n) = &v else {
                                panic!("burst number")
                            };
                            receive_env.log(format!(
                                "reply:{receiver_index}:ok:{}",
                                port_core::js_number::number_string(*n)
                            ));
                        } else {
                            receive_env.log("reply:ok");
                        }
                        V::Array(vec![V::String(text("ok")), v])
                    }
                    Err(e) => {
                        if extra == 2 {
                            receive_env.log(format!("reply:{receiver_index}:error"));
                        } else {
                            receive_env.log("reply:error");
                        }
                        let mut v = vec![V::String(text("err"))];
                        v.extend(receive_env.error(&e));
                        V::Array(v)
                    }
                };
                *receive_env.caller.borrow_mut() = value;
            },
        );
        match submitted {
            Ok(ticket) => drop(ticket),
            Err(SubmitError::Remember(error)) => {
                let mut value = vec![V::String(text("sync"))];
                value.extend(env.error(&error));
                *env.caller.borrow_mut() = V::Array(value)
            }
            Err(e) => panic!("submit {e:?}"),
        }
    }
    let pump = || assert_eq!(owner.turn().unwrap().ready, 0);
    let mut frames = vec![env.frame(&runtime)];
    pump();
    frames.push(env.frame(&runtime));
    env.now.set(1251.5);
    if body_mode == 4 || settle_mode == 2 {
        env.fire()
    }
    pump();
    frames.push(env.frame(&runtime));
    env.release();
    pump();
    frames.push(env.frame(&runtime));
    env.now.set(1220.);
    let blocks = env.blocks.borrow_mut().drain(..).collect::<Vec<_>>();
    for ctx in blocks {
        env.log("block:end");
        ctx.complete(Ok(V::String(text("block"))));
    }
    pump();
    let jobs = env.bodies.borrow_mut().drain(..).collect::<Vec<_>>();
    for ctx in jobs {
        env.doc.set(env.doc.get() + 1);
        ctx.progress(&text("late")).unwrap();
        ctx.complete(Ok(V::String(text("late"))));
    }
    pump();
    frames.push(env.frame(&runtime));
    assert_ne!(*env.caller.borrow(), V::String(text("pending")));
    format!("{id}\t{}\n", frames.join(";"))
}
#[test]
fn native_readonly_profiles_finish() {
    assert_eq!(INPUT.lines().count(), 2591);
    for line in INPUT.lines() {
        observe(line);
    }
}
#[test]
#[ignore = "full original registered readonly/interrupt comparison needs Node22"]
fn pinned_readonly_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/readonly-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), &sha.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(dirty.status.success());
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let built = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-readonly.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("oracle-build.stdout.txt"), &built.stdout).unwrap();
    std::fs::write(dir.join("oracle-build.stderr.txt"), &built.stderr).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/readonly.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(INPUT.as_bytes()).unwrap());
    let source = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    assert!(source.stderr.is_empty());
    let output = String::from_utf8(source.stdout).unwrap();
    let originals: Vec<_> = output.split_inclusive('\n').collect();
    assert_eq!(originals.len(), 2591);
    let mut mismatches = 0;
    for (line, original) in INPUT.lines().zip(originals) {
        let id = line.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("input.bin"), line.as_bytes()).unwrap();
        std::fs::write(case.join("original.bin"), original.as_bytes()).unwrap();
        std::fs::write(case.join("original.stderr.txt"), source.stderr.as_slice()).unwrap();
        let actual = observe(line);
        for (name, bytes) in [
            ("input.bin", line.as_bytes()),
            ("original.bin", original.as_bytes()),
            ("rust.bin", actual.as_bytes()),
            ("original.stderr.txt", source.stderr.as_slice()),
        ] {
            std::fs::write(case.join(name), bytes).unwrap();
        }
        if actual != original {
            mismatches += 1;
            if mismatches <= 3 {
                eprintln!("{id}:\noriginal {original}\nrust {actual}");
            }
        }
        let mut wrong = actual.clone().into_bytes();
        *wrong.last_mut().unwrap() ^= 1;
        assert_ne!(wrong, original.as_bytes(), "wrong value must fail");
        assert_ne!(
            &actual.as_bytes()[..actual.len() - 1],
            original.as_bytes(),
            "missing output must fail"
        );
    }
    println!(
        "2591 registered readonly/interrupt traces; mismatches={mismatches}; raw {}",
        dir.display()
    );
    assert_eq!(mismatches, 0);
}
