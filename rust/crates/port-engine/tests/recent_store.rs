// SPDX-License-Identifier: AGPL-3.0-only
//! Complete original recent records, observer/caller separation and shallow copy traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    error::{CommandError, CommandErrorCode},
    recent_result,
    recent_store::{Error, Limits, Recent, ResultValue, Snapshot, State, Ticket},
    schema_value::Value as V,
};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
    sync::Arc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/recent-store/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
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
fn data(mode: usize, i: usize) -> V {
    match mode {
        3 => V::Undefined,
        4 => V::String(vec![120; 16000]),
        5 => V::Bytes(vec![0; 23]),
        8 => V::Array(vec![
            V::Number(i as f64),
            V::Object(vec![(text("bytes"), V::Bytes(vec![0; 3]))]),
        ]),
        9 => V::Null,
        10 => V::Number(-0.0),
        _ => V::Object(vec![
            (text("n"), V::Number(i as f64)),
            (
                text("nested"),
                V::Object(vec![
                    (text("u"), V::Undefined),
                    (text("bytes"), V::Bytes(vec![0; 3])),
                ]),
            ),
        ]),
    }
}
fn limits() -> Limits {
    Limits {
        pending: 256,
        text_units: 1024,
        data: recent_result::Limits {
            nodes: 4096,
            depth: 64,
            input_units: 4_000_000,
            json_units: 32_000_000,
        },
    }
}
fn same_result(a: &Option<ResultValue>, b: &Option<ResultValue>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(ResultValue::Primitive(a)), Some(ResultValue::Primitive(b))) => a == b,
        (Some(ResultValue::Array(a)), Some(ResultValue::Array(b))) => Rc::ptr_eq(a, b),
        (Some(ResultValue::Object(a)), Some(ResultValue::Object(b))) => Rc::ptr_eq(a, b),
        _ => false,
    }
}
// Independent wire projection lists its observed states explicitly.
#[allow(clippy::too_many_arguments)]
fn frame(
    recent: &Recent,
    frozen: &Option<Snapshot>,
    edited: &Option<Snapshot>,
    tickets: &[Option<Ticket>],
    identities: &[String],
    callers: &[&str],
    log: &RefCell<Vec<String>>,
    unhandled: usize,
) -> String {
    let all = recent.outcomes(None).unwrap();
    let filtered = V::Array(
        ["jobA", "", "missing"]
            .iter()
            .map(|j| {
                V::Array(
                    recent
                        .outcomes(Some(&text(j)))
                        .unwrap()
                        .into_iter()
                        .map(|o| V::String(o.name.to_vec()))
                        .collect(),
                )
            })
            .collect(),
    );
    let alias = match (edited, all.first()) {
        (Some(e), Some(o)) => usize::from(
            same_result(&e.result, &o.result)
                && match (&e.error, &o.error) {
                    (None, None) => true,
                    (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                },
        )
        .to_string(),
        _ => "-".into(),
    };
    let first = tickets
        .first()
        .and_then(|t| t.as_ref())
        .map(|t| wire(&snapshot_value(&t.snapshot())))
        .unwrap_or_else(|| "-".into());
    [
        wire(&V::Array(all.iter().map(snapshot_value).collect())),
        wire(&filtered),
        frozen
            .as_ref()
            .map(|s| wire(&snapshot_value(s)))
            .unwrap_or_else(|| "-".into()),
        edited
            .as_ref()
            .map(|s| wire(&snapshot_value(s)))
            .unwrap_or_else(|| "-".into()),
        identities.join(","),
        callers.join(","),
        log.borrow().join(","),
        unhandled.to_string(),
        alias,
        first,
    ]
    .join("|")
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mode: usize = p[1].parse().unwrap();
    let start: usize = p[2].parse().unwrap();
    let end: usize = p[3].parse().unwrap();
    let times = [
        0.0,
        -0.9,
        0.9,
        -1.9,
        1.9,
        8_640_000_000_000_000.0,
        -8_640_000_000_000_000.0,
        8_640_000_000_000_001.0,
        f64::NAN,
        f64::INFINITY,
    ];
    let since = times[start];
    let recent = Recent::new(limits());
    let log = RefCell::new(Vec::new());
    let (mut identities, mut callers, mut tickets, mut values, mut frames) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut frozen, mut edited) = (None, None);
    let mut unhandled = 0;
    let total = if mode == 6 || mode == 7 { 105 } else { 1 };
    for i in 0..total {
        let job = match i % 3 {
            0 => text("jobA"),
            1 => vec![],
            _ => vec![106, 111, 98, 0xd800],
        };
        let mut name = text(&format!("cmd{i}💩"));
        name.push(0xd800);
        let run = Rc::new(i);
        let (returned, ticket) = recent.remember(&job, &name, run.clone(), || {
            log.borrow_mut().push("clock:start".into());
            Ok::<_, &str>(since)
        });
        match ticket {
            Ok(t) => {
                identities.push(usize::from(Rc::ptr_eq(&run, &returned)).to_string());
                tickets.push(Some(t));
            }
            Err(Error::Date(e)) => {
                let (name, msg) = e.source_error().unwrap();
                identities.push(format!("{name}:{msg}"));
                tickets.push(None)
            }
            _ => panic!("host/source error"),
        };
        callers.push("pending");
        values.push(data(mode, i));
        if i == 0 {
            frozen = recent.outcomes(None).unwrap().first().cloned();
            frames.push(frame(
                &recent,
                &frozen,
                &edited,
                &tickets,
                &identities,
                &callers,
                &log,
                unhandled,
            ))
        }
    }
    frames.push(frame(
        &recent,
        &frozen,
        &edited,
        &tickets,
        &identities,
        &callers,
        &log,
        unhandled,
    ));
    frames.push(frame(
        &recent,
        &frozen,
        &edited,
        &tickets,
        &identities,
        &callers,
        &log,
        unhandled,
    ));
    let now = match end {
        0 => since + 17.5,
        1 => since - 1.0,
        _ => f64::NAN,
    };
    let events = if total == 1 {
        vec![(0, mode == 1 || mode == 2)]
    } else {
        vec![(0, true), (104, mode == 7), (99, false)]
    };
    for (i, failed) in events {
        log.borrow_mut().push(format!("finish:{i}"));
        frames.push(frame(
            &recent,
            &frozen,
            &edited,
            &tickets,
            &identities,
            &callers,
            &log,
            unhandled,
        ));
        if let Some(ticket) = tickets[i].as_mut() {
            let clock = || {
                log.borrow_mut().push("clock:end".into());
                if end == 2 { Err("clock") } else { Ok(now) }
            };
            let result = if failed {
                ticket.fail(clock, || {
                    log.borrow_mut().push(format!("normalize:{i}"));
                    if mode == 2 {
                        return Err("normalize");
                    }
                    let message = [text("fail "), vec![0xd800]].concat();
                    Ok(Arc::new(
                        CommandError::from_utf16(CommandErrorCode::ALL[i % 12], &message, 1024)
                            .unwrap(),
                    ))
                })
            } else {
                ticket.complete(&values[i], clock)
            };
            if result.is_err() {
                unhandled += 1
            }
        }
        callers[i] = if failed { "err" } else { "ok" };
        frames.push(frame(
            &recent,
            &frozen,
            &edited,
            &tickets,
            &identities,
            &callers,
            &log,
            unhandled,
        ));
    }
    let all = recent.outcomes(None).unwrap();
    if let Some(first) = all.first() {
        let mut copy = first.clone();
        copy.job = Rc::from(text("copy"));
        copy.state = State::Failed;
        copy.ms = Some(-123.0);
        match &mut copy.result {
            Some(ResultValue::Array(items)) => items.borrow_mut().push(V::Number(999.0)),
            Some(ResultValue::Object(fields)) => {
                let mut fields = fields.borrow_mut();
                if let Some((_, v)) = fields.iter_mut().find(|(k, _)| *k == text("n")) {
                    *v = V::Number(999.0)
                } else {
                    fields.push((text("n"), V::Number(999.0)))
                }
            }
            Some(ResultValue::Primitive(_)) => {
                copy.result = Some(ResultValue::Primitive(V::String(text("copy primitive"))))
            }
            None => {}
        }
        if let Some(error) = &copy.error {
            error.borrow_mut().message = Rc::from(text("changed"))
        }
        edited = Some(copy);
        frames.push(frame(
            &recent,
            &frozen,
            &edited,
            &tickets,
            &identities,
            &callers,
            &log,
            unhandled,
        ));
    }
    frames.push(frame(
        &recent,
        &frozen,
        &edited,
        &tickets,
        &identities,
        &callers,
        &log,
        unhandled,
    ));
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn independent_recent_schedules_execute() {
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}

#[test]
fn source_callback_failures_leave_running_and_do_not_change_caller_run() {
    let recent = Recent::new(limits());
    let run = Rc::new(String::from("opaque run"));
    let (returned, ticket) =
        recent.remember(&[], &text("name"), run.clone(), || Ok::<_, &str>(1.9));
    assert!(Rc::ptr_eq(&run, &returned));
    let mut ticket = ticket.unwrap();
    assert_eq!(
        ticket.complete(&V::Null, || Err("clock")),
        Err(Error::Clock("clock"))
    );
    let s = ticket.snapshot();
    assert_eq!(s.state, State::Running);
    assert!(s.ms.is_none() && s.result.is_none() && s.error.is_none());
    assert_eq!(
        ticket.complete(&V::Null, || Ok::<_, &str>(2.0)),
        Err(Error::Completed)
    );
    assert_eq!(returned.as_str(), "opaque run");
    let (_, ticket) = recent.remember(&[], &text("next"), (), || Ok::<_, &str>(0.0));
    let mut ticket = ticket.unwrap();
    let order = RefCell::new(Vec::new());
    assert_eq!(
        ticket.fail(
            || {
                order.borrow_mut().push("clock");
                Ok(10.0)
            },
            || {
                order.borrow_mut().push("normalize");
                Err::<Arc<CommandError>, _>("normalize")
            }
        ),
        Err(Error::Normalize("normalize"))
    );
    assert_eq!(*order.borrow(), vec!["clock", "normalize"]);
    assert_eq!(ticket.snapshot().state, State::Running);
    assert!(ticket.snapshot().ms.is_none());
}
#[test]
fn admission_keeps_nonclone_run_and_abandoned_reactions_are_separate() {
    struct Run<'a>(&'a str);
    let text_value = String::from("borrowed");
    let mut l = limits();
    l.pending = 0;
    let recent = Recent::new(l);
    let (run, result) = recent.remember(&[], &[], Run(&text_value), || Ok::<_, ()>(0.0));
    assert_eq!(run.0.as_ptr(), text_value.as_ptr());
    assert!(matches!(result, Err(Error::Limit)));
    assert!(recent.outcomes(None).unwrap().is_empty());
    let recent = Recent::new(limits());
    let (_, ticket) = recent.remember(&[], &[], (), || Ok::<_, ()>(f64::NAN));
    assert!(matches!(ticket, Err(Error::Date(_))));
    assert!(recent.outcomes(None).unwrap().is_empty());
    let (_, ticket) = recent.remember(&[], &[], (), || Ok::<_, ()>(0.0));
    drop(ticket.unwrap());
    let (run, result) = recent.remember(&[], &[], Run(&text_value), || Ok::<_, ()>(0.0));
    assert_eq!(run.0.as_ptr(), text_value.as_ptr());
    assert!(matches!(result, Err(Error::Abandoned)));
    assert_eq!(recent.outcomes(None).unwrap()[0].state, State::Running);
}
#[test]
fn data_host_refusal_is_not_an_omitted_or_successful_result() {
    let mut l = limits();
    l.data.json_units = 1;
    let recent = Recent::new(l);
    let (_, ticket) = recent.remember(&[], &[], (), || Ok::<_, ()>(0.0));
    let mut ticket = ticket.unwrap();
    assert_eq!(
        ticket.complete(&V::Null, || Ok::<_, ()>(3.0)),
        Err(Error::Data(recent_result::Error::Limit))
    );
    let snapshot = ticket.snapshot();
    assert_eq!(snapshot.state, State::Running);
    assert!(
        snapshot.result.is_none() && snapshot.result_omitted.is_none() && snapshot.ms.is_none()
    );
}
#[test]
#[ignore = "complete original recent outcome state requires Node22 and original graph"]
fn pinned_recent_store_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-recent-store.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/recent-store.mjs"))
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
    assert_eq!(originals.len(), 362);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/recent-store-diff/run-{run}"));
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
        "362 complete recent record/reaction/shallow/eviction observations; {}",
        dir.display()
    );
}
