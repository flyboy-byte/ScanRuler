// SPDX-License-Identifier: AGPL-3.0-only
//! Real-source recording traces, including graph payload and reference identity.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::history_record::{
    Cell, Compare, Entry, Guards, HistoryError, Limits, Patch, Recorder, Section, Snapshot, Token,
};
use std::sync::Arc;
include!("support/value_wire.rs");
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/history_record_cases.rs")
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
fn flag(v: &Value) -> bool {
    matches!(v, Value::Boolean(true))
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
fn atom(v: &Value, pool: &[Cell]) -> Cell {
    match v {
        Value::Undefined => Cell::Undefined,
        Value::Null => Cell::Null,
        Value::Boolean(b) => Cell::Boolean(*b),
        Value::Number(n) => Cell::Number(*n),
        Value::String(s) => Cell::String(s.clone()),
        Value::Object(_) => pool[int(get(v, "ref"))].clone(),
        _ => panic!("fixture atom"),
    }
}
fn own_order(keys: &[Text]) -> Vec<usize> {
    let mut ids: Vec<_> = (0..keys.len()).collect();
    ids.sort_by_key(|&i| (index(&keys[i]).is_none(), index(&keys[i]).unwrap_or(0)));
    ids
}
struct Slot {
    key: Text,
    value: Cell,
    identity: bool,
}
struct Doc {
    pool: Vec<Cell>,
    slots: Vec<Slot>,
    workspace: Text,
    busy: bool,
    hook: bool,
    participant: bool,
    log: Vec<Text>,
    capture: usize,
    fail: usize,
}
impl Doc {
    fn new(case: &Value) -> Self {
        let mut pool = Vec::new();
        for node in list(get(case, "nodes")) {
            let kind = String::from_utf16(units(get(node, "kind"))).unwrap();
            let c = match kind.as_str() {
                "object" => Cell::Object(Arc::new(
                    list(get(node, "fields"))
                        .iter()
                        .map(|p| {
                            let p = list(p);
                            (units(&p[0]).to_vec(), atom(&p[1], &pool))
                        })
                        .collect(),
                )),
                "array" => Cell::Array(Arc::new(
                    list(get(node, "items"))
                        .iter()
                        .map(|v| atom(v, &pool))
                        .collect(),
                )),
                "bytes" => Cell::Bytes(Arc::new(
                    list(get(node, "values"))
                        .iter()
                        .map(|v| int(v) as u8)
                        .collect(),
                )),
                "words" => Cell::Words(Arc::new(
                    list(get(node, "values"))
                        .iter()
                        .map(|v| int(v) as u32)
                        .collect(),
                )),
                _ => panic!("fixture node"),
            };
            pool.push(c);
        }
        let slots = list(get(case, "sections"))
            .iter()
            .map(|s| Slot {
                key: units(get(s, "key")).to_vec(),
                value: atom(get(s, "value"), &pool),
                identity: flag(get(s, "identity")),
            })
            .collect();
        Self {
            pool,
            slots,
            workspace: if matches!(get(case, "workspace"), Value::Undefined) {
                txt("elements")
            } else {
                units(get(case, "workspace")).to_vec()
            },
            busy: false,
            hook: false,
            participant: false,
            log: Vec::new(),
            capture: 0,
            fail: 0,
        }
    }
    fn held(&self, h: &Recorder) -> bool {
        h.state.restoring || h.paused() || self.busy || self.hook || self.participant
    }
    fn capture(&mut self, h: &Recorder) -> Result<Snapshot, HistoryError> {
        let mut order: Vec<_> = (0..self.slots.len()).collect();
        order.sort_by_key(|&i| {
            if self.slots[i].key == txt("scope") {
                0
            } else if self.slots[i].key == txt("scan") {
                1
            } else {
                2
            }
        });
        let mut sections = Vec::new();
        for i in order {
            self.capture += 1;
            self.log.push(
                [
                    txt("capture:"),
                    self.slots[i].key.clone(),
                    txt(":"),
                    txt(if self.held(h) { "true" } else { "false" }),
                ]
                .concat(),
            );
            if self.capture == self.fail {
                return Err(HistoryError::Capture(txt("capture failed 🦀")));
            }
            let s = &self.slots[i];
            sections.push(Section {
                key: s.key.clone(),
                value: s.value.clone(),
                compare: if s.identity {
                    Compare::Identity
                } else {
                    Compare::Fields
                },
            });
        }
        let order = own_order(&sections.iter().map(|s| s.key.clone()).collect::<Vec<_>>());
        Ok(Snapshot {
            sections: order.into_iter().map(|i| sections[i].clone()).collect(),
        })
    }
    fn cell(&self, c: &Cell) -> Value {
        match c {
            Cell::Undefined => Value::Undefined,
            Cell::Null => Value::Null,
            Cell::Boolean(b) => Value::Boolean(*b),
            Cell::Number(n) => Value::Number(*n),
            Cell::String(s) => Value::String(s.clone()),
            _ => {
                let id = self
                    .pool
                    .iter()
                    .position(|p| p.strict_equal(c))
                    .expect("actual recorded composite retains input identity");
                let mut fields = vec![("id", Value::Number(id as f64))];
                match c {
                    Cell::Array(a) => fields.extend([
                        ("kind", string("array")),
                        (
                            "items",
                            Value::Array(a.iter().map(|c| self.cell(c)).collect()),
                        ),
                    ]),
                    Cell::Object(a) => {
                        let order =
                            own_order(&a.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>());
                        fields.extend([
                            ("kind", string("object")),
                            (
                                "fields",
                                Value::Array(
                                    order
                                        .into_iter()
                                        .map(|i| {
                                            Value::Array(vec![
                                                Value::String(a[i].0.clone()),
                                                self.cell(&a[i].1),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ]);
                    }
                    Cell::Bytes(a) => fields.extend([
                        ("kind", string("bytes")),
                        ("values", Value::Bytes(a.as_ref().clone())),
                    ]),
                    Cell::Words(a) => fields.extend([
                        ("kind", string("words")),
                        (
                            "values",
                            Value::Array(a.iter().map(|n| Value::Number(f64::from(*n))).collect()),
                        ),
                    ]),
                    _ => unreachable!(),
                }
                obj(fields)
            }
        }
    }
    fn patch(&self, p: &Patch) -> Value {
        Value::Array(
            p.iter()
                .map(|(k, v)| Value::Array(vec![Value::String(k.clone()), self.cell(v)]))
                .collect(),
        )
    }
    fn entry(&self, e: &Entry) -> Value {
        obj(vec![
            ("before", self.patch(&e.before)),
            ("after", self.patch(&e.after)),
            ("label", Value::String(e.label.clone())),
            ("workspace", Value::String(e.workspace.clone())),
            ("group", Value::Number(e.group as f64)),
        ])
    }
}
struct Body {
    token: Result<Option<Token>, HistoryError>,
    leave: Vec<Value>,
}
struct Driver {
    history: Recorder,
    doc: Doc,
    pending: Vec<(usize, Body)>,
}
fn user_error(e: HistoryError) -> Value {
    match e {
        HistoryError::Capture(message) => obj(vec![
            ("name", string("Error")),
            ("message", Value::String(message)),
        ]),
        e => panic!("unexpected host refusal {e:?}"),
    }
}
impl Driver {
    fn state(&self, error: Option<HistoryError>) -> Value {
        let h = &self.history;
        let d = &self.doc;
        obj(vec![
            ("error", error.map(user_error).unwrap_or(Value::Undefined)),
            ("workspace", Value::String(d.workspace.clone())),
            (
                "past",
                Value::Array(h.state.past.iter().map(|e| d.entry(e)).collect()),
            ),
            (
                "future",
                Value::Array(h.state.future.iter().map(|e| d.entry(e)).collect()),
            ),
            ("revision", Value::Number(h.state.revision as f64)),
            ("restoring", Value::Boolean(h.state.restoring)),
            ("importing", Value::Boolean(h.state.importing)),
            ("blocked", Value::Boolean(d.held(h))),
            (
                "current",
                Value::Array(
                    d.slots
                        .iter()
                        .map(|s| {
                            obj(vec![
                                ("key", Value::String(s.key.clone())),
                                ("identity", Value::Boolean(s.identity)),
                                ("value", d.cell(&s.value)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "pending",
                Value::Array(
                    self.pending
                        .iter()
                        .map(|(id, _)| Value::Number(*id as f64))
                        .collect(),
                ),
            ),
            (
                "log",
                Value::Array(d.log.iter().cloned().map(Value::String).collect()),
            ),
        ])
    }
    fn steps(&mut self, steps: &[Value]) -> Result<(), HistoryError> {
        for op in steps {
            self.sync(op)?;
        }
        Ok(())
    }
    fn sync(&mut self, op: &Value) -> Result<(), HistoryError> {
        let kind = String::from_utf16(units(get(op, "op"))).unwrap();
        match kind.as_str() {
            "set" => {
                let i = int(get(op, "section"));
                self.doc.slots[i].value = atom(get(op, "value"), &self.doc.pool);
                self.doc.log.push(txt(&format!("set:{i}")));
            }
            "workspace" => self.doc.workspace = units(get(op, "value")).to_vec(),
            "group" => self.history.begin_group()?,
            "end_group" => self.history.end_group(),
            "clear" => self.history.clear()?,
            "mode" => self.doc.slots[int(get(op, "section"))].identity = flag(get(op, "identity")),
            "flags" => {
                let f = get(op, "value");
                for (name, out) in [
                    ("busy", &mut self.doc.busy),
                    ("hook", &mut self.doc.hook),
                    ("participant", &mut self.doc.participant),
                    ("restoring", &mut self.history.state.restoring),
                    ("importing", &mut self.history.state.importing),
                ] {
                    if !matches!(get(f, name), Value::Undefined) {
                        *out = flag(get(f, name));
                    }
                }
            }
            "capture_fail" => self.doc.fail = self.doc.capture + int(get(op, "offset")),
            "seed_future" => {
                self.history.state.future = vec![self.history.state.past.last().unwrap().clone()]
            }
            "throw" => {
                return Err(HistoryError::Capture(
                    if matches!(get(op, "message"), Value::Undefined) {
                        txt("body failed 🦀")
                    } else {
                        units(get(op, "message")).to_vec()
                    },
                ));
            }
            "sync" => {
                let label = units(get(op, "label"));
                self.doc.log.push([txt("sync:"), label.to_vec()].concat());
                let workspace = self.doc.workspace.clone();
                let token = self.history.begin_sync(
                    label,
                    &workspace,
                    Guards {
                        busy: self.doc.busy,
                        hook_blocked: self.doc.hook,
                    },
                    |h| self.doc.capture(h),
                );
                let result = match token {
                    Err(e) => Err(e),
                    Ok(token) => {
                        let ran = self.steps(list(get(op, "steps")));
                        if let Some(token) = token {
                            self.history.finish(token, |h| self.doc.capture(h)).and(ran)
                        } else {
                            ran
                        }
                    }
                };
                self.doc
                    .log
                    .push([txt("sync-end:"), label.to_vec()].concat());
                result?;
            }
            "start_async" => {
                let id = int(get(op, "id"));
                assert!(!self.pending.iter().any(|(i, _)| *i == id));
                let label = units(get(op, "label"));
                self.doc.log.push([txt("async:"), label.to_vec()].concat());
                let workspace = self.doc.workspace.clone();
                let token = self
                    .history
                    .begin_async(label, &workspace, |h| self.doc.capture(h));
                // A before-capture failure becomes an already rejected source Promise. Its
                // caller sees failure only when awaiting it, with no body entered.
                if token.is_ok() && !matches!(get(op, "enter"), Value::Undefined) {
                    self.steps(list(get(op, "enter")))?;
                }
                self.pending.push((
                    id,
                    Body {
                        token,
                        leave: if matches!(get(op, "leave"), Value::Undefined) {
                            vec![]
                        } else {
                            list(get(op, "leave")).to_vec()
                        },
                    },
                ));
            }
            _ => panic!("fixture op {kind}"),
        };
        Ok(())
    }
    fn finish(&mut self, id: usize) -> Result<(), HistoryError> {
        let index = self.pending.iter().position(|(i, _)| *i == id).unwrap();
        let (_, body) = self.pending.remove(index);
        let result = match body.token {
            Err(e) => Err(e),
            Ok(token) => {
                let ran = self.steps(&body.leave);
                if let Some(token) = token {
                    self.history.finish(token, |h| self.doc.capture(h)).and(ran)
                } else {
                    ran
                }
            }
        };
        self.doc.log.push(txt(&format!("async-end:{id}")));
        result
    }
}
fn payload(b: &[u8]) -> Vec<u8> {
    let mut reader = Reader { b, at: 0 };
    let input = reader.value();
    assert_eq!(reader.at, b.len());
    let mut driver = Driver {
        history: Recorder::default(),
        doc: Doc::new(&input),
        pending: vec![],
    };
    let mut frames = Vec::new();
    for op in list(get(&input, "ops")) {
        let result = if units(get(op, "op")) == txt("finish_async") {
            driver.finish(int(get(op, "id")))
        } else {
            driver.sync(op)
        };
        frames.push(driver.state(result.err()));
    }
    assert!(driver.pending.is_empty());
    let mut b = b"HRC1".to_vec();
    encode(&mut b, &Value::Array(frames));
    b
}
#[test]
fn all_authored_traces_have_complete_typed_recording_state() {
    for c in cases() {
        assert!(payload(c.input).starts_with(b"HRC1"), "{}", c.id)
    }
}

#[test]
#[ignore = "live original history recording comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_history_record_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/history-record-diff")
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
        "contract: engine.session.history.record.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: full patch graph identities/payloads, bookkeeping/workspace/capture order and held state\nseeds: 1,7,99,1079\n",
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
        .arg(root.join("docs/port-plan/oracle/build-session-history.mjs"))
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
    assert_eq!(cases.len(), 107);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/history-record.mjs"))
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
        assert_eq!(
            candidate, observed,
            "{}: complete history patches and graph data",
            c.id
        );
        if c.id == "simple" {
            let needle: Vec<_> = "first".encode_utf16().flat_map(u16::to_le_bytes).collect();
            let mut changed = candidate.clone();
            let at = changed
                .windows(needle.len())
                .position(|b| b == needle)
                .unwrap();
            changed[at] ^= 1;
            assert_ne!(changed, observed, "changed label must fail");
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing patch data must fail"
            );
        }
        println!("{}: all original history state compared exactly", c.id);
    }
    assert_eq!(offset, original.stdout.len());
    println!(
        "107 complete history recording scenarios; raw {}",
        run.display()
    );
}

#[test]
fn metadata_limits_and_domain_bound_completions_are_safe() {
    let shared = Arc::new(vec![7u8; 2_100_000]);
    let root = Cell::Object(Arc::new(vec![(txt("bytes"), Cell::Bytes(shared.clone()))]));
    let snapshot = Snapshot {
        sections: vec![Section {
            key: txt("probe"),
            value: root,
            compare: Compare::Fields,
        }],
    };
    let mut a = Recorder::default();
    let mut b = Recorder::default();
    let token_a = a
        .begin_async(&txt("a"), &txt("elements"), |_| Ok(snapshot.clone()))
        .unwrap()
        .unwrap();
    let token_b = b
        .begin_async(&txt("b"), &txt("elements"), |_| Ok(snapshot.clone()))
        .unwrap()
        .unwrap();
    assert!(matches!(
        b.finish(token_a.clone(), |_| Ok(snapshot.clone())),
        Err(HistoryError::InvalidToken)
    ));
    assert!(b.paused());
    let stale = token_a.clone();
    a.clear().unwrap();
    assert_eq!(a.epoch(), 1);
    assert_eq!(a.state.revision, 1);
    assert!(a.paused());
    a.finish(token_a, |h| {
        assert!(!h.paused());
        Ok(snapshot.clone())
    })
    .unwrap();
    assert!(a.state.past.is_empty());
    assert!(matches!(
        a.finish(stale, |_| Ok(snapshot.clone())),
        Err(HistoryError::InvalidToken)
    ));
    b.finish(token_b, |h| {
        assert!(!h.paused());
        Ok(snapshot.clone())
    })
    .unwrap();
    assert!(b.state.past.is_empty());
    assert_eq!(b.state.revision, 0);
    assert_eq!(
        Arc::strong_count(&shared),
        2,
        "snapshots share the payload, not copies"
    );
    for limits in [
        Limits {
            sections: 0,
            ..Limits::default()
        },
        Limits {
            fields: 0,
            ..Limits::default()
        },
        Limits {
            cells: 0,
            ..Limits::default()
        },
        Limits {
            units: 0,
            ..Limits::default()
        },
        Limits {
            pending: 0,
            ..Limits::default()
        },
        Limits {
            text: 0,
            ..Limits::default()
        },
    ] {
        let mut h = Recorder::new(limits);
        assert!(matches!(
            h.begin_async(&txt("a"), &txt("elements"), |_| Ok(snapshot.clone())),
            Err(HistoryError::Limit)
        ));
        assert!(!h.paused());
        assert_eq!(h.epoch(), 0);
    }
}
