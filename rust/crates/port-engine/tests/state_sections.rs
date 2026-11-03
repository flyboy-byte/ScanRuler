// SPDX-License-Identifier: AGPL-3.0-only
//! Original live map, returned identity and ordinary-object readout traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    history_record::Cell as Value,
    state_sections::{Error, Fault, Limits, Prototype, Reader, Readout, Release, Sections},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/state-sections/cases.tsv");
#[derive(Debug)]
struct Opaque {
    _opaque: bool,
}
type E = Rc<Opaque>;
type Callback = Reader<'static, E>;
struct Env {
    logs: RefCell<Vec<String>>,
    stops: RefCell<Vec<Release<'static, E>>>,
    readers: RefCell<Vec<Callback>>,
    fired: Cell<bool>,
    fault: E,
    data: Value,
    b: Value,
    refs: Vec<(Value, &'static str)>,
}
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn object(fields: Vec<(Vec<u16>, Value)>) -> Value {
    Value::Object(Arc::new(fields))
}
fn show(e: &Env, value: &Value) -> String {
    if let Some((_, name)) = e.refs.iter().find(|(v, _)| v.strict_equal(value)) {
        return format!("@{name}");
    }
    match value {
        Value::Undefined => "u".into(),
        Value::Null => "n".into(),
        Value::Boolean(v) => format!("b{}", usize::from(*v)),
        Value::Number(n) => format!("d{n}"),
        Value::String(v) => format!("s{}", hex(v)),
        Value::Object(v) => format!(
            "o{{{}}}",
            v.iter()
                .map(|(k, v)| format!("{}:{}", hex(k), show(e, v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => panic!("unexpected unrecognized composite"),
    }
}
fn output(e: &Env, out: Readout) -> String {
    let fields = out
        .own()
        .iter()
        .map(|(k, v)| format!("{}={}", hex(k), show(e, v)))
        .collect::<Vec<_>>()
        .join(",");
    let prototype = match &out.prototype {
        Prototype::Default => "default".into(),
        Prototype::Null => "null".into(),
        Prototype::Data(v) => show(e, v),
    };
    format!("{fields}^{prototype}")
}
fn add(
    e: &Env,
    sections: &Sections<'static, E>,
    key: Rc<[u16]>,
    reader: Callback,
    brief: Option<Callback>,
) {
    match sections.register(key, reader, brief) {
        Ok(stop) => {
            e.stops.borrow_mut().push(stop);
            e.logs.borrow_mut().push("registered".into())
        }
        Err(Error::Duplicate(message)) => e
            .logs
            .borrow_mut()
            .push(format!("duplicate:{}", hex(&message))),
        _ => panic!("unexpected admission failure"),
    }
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mode: usize = p[1].parse().unwrap();
    let seed: usize = p[2].parse().unwrap();
    let keys = [
        text("a"),
        text("2"),
        text("__proto__"),
        text("constructor"),
        text(""),
        text("01"),
        text("4294967294"),
        vec![0xd800],
    ];
    let key: Rc<[u16]> = if mode >= 16 {
        text("__proto__").into()
    } else {
        keys[seed].clone().into()
    };
    let a = Sections::new(Limits {
        slots: 100,
        visits: 1000,
        depth: 20,
        key_units: 100,
    });
    let mut t = text("T");
    t.push(0xd800);
    let values = [
        Value::Null,
        Value::Boolean(true),
        Value::Number(37.0),
        Value::String(t),
        Value::Undefined,
        object(vec![(text("v"), Value::Number(seed as f64))]),
        Value::Array(Arc::new(vec![Value::Number(seed as f64)])),
        object(vec![(text("inherited"), Value::String(text("I")))]),
    ];
    let b = object(vec![(text("value"), Value::String(text("B")))]);
    let e = Rc::new(Env {
        logs: RefCell::new(vec![]),
        stops: RefCell::new(vec![]),
        readers: RefCell::new(vec![]),
        fired: Cell::new(false),
        fault: Rc::new(Opaque { _opaque: true }),
        data: values[if mode >= 16 { mode - 16 } else { seed }].clone(),
        b: b.clone(),
        refs: vec![
            (values[5].clone(), "obj"),
            (values[6].clone(), "arr"),
            (values[7].clone(), "proto"),
            (b, "B"),
        ],
    });
    for index in 0..3 {
        let weak = Rc::downgrade(&e);
        let key = key.clone();
        e.readers.borrow_mut().push(Rc::new(move |a| {
            let e = weak.upgrade().unwrap();
            e.logs.borrow_mut().push(['A', 'B', 'C'][index].to_string());
            if index == 0 && !e.fired.replace(true) {
                if mode == 2 {
                    e.stops.borrow()[1].release();
                    let reader = e.readers.borrow()[2].clone();
                    let stop = a
                        .register(text("c").into(), reader.clone(), Some(reader))
                        .unwrap();
                    e.stops.borrow_mut().push(stop)
                }
                if mode == 3 {
                    e.stops.borrow()[0].release();
                    let reader = e.readers.borrow()[0].clone();
                    let brief = e.readers.borrow()[2].clone();
                    let stop = a.register(key.clone(), reader, Some(brief)).unwrap();
                    e.stops.borrow_mut().push(stop)
                }
                if mode == 4 {
                    let stop = a
                        .register(text("c").into(), e.readers.borrow()[2].clone(), None)
                        .unwrap();
                    e.stops.borrow_mut().push(stop)
                }
            }
            if index == 0 && (mode == 8 || mode == 10) {
                return Err(Fault::Error(Box::new(move || {
                    e.logs.borrow_mut().push("message".into());
                    if mode == 10 {
                        Err(e.fault.clone())
                    } else {
                        Ok(e.data.clone())
                    }
                })));
            }
            if index == 0 && (mode == 9 || mode == 11) {
                return Err(Fault::Other(Box::new(move || {
                    e.logs.borrow_mut().push("string".into());
                    if mode == 11 {
                        Err(e.fault.clone())
                    } else {
                        let mut v = text("converted");
                        v.push(0xd800);
                        Ok(v)
                    }
                })));
            }
            Ok(match index {
                0 => e.data.clone(),
                1 => e.b.clone(),
                _ => Value::String(text("C")),
            })
        }))
    }
    add(
        &e,
        &a,
        key.clone(),
        e.readers.borrow()[0].clone(),
        if mode == 5 {
            None
        } else {
            Some(e.readers.borrow()[2].clone())
        },
    );
    add(
        &e,
        &a,
        text("b").into(),
        e.readers.borrow()[1].clone(),
        Some(e.readers.borrow()[1].clone()),
    );
    if mode == 1 {
        add(
            &e,
            &a,
            key.clone(),
            e.readers.borrow()[2].clone(),
            Some(e.readers.borrow()[2].clone()),
        )
    }
    if mode == 6 {
        e.stops.borrow()[0].release();
        add(&e, &a, key.clone(), e.readers.borrow()[0].clone(), None);
        e.stops.borrow()[0].release()
    }
    if mode == 7 {
        e.stops.borrow()[0].release();
        add(
            &e,
            &a,
            key.clone(),
            e.readers.borrow()[2].clone(),
            Some(e.readers.borrow()[2].clone()),
        );
        e.stops.borrow()[0].release()
    }
    if mode == 12 {
        e.stops.borrow_mut().pop();
    }
    if mode == 13 {
        e.stops.borrow()[1].release()
    }
    if mode == 14 {
        e.stops.borrow()[0].release();
        e.stops.borrow()[0].release()
    }
    if mode == 15 {
        add(
            &e,
            &a,
            text("0").into(),
            e.readers.borrow()[2].clone(),
            Some(e.readers.borrow()[2].clone()),
        )
    }
    for brief in [false, true, false] {
        let name = if brief { "brief" } else { "read" };
        let result = if brief { a.briefs() } else { a.read() };
        let record = match result {
            Ok(out) => format!("{name}:{}", output(&e, out)),
            Err(Error::Conversion(f)) => {
                assert!(Rc::ptr_eq(&f, &e.fault));
                format!("{name}:fault:same")
            }
            other => panic!("host error {other:?}"),
        };
        e.logs.borrow_mut().push(record)
    }
    format!("{}\t{}\n", p[0], e.logs.borrow().join("|"))
}
#[test]
fn all_authored_traces_and_owned_limits() {
    for line in INPUT.lines() {
        assert!(observe(line).contains("read:"))
    }
    let a: Sections<'_, ()> = Sections::new(Limits {
        slots: 1,
        visits: 0,
        depth: 1,
        key_units: 3,
    });
    let reader: Reader<'_, ()> = Rc::new(|_| Ok(Value::Null));
    assert!(matches!(
        a.register(text("long").into(), reader.clone(), None),
        Err(Error::Limit)
    ));
    drop(a.register(text("a").into(), reader.clone(), None).unwrap());
    assert!(matches!(
        a.register(text("b").into(), reader, None),
        Err(Error::Limit)
    ));
    assert!(matches!(a.read(), Err(Error::Limit)));
}

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
#[test]
#[ignore = "full original state-sections module requires Node22 and original graph"]
fn pinned_state_sections_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-state-sections.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/state-sections.mjs"))
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
    assert_eq!(originals.len(), 192);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/state-sections-diff/run-{run}"));
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
        "192 complete state section/map/prototype/error observations; {}",
        dir.display()
    );
}

#[test]
fn borrowed_conversion_fault_and_reentrant_read_bounds() {
    #[derive(Debug, PartialEq, Eq)]
    struct Borrowed<'a>(&'a [u16]);
    let raw = [0xd800];
    let a = Sections::new(Limits {
        slots: 2,
        visits: 10,
        depth: 2,
        key_units: 10,
    });
    let reader: Reader<'_, Borrowed<'_>> =
        Rc::new(|_| Err(Fault::Other(Box::new(|| Err(Borrowed(&raw))))));
    let stop = a.register(text("a").into(), reader, None).unwrap();
    assert!(
        matches!(a.read(),Err(Error::Conversion(Borrowed(value)))if std::ptr::eq(value,raw.as_slice()))
    );
    stop.release();
    assert!(a.read().unwrap().own().is_empty());
    let a: Sections<'_, ()> = Sections::new(Limits {
        slots: 2,
        visits: 10,
        depth: 1,
        key_units: 10,
    });
    let reader: Reader<'_, ()> = Rc::new(|a| {
        assert!(matches!(a.briefs(), Err(Error::Limit)));
        Ok(Value::Null)
    });
    drop(a.register(text("a").into(), reader, None).unwrap());
    assert_eq!(a.read().unwrap().own().len(), 1);
}
