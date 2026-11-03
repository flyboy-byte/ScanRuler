// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original core capture/apply at the borrowed owned-data boundary.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::history_record::core::{self, CoreError, Fields, Participant, Stores};
use port_engine::history_record::{Cell, Limits, Patch};
use std::{cell::RefCell, rc::Rc, sync::Arc};
include!("support/value_wire.rs");
struct Case {
    id: &'static str,
    input: &'static [u8],
}
fn cases() -> Vec<Case> {
    include!("support/history_core_cases.rs")
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
struct Observer {
    ids: Vec<Cell>,
}
impl Observer {
    fn cell(&mut self, c: &Cell) -> Value {
        match c {
            Cell::Undefined => Value::Undefined,
            Cell::Null => Value::Null,
            Cell::Boolean(b) => Value::Boolean(*b),
            Cell::Number(n) => Value::Number(*n),
            Cell::String(s) => Value::String(s.clone()),
            _ => {
                let id = if let Some(i) = self.ids.iter().position(|p| p.strict_equal(c)) {
                    i
                } else {
                    self.ids.push(c.clone());
                    self.ids.len() - 1
                };
                let mut fields = vec![("id", Value::Number(id as f64))];
                match c {
                    Cell::Array(a) => fields.extend([
                        ("kind", string("array")),
                        (
                            "items",
                            Value::Array(a.iter().map(|x| self.cell(x)).collect()),
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
                            Value::Array(a.iter().map(|&n| Value::Number(f64::from(n))).collect()),
                        ),
                    ]),
                    _ => unreachable!(),
                };
                obj(fields)
            }
        }
    }
}
fn fields(v: &Value, pool: &[Cell]) -> Fields {
    list(v)
        .iter()
        .map(|pair| {
            let pair = list(pair);
            (units(&pair[0]).to_vec(), atom(&pair[1], pool))
        })
        .collect()
}
// Observer-only stringification for the fixed version-counter domain. The actual
// state and all captured numbers also compare as complete float64 wire bytes.
fn number(c: &Cell) -> String {
    let Cell::Number(n) = c else {
        panic!("version")
    };
    if n.is_nan() {
        "NaN".into()
    } else if *n == f64::INFINITY {
        "Infinity".into()
    } else if *n == f64::NEG_INFINITY {
        "-Infinity".into()
    } else if *n == 0. {
        "0".into()
    } else if *n == 1e21 {
        "1e+21".into()
    } else {
        assert!(n.abs() < 1e21 && n.abs() >= 1e-6, "observer domain");
        n.to_string()
    }
}
fn event(log: &Rc<RefCell<Vec<Text>>>, name: &str, stores: &Stores<'_>) {
    log.borrow_mut().push(txt(&format!(
        "{name}:{}:{}",
        number(core::get(stores.flat, "subjectVersion")),
        number(core::get(stores.deviation, "scopeVersion"))
    )))
}
fn fail(fault: &str, name: &str) -> Result<(), Text> {
    if fault == name {
        let mut s = txt(&format!("callback {name}"));
        s.push(0xd800);
        Err(s)
    } else {
        Ok(())
    }
}
fn payload(b: &[u8]) -> Vec<u8> {
    let mut reader = Reader { b, at: 0 };
    let input = reader.value();
    assert_eq!(reader.at, b.len());
    let mut pool = Vec::new();
    for node in list(get(&input, "nodes")) {
        let kind = String::from_utf16(units(get(node, "kind"))).unwrap();
        let value = match kind.as_str() {
            "object" => Cell::Object(Arc::new(fields(get(node, "fields"), &pool))),
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
            _ => panic!("node"),
        };
        pool.push(value);
    }
    let mut data: [Fields; 4] = list(get(&input, "stores"))
        .iter()
        .map(|v| fields(v, &pool))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let mut keys: Vec<Vec<Text>> = data
        .iter()
        .map(|fields| fields.iter().map(|(k, _)| k.clone()).collect())
        .collect();
    let scope = Rc::new(RefCell::new(atom(get(&input, "scope"), &pool)));
    let scan = atom(get(&input, "scan"), &pool);
    let log = Rc::new(RefCell::new(Vec::new()));
    let names: Vec<Text> = list(get(&input, "participants"))
        .iter()
        .map(|v| units(v).to_vec())
        .collect();
    let slots = Rc::new(RefCell::new(vec![Cell::Undefined; names.len()]));
    let fault = String::from_utf16(units(get(&input, "fault"))).unwrap();
    let mut observer = Observer { ids: pool.clone() };
    let mut frames = Vec::new();
    for op in list(get(&input, "ops")) {
        let [measure, flat, deviation, thickness] = &mut data;
        let mut stores = Stores {
            measure,
            flat,
            deviation,
            thickness,
        };
        let mut snapshot = Value::Undefined;
        let result = if units(get(op, "op"))
            .iter()
            .copied()
            .eq("capture".encode_utf16())
        {
            core::capture(
                &stores,
                Limits::default(),
                || {
                    event(&log, "scope-get", &stores);
                    fail(&fault, "scope-get")?;
                    Ok(scope.borrow().clone())
                },
                || {
                    event(&log, "scan-get", &stores);
                    fail(&fault, "scan-get")?;
                    Ok(scan.clone())
                },
            )
            .map(|s| {
                snapshot = Value::Array(
                    s.sections
                        .iter()
                        .map(|s| {
                            Value::Array(vec![
                                Value::String(s.key.clone()),
                                observer.cell(&s.value),
                            ])
                        })
                        .collect(),
                )
            })
        } else {
            let patch: Patch = fields(get(op, "patch"), &pool);
            for (i, name) in ["measure", "flat", "deviation", "thickness"]
                .iter()
                .enumerate()
            {
                if let Cell::Object(fields) = core::get(&patch, name) {
                    for (k, _) in fields.iter() {
                        if !keys[i].contains(k) {
                            keys[i].push(k.clone())
                        }
                    }
                }
            }
            type Apply = Box<dyn FnMut(&Cell, &Stores<'_>) -> Result<(), Text>>;
            let mut callbacks: Vec<Apply> = (0..names.len())
                .map(|i| {
                    let log = log.clone();
                    let slots = slots.clone();
                    let fault = fault.clone();
                    Box::new(move |value: &Cell, stores: &Stores<'_>| {
                        let name = format!("participant-{i}");
                        event(&log, &name, stores);
                        fail(&fault, &name)?;
                        slots.borrow_mut()[i] = value.clone();
                        Ok(())
                    }) as Apply
                })
                .collect();
            let mut participants: Vec<_> = names
                .iter()
                .zip(callbacks.iter_mut())
                .map(|(key, apply)| Participant {
                    key,
                    apply: apply.as_mut(),
                })
                .collect();
            core::apply(
                &mut stores,
                &patch,
                &mut participants,
                Limits::default(),
                |value, stores| {
                    event(&log, "scope-set", stores);
                    fail(&fault, "scope-set")?;
                    *scope.borrow_mut() = value;
                    fail(&fault, "scope-after")
                },
            )
        };
        let error = match result {
            Ok(()) => Value::Undefined,
            Err(CoreError::Callback(e)) => obj(vec![
                ("name", string("Error")),
                ("message", Value::String(e)),
            ]),
            Err(e) => panic!("unexpected host failure {e:?}"),
        };
        let stores = Value::Array(
            data.iter()
                .zip(&keys)
                .map(|(fields, keys)| {
                    Value::Array(
                        keys.iter()
                            .map(|k| {
                                let value = fields
                                    .iter()
                                    .find(|(name, _)| name == k)
                                    .map(|(_, v)| v)
                                    .unwrap_or(&Cell::Undefined);
                                Value::Array(vec![Value::String(k.clone()), observer.cell(value)])
                            })
                            .collect(),
                    )
                })
                .collect(),
        );
        frames.push(obj(vec![
            ("error", error),
            ("snapshot", snapshot),
            ("stores", stores),
            ("scope", observer.cell(&scope.borrow())),
            ("scan", observer.cell(&scan)),
            (
                "participants",
                Value::Array(slots.borrow().iter().map(|v| observer.cell(v)).collect()),
            ),
            (
                "log",
                Value::Array(
                    log.borrow()
                        .iter()
                        .map(|s| Value::String(s.clone()))
                        .collect(),
                ),
            ),
        ]));
    }
    let mut output = b"HCP1".to_vec();
    encode(&mut output, &Value::Array(frames));
    output
}
#[test]
fn complete_core_case_inputs_decode() {
    for c in cases() {
        assert!(payload(c.input).starts_with(b"HCP1"), "{}", c.id)
    }
}

fn change_version(value: &mut Value) -> bool {
    match value {
        Value::Array(a) => {
            for v in a {
                if let Value::Array(pair) = v {
                    if pair.len() == 2
                        && matches!(&pair[0],Value::String(k)if k.iter().copied().eq("subjectVersion".encode_utf16()))
                    {
                        if let Value::Number(n) = &mut pair[1] {
                            *n += 1.;
                            return true;
                        }
                    }
                }
                if change_version(v) {
                    return true;
                }
            }
            false
        }
        Value::Object(a) => a.iter_mut().any(|(_, v)| change_version(v)),
        _ => false,
    }
}
#[test]
#[ignore = "live original history recording comparison requires Node 22 and pinned TypeScript dependency"]
fn pinned_history_core_comparison() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root
        .join("rust/target/history-core-diff")
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
        "contract: engine.session.history.core.bytes.v1\nreference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncomparison: full patch graph identities/payloads, bookkeeping/workspace/capture order and held state\nseeds: 1,7,99,1039\n",
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
        .arg(root.join("docs/port-plan/oracle/build-history-core.mjs"))
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
    assert_eq!(cases.len(), 98);
    let mut input = Vec::new();
    for c in &cases {
        n(&mut input, c.input.len());
        input.extend(c.input);
    }
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/history-core.mjs"))
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
            "{}: complete core capture/apply fields, identity and state",
            c.id
        );
        if c.id == "families-15" {
            let mut r = Reader {
                b: &candidate[4..],
                at: 0,
            };
            let mut value = r.value();
            assert!(change_version(&mut value));
            let mut changed = b"HCP1".to_vec();
            encode(&mut changed, &value);
            assert_ne!(changed, observed, "actual version value mutation must fail");
            assert_ne!(
                &candidate[..candidate.len() - 1],
                observed,
                "missing state tail must fail"
            );
        }
        println!("{}: all original history state compared exactly", c.id);
    }
    assert_eq!(offset, original.stdout.len());
    println!(
        "98 complete core capture/apply scenarios; raw {}",
        run.display()
    );
}
