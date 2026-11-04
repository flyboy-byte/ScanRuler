// SPDX-License-Identifier: AGPL-3.0-only
//! Independent actual registered commands, paired with the pinned full command/store graph.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    guarded::{self, Callbacks, Context, Fault, Host, Runtime},
    history_record::{Cell as HistoryCell, Compare, Section, Snapshot},
    schema_value::Value,
    tasks::{Limits as TaskLimits, Owner},
};
use std::{
    cell::{Cell, RefCell},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
    sync::Arc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/guarded/cases.tsv");
fn txt(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|c| format!("{c:04x}")).collect()
}
fn first() -> Vec<u16> {
    let mut v = txt("first ");
    v.push(0xd800);
    v
}
fn object(fields: Vec<(&str, Value)>) -> Value {
    Value::Object(fields.into_iter().map(|(k, v)| (txt(k), v)).collect())
}
struct Timer {
    callback: Rc<dyn Fn() -> Result<(), Fault>>,
    active: bool,
}
struct Env {
    stopped: Cell<bool>,
    done_seen: Cell<usize>,
    done_waits: RefCell<Vec<port_engine::execute::DoneWait>>,
    plugins: port_engine::plugin_busy::Plugins<
        'static,
        usize,
        port_engine::plugin_stop::Stop<'static, port_engine::execute::Done, Fault>,
        Fault,
    >,
    busy: usize,
    body: usize,
    fault: usize,
    doc: Cell<i32>,
    captures: Cell<usize>,
    log: RefCell<Vec<String>>,
    timers: RefCell<Vec<Timer>>,
    jobs: RefCell<Vec<Context>>,
    context: RefCell<Option<Context>>,
    reason: RefCell<Option<Arc<port_engine::error::CommandError>>>,
    inspector: RefCell<Option<guarded::WeakRuntime>>,
    child: RefCell<Option<Result<Value, Fault>>>,
    body_error: Arc<port_engine::error::CommandError>,
    caller: RefCell<Option<Result<Value, Fault>>>,
}
impl Host for Env {
    fn busy(&self) -> Result<Option<Vec<u16>>, Fault> {
        self.log.borrow_mut().push("busy".into());
        if self.stopped.get() {
            return Ok(None);
        }
        match self.busy {
            0 => Ok(None),
            1 => Ok(Some(vec![])),
            2 => Ok(Some(txt("working"))),
            _ => Err(Fault::error("busy fault")),
        }
    }
    fn workspace(&self) -> Result<Vec<u16>, Fault> {
        Ok(txt("elements"))
    }
    fn capture(&self) -> Result<Snapshot, Fault> {
        let n = self.captures.get() + 1;
        self.captures.set(n);
        self.log.borrow_mut().push(format!("capture:{n}"));
        if self.body == 14 {
            self.log.borrow_mut().push(format!(
                "capture-paused:{}",
                usize::from(
                    self.inspector
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .history_paused()
                )
            ));
        }

        if self.fault == 5 && n == 1 || self.fault == 6 && n == 2 {
            return Err(Fault::error("capture fault"));
        }
        Ok(Snapshot {
            sections: vec![Section {
                key: txt("unit"),
                value: HistoryCell::Number(f64::from(self.doc.get())),
                compare: Compare::Identity,
            }],
        })
    }
    fn stale_now(&self) -> Result<f64, Fault> {
        self.now()
    }
    fn now(&self) -> Result<f64, Fault> {
        Ok(1234.)
    }
    fn timer(&self, ms: f64, callback: Rc<dyn Fn() -> Result<(), Fault>>) -> u64 {
        assert_eq!(ms, 10.);
        self.log.borrow_mut().push("timer:set".into());
        let mut timers = self.timers.borrow_mut();
        timers.push(Timer {
            callback,
            active: true,
        });
        timers.len() as u64
    }
    fn clear_timer(&self, id: u64) {
        self.log.borrow_mut().push("timer:clear".into());
        self.timers.borrow_mut()[id as usize - 1].active = false;
    }
    fn settle_readonly(
        &self,
        _: &[u16],
    ) -> Result<Option<port_engine::settle::Work<'static, Fault>>, Fault> {
        panic!("legacy matrix has no readonly")
    }
    fn observer_fault(&self, error: Fault) {
        panic!("unexpected observer fault {error:?}")
    }
    fn settle(&self, _: &[u16]) -> Result<port_engine::settle::Work<'static, Fault>, Fault> {
        self.log.borrow_mut().push("settle".into());
        Ok(Box::pin(std::future::ready(Ok(()))))
    }
    fn stop(&self, done: port_engine::execute::Done) -> port_engine::settle::Work<'static, Fault> {
        match self.plugins.stop_work(Rc::new(done)) {
            Ok(work) => Box::pin(async move { work.await.map_err(Fault::host) }),
            Err(error) => Box::pin(std::future::ready(Err(Fault::host(error)))),
        }
    }
}
impl Env {
    fn error(&self, error: &Fault) -> String {
        match error {
            Fault::Command(e) => format!(
                "e:{}:{}:{}:{}:{}",
                hex(&txt(e.name())),
                hex(&txt(e.code().as_str())),
                hex(e.message_units()),
                usize::from(Arc::ptr_eq(e, &self.body_error)),
                usize::from(
                    self.reason
                        .borrow()
                        .as_ref()
                        .is_some_and(|r| Arc::ptr_eq(r, e))
                )
            ),
            Fault::Error { name, message } => format!("e:{}::{}:0:0", hex(name), hex(message)),
            other => format!("host:{other:?}"),
        }
    }
    fn frame(&self, runtime: &Runtime) -> String {
        let caller = match &*self.caller.borrow() {
            None => "pending".into(),
            Some(Ok(Value::String(v))) => format!("v:{}", hex(v)),
            Some(Err(e)) => self.error(e),
            value => panic!("unexpected result {value:?}"),
        };
        let activity = runtime.activity().current();
        let running = activity.running.as_ref().map_or_else(
            || "-".into(),
            |r| {
                format!(
                    "{}:{}:{}",
                    hex(&r.name),
                    hex(&r.title),
                    port_core::js_number::number_string(r.since)
                )
            },
        );
        let progress = activity
            .progress
            .as_ref()
            .map_or_else(|| "-".into(), |p| hex(p));
        let current = runtime.current().map_or_else(
            || "-".into(),
            |r| format!("{}:{}", hex(&r.name), usize::from(r.deferred)),
        );
        let h = runtime.history();
        let scalar = |value: &HistoryCell| match value {
            HistoryCell::Number(n) => port_core::js_number::number_string(*n),
            _ => panic!("section shape"),
        };
        let past = h
            .past
            .iter()
            .map(|e| {
                format!(
                    "{}:{}:{}:{}:{}:{}",
                    hex(&e.label),
                    hex(&e.workspace),
                    e.group,
                    e.before
                        .iter()
                        .map(|(k, _)| String::from_utf16(k).unwrap())
                        .collect::<Vec<_>>()
                        .join(","),
                    scalar(&e.before[0].1),
                    scalar(&e.after[0].1)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let child = match &*self.child.borrow() {
            None => "-".into(),
            Some(Ok(Value::String(v))) => format!("v:{}", hex(v)),
            Some(Err(e)) => self.error(e),
            _ => panic!("child result"),
        };
        let stale = runtime.page_stale().map_or_else(|| "-".into(), |s| hex(&s));
        let prefix = format!(
            "{caller}|{}|{}|{running}|{progress}|{current}|{past}|{}|{}|{}|{}|{}|{}|0",
            self.doc.get(),
            self.captures.get(),
            h.future.len(),
            usize::from(h.restoring),
            usize::from(h.importing),
            h.revision,
            self.timers.borrow().iter().filter(|t| t.active).count(),
            self.log.borrow().join(",")
        );
        format!(
            "{prefix}|{}|{stale}|{child}|{}",
            usize::from(runtime.history_paused()),
            self.done_seen.get()
        )
    }
    fn fire(&self) {
        let callbacks = {
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
        for callback in callbacks {
            self.log.borrow_mut().push("timer:fire".into());
            callback().unwrap();
        }
    }
    fn finish(&self) {
        let jobs = std::mem::take(&mut *self.jobs.borrow_mut());
        for context in jobs {
            self.doc.set(2);
            if let Err(e) = context.progress(&txt("late")) {
                self.log
                    .borrow_mut()
                    .push(format!("late-throw:{}", self.error(&e)));
                continue;
            }
            context.complete(if self.body == 4 || self.body == 6 {
                Err(Fault::Command(self.body_error.clone()))
            } else {
                Ok(Value::String(txt("late")))
            });
        }
    }
}
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let fields: Vec<usize> = parts[1..].iter().map(|v| v.parse().unwrap()).collect();
    let [interrupt, history, busy, body, label, deferred, fault] = fields[..] else {
        panic!()
    };
    let mut message = txt("body failed ");
    message.push(0xd800);
    let env = Rc::new(Env {
        stopped: Cell::new(false),
        done_seen: Cell::new(0),
        done_waits: RefCell::new(Vec::new()),
        plugins: port_engine::plugin_busy::Plugins::new(port_engine::plugin_busy::Limits {
            slots: 32,
            visits: 32,
            id_units: 32,
            depth: 16,
        }),
        busy,
        body,
        fault,
        doc: Cell::new(0),
        captures: Cell::new(0),
        log: RefCell::new(Vec::new()),
        timers: RefCell::new(Vec::new()),
        jobs: RefCell::new(Vec::new()),
        context: RefCell::new(None),
        reason: RefCell::new(None),
        inspector: RefCell::new(None),
        child: RefCell::new(None),
        body_error: Arc::new(
            port_engine::error::CommandError::from_utf16(
                port_engine::error::CommandErrorCode::InvalidState,
                &message,
                1024,
            )
            .unwrap(),
        ),
        caller: RefCell::new(None),
    });
    let weak = Rc::downgrade(&env);
    let check = Rc::new(move || {
        let env = weak.upgrade().unwrap();
        env.log.borrow_mut().push("plugin-check".into());
        Ok(Some(Rc::from(txt("plugin"))))
    });
    let weak = Rc::downgrade(&env);
    let stop: Rc<port_engine::plugin_stop::Stop<'static, port_engine::execute::Done, Fault>> =
        Rc::new(move |done| {
            let env = weak.upgrade().unwrap();
            env.log.borrow_mut().push("stop".into());
            env.stopped.set(true);
            env.done_waits.borrow_mut().push(done.wait().unwrap());
            Ok(Box::pin(std::future::ready(Ok(()))))
        });
    drop(
        env.plugins
            .register(Rc::from(txt("plugin")), check, Rc::from(vec![]), Some(stop))
            .unwrap(),
    );
    let owner = Owner::new(TaskLimits {
        tasks: 32,
        depth: 16,
        polls_per_turn: 256,
    })
    .unwrap();
    let runtime = Runtime::new(owner.clone(), env.clone(), guarded::Limits::default());
    if body == 13 || body == 14 || body == 15 {
        *env.inspector.borrow_mut() = Some(runtime.downgrade());
    }
    let weak = Rc::downgrade(&env);
    let _subscription = runtime
        .activity()
        .subscribe(Rc::new(move |_, state, _| {
            let env = weak.upgrade().unwrap();
            env.log.borrow_mut().push(if state.running.is_some() {
                format!(
                    "activity:{}",
                    state
                        .progress
                        .as_ref()
                        .map_or_else(|| "start".into(), |p| hex(p))
                )
            } else {
                "activity:clear".into()
            });
            if env.fault == 1 && state.running.is_some() && state.progress.is_none()
                || env.fault == 2 && state.running.is_some() && state.progress.is_some()
                || env.fault == 3 && state.running.is_none()
            {
                return Err(Fault::error("activity fault"));
            }
            Ok(())
        }))
        .unwrap();
    let weak = Rc::downgrade(&env);
    let progress = Rc::new(move |text: &[u16]| {
        let env = weak.upgrade().unwrap();
        env.log
            .borrow_mut()
            .push(format!("caller-progress:{}", hex(text)));
        if env.fault == 4 {
            return Err(Fault::error("progress fault"));
        }
        Ok(())
    });
    let weak = Rc::downgrade(&env);
    let run = Rc::new(move |_: &Value, context: Context| {
        let env = weak.upgrade().unwrap();
        *env.context.borrow_mut() = Some(context.clone());
        env.log.borrow_mut().push(format!(
            "body:{}",
            usize::from(context.deferred() == Some(true))
        ));
        let weak = Rc::downgrade(&env);
        context.on_abort(Rc::new(move |context| {
            let env = weak.upgrade().unwrap();
            *env.reason.borrow_mut() = context.abort_reason();
            env.log.borrow_mut().push("abort".into());
        }))?;
        if env.body == 22 {
            let weak = Rc::downgrade(&env);
            let extra: guarded::AbortListener = Rc::new(move |_| {
                weak.upgrade()
                    .unwrap()
                    .log
                    .borrow_mut()
                    .push("abort-extra".into())
            });
            context.on_abort(extra.clone())?;
            context.on_abort(extra)?;
        }
        env.doc.set(1);
        context.progress(&first())?;
        match env.body {
            0 | 16..=18 | 21 => {
                context.complete(Ok(Value::String(txt("ok"))));
            }
            1 => {
                context.complete(Err(Fault::error("body failed")));
            }
            2 => return Err(Fault::Command(env.body_error.clone())),
            19 => {
                context.complete(Ok(Value::String(txt("ignored"))));
                return Err(Fault::Command(env.body_error.clone()));
            }
            15 => {
                env.inspector
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .clear_history()?;
                context.complete(Ok(Value::String(txt("ok"))));
            }
            7 => {
                let mut text = txt("thrown ");
                text.push(0xd800);
                return Err(Fault::Text(text));
            }
            8 => {
                context.complete(Err(Fault::error(
                    "Failed to fetch dynamically imported module: fixture",
                )));
            }
            9 => {
                return Err(Fault::Other(Rc::new(|| {
                    Err(Fault::Error {
                        name: txt("TypeError"),
                        message: txt("String fault"),
                    })
                })));
            }
            10 => return Err(Fault::Other(Rc::new(|| Ok(txt("false"))))),
            13 => {
                let weak = Rc::downgrade(&env);
                return Err(Fault::Other(Rc::new(move || {
                    let env = weak.upgrade().unwrap();
                    let stale = env
                        .inspector
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .page_stale()
                        .map_or_else(|| "-".into(), |v| hex(&v));
                    env.log.borrow_mut().push(format!("convert-stale:{stale}"));
                    Ok(txt(
                        "Failed to fetch dynamically imported module: converted",
                    ))
                })));
            }
            _ => env.jobs.borrow_mut().push(context),
        }
        Ok(())
    });
    let weak = Rc::downgrade(&env);
    let discards = if (16..=18).contains(&body) {
        Some(Rc::new(move |_: &Value| {
            let env = weak.upgrade().unwrap();
            env.log.borrow_mut().push("discards".into());
            if body == 18 {
                return Err(Fault::error("discard fault"));
            }
            Ok(if body == 16 {
                Value::Boolean(true)
            } else {
                Value::String(txt("truthy"))
            })
        }) as guarded::DiscardsProvider)
    } else {
        None
    };
    let weak = Rc::downgrade(&env);
    let label_fn = if label == 0 {
        None
    } else {
        Some(Rc::new(move |_: &Value| {
            let env = weak.upgrade().unwrap();
            env.log.borrow_mut().push("label".into());
            if label == 3 {
                return Err(Fault::error("label fault"));
            }
            if label == 5 {
                let weak = Rc::downgrade(&env);
                return Err(Fault::Other(Rc::new(move || {
                    weak.upgrade()
                        .unwrap()
                        .log
                        .borrow_mut()
                        .push("convert-label".into());
                    Ok(txt("label object"))
                })));
            }

            Ok(match label {
                1 => {
                    let mut v = txt("Custom ");
                    v.push(0xd800);
                    Some(v)
                }
                2 => Some(vec![]),
                _ => None,
            })
        })
            as Rc<dyn Fn(&Value) -> Result<Option<Vec<u16>>, Fault>>)
    };
    let definition = object(vec![
        ("name", Value::String(txt("run"))),
        ("title", Value::String(txt("ΟΣ 🦀"))),
        ("description", Value::String(txt("fixture"))),
        (
            "input",
            object(vec![
                ("type", Value::String(txt("object"))),
                ("properties", object(vec![])),
            ]),
        ),
        ("timeoutMs", Value::Number(10.)),
        ("history", Value::Boolean(history != 0)),
        ("interrupts", Value::Boolean(interrupt != 0)),
        ("scene", Value::Boolean(body == 21)),
    ]);
    runtime
        .register(
            &txt("unit"),
            definition,
            Callbacks {
                run,
                label: label_fn,
                discards,
            },
        )
        .unwrap();
    let weak = Rc::downgrade(&env);
    runtime
        .register(
            &txt("unit"),
            object(vec![
                ("name", Value::String(txt("stop"))),
                ("title", Value::String(txt("Stop"))),
                ("description", Value::String(txt("nested interrupt"))),
                (
                    "input",
                    object(vec![
                        ("type", Value::String(txt("object"))),
                        ("properties", object(vec![])),
                    ]),
                ),
                ("interrupts", Value::Boolean(true)),
                ("timeoutMs", Value::Number(10.)),
            ]),
            Callbacks {
                run: Rc::new(move |_, context| {
                    let env = weak.upgrade().unwrap();
                    env.log.borrow_mut().push(format!(
                        "stop:{}",
                        usize::from(context.deferred() == Some(true))
                    ));
                    env.doc.set(3);
                    context.progress(&txt("stop"))?;
                    context.complete(Ok(Value::String(txt("stop"))));
                    Ok(())
                }),
                label: None,
                discards: None,
            },
        )
        .unwrap();
    let receiver = env.clone();
    drop(
        runtime
            .submit(
                txt("unit.run"),
                object(vec![]),
                Some(progress),
                deferred != 0,
                move |result| {
                    receiver.log.borrow_mut().push(
                        if result.is_ok() {
                            "reply:ok"
                        } else {
                            "reply:error"
                        }
                        .into(),
                    );
                    *receiver.caller.borrow_mut() = Some(result);
                },
            )
            .unwrap(),
    );
    let pump = || {
        let turn = owner.turn().unwrap();
        assert_eq!(
            turn.ready, 0,
            "profile must reach quiescence within its finite turn"
        );
        use std::{
            future::Future,
            pin::Pin,
            task::{Context as TaskContext, Waker},
        };
        let mut cx = TaskContext::from_waker(Waker::noop());
        let mut waits = env.done_waits.borrow_mut();
        let mut i = 0;
        while i < waits.len() {
            if Pin::new(&mut waits[i]).poll(&mut cx).is_ready() {
                drop(waits.remove(i));
                env.done_seen.set(env.done_seen.get() + 1);
            } else {
                i += 1;
            }
        }
    };
    let mut frames = vec![env.frame(&runtime)];
    pump();
    frames.push(env.frame(&runtime));
    if body == 11 || body == 12 {
        let receiver = env.clone();
        let weak = Rc::downgrade(&env);
        let progress = Rc::new(move |text: &[u16]| {
            let env = weak.upgrade().unwrap();
            env.log
                .borrow_mut()
                .push(format!("caller-progress:{}", hex(text)));
            Ok(())
        });
        drop(
            runtime
                .submit(
                    txt("unit.stop"),
                    object(vec![]),
                    Some(progress),
                    true,
                    move |result| {
                        receiver.log.borrow_mut().push(
                            if result.is_ok() {
                                "reply:stop"
                            } else {
                                "reply:stop-error"
                            }
                            .into(),
                        );
                        *receiver.child.borrow_mut() = Some(result);
                    },
                )
                .unwrap(),
        );
        pump();
        frames.push(env.frame(&runtime));
    }
    if body == 5 || body == 6 || body == 12 || body == 22 {
        env.fire();
    }
    pump();
    frames.push(env.frame(&runtime));
    env.finish();
    pump();
    frames.push(env.frame(&runtime));
    assert!(env.caller.borrow().is_some());
    env.inspector.borrow_mut().take();
    owner.finish().unwrap();
    format!("{}\t{}\n", parts[0], frames.join(";"))
}
#[test]
fn native_registered_commands_complete_all_profiles() {
    assert_eq!(INPUT.lines().count(), 1972);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "full original registered guarded/interrupt comparison needs Node22"]
fn pinned_guarded_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/guarded-diff/run-{stamp}"));
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
        .arg(root.join("docs/port-plan/oracle/build-guarded.mjs"))
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
        .arg(root.join("docs/port-plan/oracle/guarded.mjs"))
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
    assert_eq!(originals.len(), 1972);
    let sabotage = std::env::var_os("SCANRULER_GUARDED_SABOTAGE").is_some();
    let mut mismatches = 0;
    for (line, original) in INPUT.lines().zip(originals) {
        let id = line.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("input.bin"), line.as_bytes()).unwrap();
        std::fs::write(case.join("original.bin"), original.as_bytes()).unwrap();
        std::fs::write(case.join("original.stderr.txt"), source.stderr.as_slice()).unwrap();
        let mut actual = observe(line);
        if sabotage {
            actual = actual.replacen("|1|", "|9|", 1);
        }
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
        "1972 registered guarded/interrupt traces; mismatches={mismatches}; raw {}",
        dir.display()
    );
    assert_eq!(mismatches, 0);
}

#[test]
fn session_shutdown_drops_owned_producers_without_a_runtime_owner_cycle() {
    struct Quiet;
    impl Host for Quiet {
        fn busy(&self) -> Result<Option<Vec<u16>>, Fault> {
            Ok(None)
        }
        fn workspace(&self) -> Result<Vec<u16>, Fault> {
            Ok(txt("elements"))
        }
        fn capture(&self) -> Result<Snapshot, Fault> {
            Ok(Snapshot { sections: vec![] })
        }
        fn stale_now(&self) -> Result<f64, Fault> {
            self.now()
        }
        fn now(&self) -> Result<f64, Fault> {
            Ok(0.)
        }
        fn timer(&self, _: f64, _: Rc<dyn Fn() -> Result<(), Fault>>) -> u64 {
            1
        }
        fn clear_timer(&self, _: u64) {}
        fn settle_readonly(
            &self,
            _: &[u16],
        ) -> Result<Option<port_engine::settle::Work<'static, Fault>>, Fault> {
            Ok(None)
        }
        fn observer_fault(&self, error: Fault) {
            panic!("unexpected observer fault {error:?}")
        }
        fn settle(&self, _: &[u16]) -> Result<port_engine::settle::Work<'static, Fault>, Fault> {
            Ok(Box::pin(std::future::ready(Ok(()))))
        }
        fn stop(&self, _: port_engine::execute::Done) -> port_engine::settle::Work<'static, Fault> {
            Box::pin(std::future::ready(Ok(())))
        }
    }
    struct Released(Rc<Cell<usize>>);
    impl Drop for Released {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let owner = Owner::new(TaskLimits {
        tasks: 8,
        depth: 8,
        polls_per_turn: 64,
    })
    .unwrap();
    let weak_owner = owner.downgrade();
    let host = Rc::new(Quiet);
    let weak_host = Rc::downgrade(&host);
    let runtime = Runtime::new(owner.clone(), host, guarded::Limits::default());
    let weak_runtime = runtime.downgrade();
    let released = Rc::new(Cell::new(0));
    let marker = released.clone();
    runtime
        .register(
            &txt("unit"),
            object(vec![
                ("name", Value::String(txt("pending"))),
                ("title", Value::String(txt("Pending"))),
                ("description", Value::String(txt("lifetime"))),
                (
                    "input",
                    object(vec![("type", Value::String(txt("object")))]),
                ),
                ("history", Value::Boolean(false)),
            ]),
            Callbacks {
                run: Rc::new(move |_, context| {
                    let marker = marker.clone();
                    drop(context.spawn(async move {
                        let _held = Released(marker);
                        std::future::pending::<()>().await;
                        Ok(Value::Null)
                    })?);
                    Ok(())
                }),
                label: None,
                discards: None,
            },
        )
        .unwrap();
    drop(
        runtime
            .submit(txt("unit.pending"), object(vec![]), None, false, |_| {
                panic!("a closed session has no caller continuation")
            })
            .unwrap(),
    );
    owner.turn().unwrap();
    assert_eq!(owner.active(), 3);
    assert_eq!(released.get(), 0);
    drop(runtime);
    drop(owner);
    assert!(weak_owner.upgrade().is_none());
    assert!(weak_runtime.upgrade().is_none());
    assert!(weak_host.upgrade().is_none());
    assert_eq!(released.get(), 1);
}
