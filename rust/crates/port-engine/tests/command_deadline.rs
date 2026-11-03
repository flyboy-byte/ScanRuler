// SPDX-License-Identifier: AGPL-3.0-only
//! Full original limited race, with independent serial timer/body events.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    command_deadline::{COMMAND_TIMEOUT_MS, Completion, Deadline},
    error::{CommandError, CommandErrorCode},
};
use std::sync::Arc;
include!("support/value_wire.rs");
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/deadline_cases.rs")
}
fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    if let Value::Object(a) = v
        && let Some((_, v)) = a
            .iter()
            .find(|(k, _)| k.iter().copied().eq(key.encode_utf16()))
    {
        return v;
    }
    &Value::Undefined
}
fn list(v: &Value) -> &[Value] {
    if let Value::Array(a) = v {
        a
    } else {
        panic!("fixture array")
    }
}
fn units(v: &Value) -> &[u16] {
    if let Value::String(s) = v {
        s
    } else {
        panic!("fixture text")
    }
}
fn txt(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn string(s: &str) -> Value {
    Value::String(txt(s))
}
fn obj(p: Vec<(&str, Value)>) -> Value {
    Value::Object(p.into_iter().map(|(k, v)| (txt(k), v)).collect())
}
#[derive(Clone)]
enum Fault {
    Plain { kind: &'static str, message: Text },
    Command(Arc<CommandError>),
}
enum Body {
    Pending,
    Ok(Text),
    Error(Fault),
    Thrown(Fault),
}
struct Actor {
    deadline: Deadline<Text, Fault>,
    body: Body,
    caller: Option<Completion<Text, Fault>>,
    doc: f64,
    log: Vec<Value>,
    value: Text,
    fallback: Text,
    body_error: Fault,
    policy: String,
}
impl Actor {
    fn new(input: &Value) -> Self {
        let ms = match get(input, "ms") {
            Value::Number(n) => *n,
            _ => panic!("duration"),
        };
        let mut message = txt("body failed ");
        message.push(0xd800);
        let kind = String::from_utf16(units(get(input, "body_kind"))).unwrap();
        let body_error = if kind == "typed" {
            Fault::Command(Arc::new(
                CommandError::from_utf16(CommandErrorCode::InvalidState, &message, 1_000_000)
                    .unwrap(),
            ))
        } else {
            Fault::Plain {
                kind: if kind == "typeerror" {
                    "TypeError"
                } else {
                    "Error"
                },
                message,
            }
        };
        let mut deadline = Deadline::new(units(get(input, "name")), ms, 1_000_000).unwrap();
        deadline.registered(1).unwrap();
        let mut a = Self {
            deadline,
            body: Body::Pending,
            caller: None,
            doc: 0.,
            log: vec![],
            value: units(get(input, "value")).to_vec(),
            fallback: units(get(input, "fallback")).to_vec(),
            body_error,
            policy: String::from_utf16(units(get(input, "abort"))).unwrap(),
        };
        a.log.push(obj(vec![
            ("event", string("timer:set")),
            ("id", Value::Number(1.)),
            ("ms", Value::Number(ms)),
        ]));
        a.event("run");
        match String::from_utf16(units(get(input, "mode")))
            .unwrap()
            .as_str()
        {
            "pending" => {}
            "fire-only" => a.fire(),
            "resolved" => a.resolve(a.value.clone()),
            "rejected" => a.reject(a.body_error.clone()),
            "throw" => {
                a.body = Body::Thrown(a.body_error.clone());
                a.doc = -2.;
                let error = a.error(&a.body_error);
                a.log.push(obj(vec![
                    ("event", string("body:thrown")),
                    ("error", error),
                ]));
                a.deadline.body_settled(Err(a.body_error.clone()));
            }
            "fire-resolve" => {
                a.fire();
                a.resolve(a.value.clone())
            }
            "resolve-fire" => {
                a.resolve(a.value.clone());
                a.fire()
            }
            "fire-reject" => {
                a.fire();
                a.reject(a.body_error.clone())
            }
            "reject-fire" => {
                a.reject(a.body_error.clone());
                a.fire()
            }
            _ => panic!("mode"),
        }
        a.deadline.factory_returned().unwrap();
        a.flush();
        a
    }
    fn event(&mut self, name: &str) {
        self.log.push(obj(vec![("event", string(name))]));
    }
    fn error(&self, e: &Fault) -> Value {
        let mut fields = match e {
            Fault::Plain { kind, message } => vec![
                ("name", string(kind)),
                ("message", Value::String(message.clone())),
            ],
            Fault::Command(e) => vec![
                ("name", string(e.name())),
                ("message", Value::String(e.message_units().to_vec())),
                ("code", string(e.code().as_str())),
            ],
        };
        let same = match (e, self.deadline.abort_reason()) {
            (Fault::Command(e), Some(reason)) => Arc::ptr_eq(e, reason),
            _ => false,
        };
        fields.push(("sameReason", Value::Boolean(same)));
        obj(fields)
    }
    fn resolve(&mut self, value: Text) {
        if !matches!(self.body, Body::Pending) {
            return;
        }
        self.body = Body::Ok(value.clone());
        self.doc = 42.;
        self.log.push(obj(vec![
            ("event", string("body:resolved")),
            ("value", Value::String(value.clone())),
        ]));
        self.deadline.body_settled(Ok(value));
    }
    fn reject(&mut self, e: Fault) {
        if !matches!(self.body, Body::Pending) {
            return;
        }
        let error = self.error(&e);
        self.body = Body::Error(e.clone());
        self.doc = -1.;
        self.log.push(obj(vec![
            ("event", string("body:rejected")),
            ("error", error),
        ]));
        self.deadline.body_settled(Err(e));
    }
    fn fire(&mut self) {
        if !self.deadline.timer_active() {
            self.event("timer:ignored");
            return;
        }
        self.log.push(obj(vec![
            ("event", string("timer:fire")),
            ("id", Value::Number(1.)),
        ]));
        let event = self.deadline.fire_begin().unwrap().unwrap();
        let reason = event.reason().clone();
        let error = self.error(&Fault::Command(reason.clone()));
        self.log
            .push(obj(vec![("event", string("abort")), ("reason", error)]));
        match self.policy.as_str() {
            "resolve" => self.resolve(self.fallback.clone()),
            "reject-reason" => self.reject(Fault::Command(reason)),
            "ignore" => {}
            _ => panic!("abort policy"),
        }
        self.deadline.fire_end(event).unwrap();
    }
    fn flush(&mut self) {
        if let Some(id) = self.deadline.begin_clear() {
            self.log.push(obj(vec![
                ("event", string("timer:clear")),
                ("id", Value::Number(id as f64)),
            ]));
            self.deadline.cleared().unwrap();
            let outcome = self.deadline.take_outcome().unwrap();
            self.event(if matches!(&outcome, Completion::Body(Ok(_))) {
                "caller:ok"
            } else {
                "caller:error"
            });
            self.caller = Some(outcome);
        }
    }
    fn state(&self) -> Value {
        let caller = match &self.caller {
            None => obj(vec![("status", string("pending"))]),
            Some(Completion::Body(Ok(value))) => obj(vec![
                ("status", string("ok")),
                ("value", Value::String(value.clone())),
            ]),
            Some(Completion::Body(Err(e))) => {
                obj(vec![("status", string("error")), ("error", self.error(e))])
            }
            Some(Completion::Timeout(e)) => obj(vec![
                ("status", string("error")),
                ("error", self.error(&Fault::Command(e.clone()))),
            ]),
        };
        let body = match &self.body {
            Body::Pending => obj(vec![("status", string("pending"))]),
            Body::Ok(value) => obj(vec![
                ("status", string("ok")),
                ("value", Value::String(value.clone())),
            ]),
            Body::Error(e) => obj(vec![("status", string("error")), ("error", self.error(e))]),
            Body::Thrown(e) => obj(vec![("status", string("thrown")), ("error", self.error(e))]),
        };
        obj(vec![
            ("caller", caller),
            ("body", body),
            ("doc", Value::Number(self.doc)),
            (
                "aborted",
                Value::Boolean(self.deadline.abort_reason().is_some()),
            ),
            (
                "reason",
                self.deadline
                    .abort_reason()
                    .map(|e| self.error(&Fault::Command(e.clone())))
                    .unwrap_or(Value::Undefined),
            ),
            ("timerActive", Value::Boolean(self.deadline.timer_active())),
            ("log", Value::Array(self.log.clone())),
            ("unhandled", Value::Array(vec![])),
        ])
    }
}
fn payload(b: &[u8]) -> Vec<u8> {
    let mut r = Reader { b, at: 0 };
    let input = r.value();
    assert_eq!(r.at, b.len());
    let mut a = Actor::new(&input);
    let mut frames = vec![a.state()];
    for op in list(get(&input, "ops")) {
        match String::from_utf16(units(op)).unwrap().as_str() {
            "fire" => a.fire(),
            "resolve" => a.resolve(a.value.clone()),
            "reject" => a.reject(a.body_error.clone()),
            "resolve-fire" => {
                a.resolve(a.value.clone());
                a.fire()
            }
            "fire-resolve" => {
                a.fire();
                a.resolve(a.value.clone())
            }
            "reject-fire" => {
                a.reject(a.body_error.clone());
                a.fire()
            }
            "fire-reject" => {
                a.fire();
                a.reject(a.body_error.clone())
            }
            _ => panic!("operation"),
        };
        a.flush();
        frames.push(a.state());
    }
    assert!(a.caller.is_some());
    assert!(!matches!(a.body, Body::Pending));
    let mut bytes = b"CDL1".to_vec();
    encode(
        &mut bytes,
        &obj(vec![
            ("defaultTimeoutMs", Value::Number(COMMAND_TIMEOUT_MS as f64)),
            ("frames", Value::Array(frames)),
        ]),
    );
    bytes
}
#[test]
fn all_owned_deadline_events_settle() {
    assert_eq!(cases().len(), 291);
    for c in cases() {
        assert!(payload(c.input).starts_with(b"CDL1"), "{}", c.id)
    }
}
#[test]
#[ignore = "live original command deadline comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_command_deadline_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/deadline-diff")
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
        "contract: engine.session.command.deadline.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: full pending/busy, ordered notifications/jobs/results and opaque outcomes\nseeds: 1,7,99,1039\n",
    );
    for (label, program, args) in [
        ("dirty", "git", vec!["status", "--porcelain"]),
        ("node", "node", vec!["--version"]),
        ("rustc", "rustc", vec!["--version"]),
    ] {
        let r = Command::new(program)
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(r.status.success());
        identity.push_str(&format!(
            "{label}: {}\n",
            String::from_utf8_lossy(&r.stdout).trim()
        ));
    }
    std::fs::write(run.join("identity.txt"), identity).unwrap();
    let prepared = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-deadline.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(run.join("oracle-build.stdout.txt"), &prepared.stdout).unwrap();
    std::fs::write(run.join("oracle-build.stderr.txt"), &prepared.stderr).unwrap();
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let cases = cases();
    assert_eq!(cases.len(), 291);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/command-deadline.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Read stdout while writing: large outputs cannot deadlock the pipe.
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        stdin.write_all(&input).unwrap();
    });
    let original = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert!(original.stderr.is_empty());
    let mut offset = 0;
    for c in cases {
        let candidate = payload(c.input);
        let len =
            u32::from_le_bytes(original.stdout[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let observed = &original.stdout[offset..offset + len];
        offset += len;
        let d = run.join(c.id);
        std::fs::create_dir_all(&d).unwrap();
        for (name, b) in [
            ("input.bin", c.input),
            ("original.bin", observed),
            ("rust.bin", candidate.as_slice()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(d.join(name), b).unwrap();
        }
        assert!(
            original.status.success(),
            "{}: {}",
            c.id,
            String::from_utf8_lossy(&original.stderr)
        );
        assert!(
            original.stderr.is_empty(),
            "{}: unexpected source diagnostics",
            c.id
        );
        assert!(
            candidate == observed,
            "{}: complete deadline values, abort identity, cleanup and late effects",
            c.id
        );
        if c.id == "case-000" {
            let needle: Vec<_> = " did not finish"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            let mut changed = candidate.clone();
            let at = changed
                .windows(needle.len())
                .position(|b| b == needle)
                .unwrap();
            changed[at] ^= 1;
            assert_ne!(
                changed, observed,
                "changed actual timeout message must fail"
            );
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing results must fail"
            );
        }
        println!(
            "{}: all original deadline state and outcomes compared exactly",
            c.id
        );
    }
    assert_eq!(offset, original.stdout.len());
    println!(
        "291 complete command deadline scenarios; raw {}",
        run.display()
    );
}

#[test]
fn abort_capabilities_and_late_non_clone_faults_are_owned() {
    use std::{cell::Cell, rc::Rc};
    #[derive(Debug)]
    struct Fault(Rc<Cell<usize>>);
    impl Drop for Fault {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let mut d = Deadline::<(), Fault>::new(&txt("probe.run"), 1234., 1_000_000).unwrap();
    d.registered(1).unwrap();
    d.factory_returned().unwrap();
    let event = d.fire_begin().unwrap().unwrap();
    let reason = event.reason().clone();
    assert!(
        d.begin_clear().is_none(),
        "abort dispatch must finish first"
    );
    assert!(d.outcome().is_none());
    d.fire_end(event).unwrap();
    assert_eq!(d.begin_clear(), Some(1));
    assert!(d.outcome().is_none());
    d.cleared().unwrap();
    match d.take_outcome().unwrap() {
        Completion::Timeout(e) => assert!(Arc::ptr_eq(&e, &reason)),
        _ => panic!("timeout"),
    }
    let drops = Rc::new(Cell::new(0));
    assert!(!d.body_settled(Err(Fault(drops.clone()))));
    assert_eq!(drops.get(), 1);
    assert!(!d.body_settled(Err(Fault(drops.clone()))));
    assert_eq!(drops.get(), 2);
    assert!(d.take_outcome().is_none());
    assert!(d.fire_begin().unwrap().is_none());
}
#[test]
fn timer_tokens_cannot_cross_deadlines_or_complete_twice() {
    use port_engine::command_deadline::DeadlineError;
    let mut a = Deadline::<(), ()>::new(&txt("a.run"), 0., 1_000_000).unwrap();
    let mut b = Deadline::<(), ()>::new(&txt("b.run"), 0., 1_000_000).unwrap();
    a.registered(1).unwrap();
    b.registered(1).unwrap();
    a.factory_returned().unwrap();
    b.factory_returned().unwrap();
    let ea = a.fire_begin().unwrap().unwrap();
    let eb = b.fire_begin().unwrap().unwrap();
    assert_eq!(a.fire_end(eb.clone()), Err(DeadlineError::InvalidToken));
    assert!(a.begin_clear().is_none());
    a.fire_end(ea.clone()).unwrap();
    assert_eq!(a.fire_end(ea), Err(DeadlineError::InvalidToken));
    b.fire_end(eb).unwrap();
}
#[test]
fn preexisting_result_waits_for_registration_and_cleanup() {
    let mut d = Deadline::<Text, Text>::new(&txt("probe.run"), 0., 1_000_000).unwrap();
    assert!(d.body_settled(Ok(txt("owned result"))));
    assert!(d.outcome().is_none());
    d.registered(7).unwrap();
    let event = d.fire_begin().unwrap().unwrap();
    d.fire_end(event).unwrap();
    d.factory_returned().unwrap();
    assert_eq!(d.begin_clear(), Some(7));
    assert!(d.outcome().is_none());
    d.cleared().unwrap();
    match d.take_outcome().unwrap() {
        Completion::Body(Ok(value)) => assert_eq!(value, txt("owned result")),
        _ => panic!("array-order tie"),
    }
    assert!(d.abort_reason().is_some());
    assert!(!d.timer_active());
    assert!(matches!(
        Deadline::<(), ()>::new(&txt("too large"), 0., 0),
        Err(port_engine::command_deadline::DeadlineError::Envelope(
            port_engine::error::EnvelopeError::MessageLimit
        ))
    ));
}
