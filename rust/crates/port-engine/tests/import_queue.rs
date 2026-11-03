// SPDX-License-Identifier: AGPL-3.0-only
//! Real original import queue, independently controlled jobs and live listeners.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::import_queue::{ImportQueue, Listener, QueueError, Subscription};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};
include!("support/value_wire.rs");
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/import_queue_cases.rs")
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
fn int(v: &Value) -> usize {
    if let Value::Number(n) = v {
        *n as usize
    } else {
        panic!("fixture number")
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
fn ascii(v: &Value) -> String {
    String::from_utf16(units(v)).unwrap()
}
fn bool_(v: &Value) -> bool {
    matches!(v, Value::Boolean(true))
}
struct Gate {
    resolved: Cell<bool>,
    waker: RefCell<Option<Waker>>,
}
impl Gate {
    fn resolve(&self) {
        self.resolved.set(true);
        let w = self.waker.borrow_mut().take();
        if let Some(w) = w {
            w.wake()
        }
    }
}
#[derive(Clone)]
struct Job {
    id: String,
    value: Text,
    fail: bool,
    immediate: bool,
    gate: Rc<Gate>,
}
struct JobFuture {
    control: Weak<Control>,
    job: Job,
}
impl Future for JobFuture {
    type Output = Result<Text, Text>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.job.immediate && !self.job.gate.resolved.get() {
            *self.job.gate.waker.borrow_mut() = Some(cx.waker().clone());
            return Poll::Pending;
        }
        let c = self.control.upgrade().unwrap();
        c.log.borrow_mut().push(txt(&format!(
            "finish:{}:{}",
            self.job.id,
            c.queue.pending()
        )));
        if self.job.fail {
            let mut error = txt(&format!("job {}", self.job.id));
            error.push(0xd800);
            Poll::Ready(Err(error))
        } else {
            Poll::Ready(Ok(self.job.value.clone()))
        }
    }
}
type TaskFuture = Pin<Box<dyn Future<Output = Result<Text, QueueError<Text>>>>>;
struct Task {
    id: String,
    future: Option<TaskFuture>,
    result: Option<Result<Text, QueueError<Text>>>,
}
struct Control {
    queue: ImportQueue<Text>,
    defs: Vec<Value>,
    hits: Vec<Cell<usize>>,
    callbacks: RefCell<Vec<Listener<Text>>>,
    tokens: RefCell<BTreeMap<Text, Subscription<Text>>>,
    jobs: BTreeMap<String, Job>,
    tasks: RefCell<Vec<Task>>,
    log: RefCell<Vec<Text>>,
}
impl Control {
    fn new(input: &Value) -> Rc<Self> {
        let jobs = list(get(input, "jobs"))
            .iter()
            .map(|j| {
                let id = ascii(get(j, "id"));
                let job = Job {
                    id: id.clone(),
                    value: units(get(j, "value")).to_vec(),
                    fail: bool_(get(j, "fail")),
                    immediate: bool_(get(j, "immediate")),
                    gate: Rc::new(Gate {
                        resolved: Cell::new(false),
                        waker: RefCell::new(None),
                    }),
                };
                (id, job)
            })
            .collect();
        let defs = list(get(input, "listeners")).to_vec();
        let hits = defs.iter().map(|_| Cell::new(0)).collect();
        let c = Rc::new(Self {
            queue: ImportQueue::default(),
            defs,
            hits,
            callbacks: RefCell::new(Vec::new()),
            tokens: RefCell::new(BTreeMap::new()),
            jobs,
            tasks: RefCell::new(Vec::new()),
            log: RefCell::new(Vec::new()),
        });
        for i in 0..c.defs.len() {
            let weak = Rc::downgrade(&c);
            let callback: Listener<Text> = Rc::new(move || weak.upgrade().unwrap().notify(i));
            c.callbacks.borrow_mut().push(callback)
        }
        c
    }
    fn subscribe(self: &Rc<Self>, i: usize, token: Text) {
        let callback = self.callbacks.borrow()[i].clone();
        let subscription = self.queue.subscribe(callback).unwrap();
        self.tokens.borrow_mut().insert(token, subscription);
    }
    fn notify(self: &Rc<Self>, i: usize) -> Result<(), Text> {
        let hit = self.hits[i].get() + 1;
        self.hits[i].set(hit);
        self.log.borrow_mut().push(txt(&format!(
            "notify:{i}:{hit}:{}:{}",
            self.queue.pending(),
            self.queue.busy()
        )));
        let action = get(&self.defs[i], "action");
        if matches!(action, Value::Object(_)) && int(get(action, "at")) == hit {
            match ascii(get(action, "kind")).as_str() {
                "add" => self.subscribe(
                    int(get(action, "listener")),
                    units(get(action, "token")).to_vec(),
                ),
                "remove" => self
                    .tokens
                    .borrow()
                    .get(units(get(action, "token")))
                    .unwrap()
                    .unsubscribe(),
                "enqueue" => self.run(&ascii(get(action, "job")))?,
                _ => panic!("action"),
            }
        }
        if int(get(&self.defs[i], "throwAt")) == hit {
            let mut error = txt(&format!("listener {i}"));
            error.push(0xd800);
            Err(error)
        } else {
            Ok(())
        }
    }
    fn run(self: &Rc<Self>, id: &str) -> Result<(), Text> {
        assert!(
            !self.tasks.borrow().iter().any(|t| t.id == id),
            "duplicate fixture job"
        );
        let job = self.jobs[id].clone();
        let weak = Rc::downgrade(self);
        let future = self.queue.run(move || {
            let c = weak.upgrade().unwrap();
            c.log
                .borrow_mut()
                .push(txt(&format!("start:{}:{}", job.id, c.queue.pending())));
            JobFuture { control: weak, job }
        });
        match future {
            Ok(future) => {
                self.tasks.borrow_mut().push(Task {
                    id: id.to_string(),
                    future: Some(Box::pin(future)),
                    result: None,
                });
                Ok(())
            }
            Err(QueueError::Callback(e)) => Err(e),
            Err(e) => panic!("unexpected host fault {e:?}"),
        }
    }
    fn drain(self: &Rc<Self>) {
        let mut cx = Context::from_waker(Waker::noop());
        for _ in 0..128 {
            let before = (self.log.borrow().len(), self.tasks.borrow().len());
            let len = self.tasks.borrow().len();
            for i in 0..len {
                let future = self.tasks.borrow_mut()[i].future.take();
                if let Some(mut future) = future {
                    match future.as_mut().poll(&mut cx) {
                        Poll::Pending => self.tasks.borrow_mut()[i].future = Some(future),
                        Poll::Ready(result) => {
                            let status = if result.is_ok() { "ok" } else { "error" };
                            let id = self.tasks.borrow()[i].id.clone();
                            self.tasks.borrow_mut()[i].result = Some(result);
                            self.log
                                .borrow_mut()
                                .push(txt(&format!("settled:{id}:{status}")));
                        }
                    }
                }
            }
            if before == (self.log.borrow().len(), self.tasks.borrow().len()) {
                return;
            }
        }
        panic!("fixture drain work limit")
    }
    fn frame(&self, error: Option<Text>) -> Value {
        let results = Value::Array(
            self.tasks
                .borrow()
                .iter()
                .map(|task| {
                    let mut fields = vec![("id", string(&task.id))];
                    match &task.result {
                        None => fields.push(("status", string("pending"))),
                        Some(Ok(value)) => fields.extend([
                            ("status", string("ok")),
                            ("value", Value::String(value.clone())),
                        ]),
                        Some(Err(QueueError::Callback(e))) => fields.extend([
                            ("status", string("error")),
                            ("error", error_value(e.clone())),
                        ]),
                        Some(Err(e)) => panic!("host fault not parity {e:?}"),
                    };
                    obj(fields)
                })
                .collect(),
        );
        obj(vec![
            ("error", error.map(error_value).unwrap_or(Value::Undefined)),
            ("busy", Value::Boolean(self.queue.busy())),
            ("pending", Value::Number(self.queue.pending() as f64)),
            (
                "log",
                Value::Array(
                    self.log
                        .borrow()
                        .iter()
                        .map(|s| Value::String(s.clone()))
                        .collect(),
                ),
            ),
            ("results", results),
        ])
    }
}
fn error_value(message: Text) -> Value {
    obj(vec![
        ("name", string("Error")),
        ("message", Value::String(message)),
    ])
}
fn payload(b: &[u8]) -> Vec<u8> {
    let mut reader = Reader { b, at: 0 };
    let input = reader.value();
    assert_eq!(reader.at, b.len());
    let c = Control::new(&input);
    let mut frames = Vec::new();
    for op in list(get(&input, "ops")) {
        let error = match ascii(get(op, "op")).as_str() {
            "subscribe" => {
                c.subscribe(int(get(op, "listener")), units(get(op, "token")).to_vec());
                None
            }
            "unsubscribe" => {
                c.tokens
                    .borrow()
                    .get(units(get(op, "token")))
                    .unwrap()
                    .unsubscribe();
                None
            }
            "run" => c.run(&ascii(get(op, "job"))).err(),
            "resolve" => {
                c.jobs[&ascii(get(op, "job"))].gate.resolve();
                None
            }
            _ => panic!("operation"),
        };
        c.drain();
        frames.push(c.frame(error));
    }
    assert!(
        c.tasks.borrow().iter().all(|t| t.result.is_some()),
        "unfinished fixture task"
    );
    let mut bytes = b"IQU1".to_vec();
    encode(&mut bytes, &Value::Array(frames));
    bytes
}
#[test]
fn controlled_jobs_settle_on_all_input_schedules() {
    assert_eq!(cases().len(), 67);
    for c in cases() {
        assert!(payload(c.input).starts_with(b"IQU1"), "{}", c.id)
    }
}
#[test]
#[ignore = "live original history recording comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_import_queue_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/import-queue-diff")
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
        "contract: engine.session.import.queue.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: full patch graph identities/payloads, bookkeeping/workspace/capture order and held state\nseeds: 1,7,99,1039\n",
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
        .arg(root.join("docs/port-plan/oracle/build-import-queue.mjs"))
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
    assert_eq!(cases.len(), 67);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/import-queue.mjs"))
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
            "{}: complete history patches and graph data",
            c.id
        );
        if c.id == "fifo" {
            let needle: Vec<_> = "start:A"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            let mut changed = candidate.clone();
            let at = changed
                .windows(needle.len())
                .position(|b| b == needle)
                .unwrap();
            changed[at] ^= 1;
            assert_ne!(changed, observed, "changed actual job event must fail");
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing results must fail"
            );
        }
        println!("{}: all original history state compared exactly", c.id);
    }
    assert_eq!(offset, original.stdout.len());
    println!("67 complete import queue scenarios; raw {}", run.display());
}
