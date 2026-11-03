// SPDX-License-Identifier: AGPL-3.0-only
//! Complete state/identity/listener traces against the pinned original store.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::activity_state::{
    Activity, Error, Limits, Listener, Patch, Release, Running, State,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/activity-state/cases.tsv");
#[derive(Debug)]
struct Fault {
    _opaque: bool,
}
type Callback = Listener<'static, Rc<Fault>>;
type Stop = Release<'static, Rc<Fault>>;
enum Identity {
    State(Rc<State>),
    Running(Rc<Running>),
}
struct Env {
    ids: RefCell<Vec<Identity>>,
    logs: RefCell<Vec<String>>,
    held: RefCell<Vec<Rc<State>>>,
    stops: RefCell<Vec<Option<Stop>>>,
    callbacks: RefCell<Vec<Callback>>,
    fired: Cell<bool>,
    fault: Rc<Fault>,
}
fn state_id(e: &Env, state: &Rc<State>) -> usize {
    let mut ids = e.ids.borrow_mut();
    if let Some(i) = ids
        .iter()
        .position(|x| matches!(x,Identity::State(s) if Rc::ptr_eq(s,state)))
    {
        return i + 1;
    }
    ids.push(Identity::State(state.clone()));
    ids.len()
}
fn running_id(e: &Env, r: &Rc<Running>) -> usize {
    let mut ids = e.ids.borrow_mut();
    if let Some(i) = ids
        .iter()
        .position(|x| matches!(x,Identity::Running(s) if Rc::ptr_eq(s,r)))
    {
        return i + 1;
    }
    ids.push(Identity::Running(r.clone()));
    ids.len()
}
fn hex(v: &[u16]) -> String {
    v.iter().map(|n| format!("{n:04x}")).collect()
}
fn text(v: &str) -> Rc<[u16]> {
    v.encode_utf16().collect::<Vec<_>>().into()
}
fn show(e: &Env, state: &Rc<State>) -> String {
    let id = state_id(e, state);
    let running = state.running.as_ref().map_or_else(
        || "-".into(),
        |r| {
            format!(
                "{},{},{},{}",
                running_id(e, r),
                hex(&r.name),
                hex(&r.title),
                r.since
                    .to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
        },
    );
    let progress = state
        .progress
        .as_ref()
        .map_or_else(|| "-".into(), |v| hex(v));
    format!("{id}:{running}:{progress}")
}
fn take(e: &Env, a: &Activity<'_, Rc<Fault>>) {
    let state = a.current();
    e.held.borrow_mut().push(state.clone());
    let record = format!("S:{}:{}", show(e, &state), usize::from(a.command_running()));
    e.logs.borrow_mut().push(record)
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mode: usize = p[1].parse().unwrap();
    let seed: usize = p[2].parse().unwrap();
    let a = Activity::new(Limits {
        slots: 100,
        visits: 1000,
        depth: 20,
        text_units: 100,
    });
    let e = Rc::new(Env {
        ids: RefCell::new(vec![]),
        logs: RefCell::new(vec![]),
        held: RefCell::new(vec![]),
        stops: RefCell::new(vec![]),
        callbacks: RefCell::new(vec![]),
        fired: Cell::new(false),
        fault: Rc::new(Fault { _opaque: true }),
    });
    let mut progress = format!("P{seed}").encode_utf16().collect::<Vec<_>>();
    progress.push(0xd800);
    let progress: Rc<[u16]> = if mode == 12 {
        text("")
    } else {
        progress.into()
    };
    for index in 0..3 {
        let weak = Rc::downgrade(&e);
        let progress = progress.clone();
        e.callbacks
            .borrow_mut()
            .push(Rc::new(move |a, state, previous| {
                let e = weak.upgrade().unwrap();
                let record = format!(
                    "{}:{}<{}@{}:{}",
                    ['A', 'B', 'C'][index],
                    show(&e, &state),
                    show(&e, &previous),
                    state_id(&e, &a.current()),
                    usize::from(a.command_running())
                );
                e.logs.borrow_mut().push(record);
                if (index == 1 && mode == 13) || (index == 2 && mode == 14) {
                    return Err(e.fault.clone());
                }
                if index != 0 || e.fired.replace(true) {
                    return Ok(());
                }
                if mode == 2 {
                    e.stops.borrow()[1].as_ref().unwrap().release();
                    let stop = a.subscribe(e.callbacks.borrow()[2].clone()).unwrap();
                    e.stops.borrow_mut().push(Some(stop));
                }
                if mode == 3 {
                    e.stops.borrow()[0].as_ref().unwrap().release();
                    let stop = a.subscribe(e.callbacks.borrow()[0].clone()).unwrap();
                    e.stops.borrow_mut().push(Some(stop));
                }
                if mode == 4 {
                    return Err(e.fault.clone());
                }
                if mode == 5 || mode == 13 {
                    a.publish(Patch::Progress(progress.clone()))
                        .map_err(|error| match error {
                            Error::Listener(f) => f,
                            other => panic!("host error {other:?}"),
                        })?;
                }
                if mode == 14 {
                    let stop = a.subscribe(e.callbacks.borrow()[2].clone()).unwrap();
                    e.stops.borrow_mut().push(Some(stop));
                }
                if mode == 15 {
                    e.stops.borrow()[1].as_ref().unwrap().release();
                    let stop = a.subscribe(e.callbacks.borrow()[1].clone()).unwrap();
                    e.stops.borrow_mut().push(Some(stop));
                }
                if mode == 8 {
                    a.publish(Patch::Clear).unwrap()
                }
                Ok(())
            }));
    }
    take(&e, &a);
    if mode != 0 {
        for i in 0..2 {
            e.stops
                .borrow_mut()
                .push(Some(a.subscribe(e.callbacks.borrow()[i].clone()).unwrap()))
        }
    }
    if mode == 1 {
        e.stops
            .borrow_mut()
            .push(Some(a.subscribe(e.callbacks.borrow()[0].clone()).unwrap()))
    }
    if mode == 6 {
        e.stops.borrow()[0].as_ref().unwrap().release();
        e.stops
            .borrow_mut()
            .push(Some(a.subscribe(e.callbacks.borrow()[0].clone()).unwrap()));
        e.stops.borrow()[0].as_ref().unwrap().release();
    }
    if mode == 7 {
        e.stops.borrow_mut().pop();
    }
    let run = |patch| {
        let status = match a.publish(patch) {
            Ok(()) => "ok",
            Err(Error::Listener(f)) => {
                assert!(Rc::ptr_eq(&f, &e.fault));
                "fault:same"
            }
            other => panic!("unexpected host error {other:?}"),
        };
        e.logs.borrow_mut().push(status.into());
        take(&e, &a);
    };
    let name: Rc<[u16]> = if seed % 2 == 1 {
        text("")
    } else {
        let mut v = "run".encode_utf16().collect::<Vec<_>>();
        v.push(0xd800);
        v.into()
    };
    let since = [
        0.0,
        -0.0,
        1.5,
        -37.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        8640000000000001.0,
    ][seed % 8];
    run(Patch::Start(Rc::new(Running {
        name,
        title: text(&format!("ΟΣ{seed}")),
        since,
    })));
    if mode == 4 {
        e.stops.borrow()[0].as_ref().unwrap().release()
    }
    run(Patch::Progress(progress.clone()));
    run(Patch::Progress(progress));
    run(Patch::Clear);
    run(Patch::Clear);
    let held = e
        .held
        .borrow()
        .iter()
        .map(|s| show(&e, s))
        .collect::<Vec<_>>()
        .join("/");
    e.logs.borrow_mut().push(format!("held:{held}"));
    format!("{}\t{}\n", p[0], e.logs.borrow().join("|"))
}
#[test]
fn source_traces_are_finite_and_host_refusals_preserve_published_state() {
    for line in INPUT.lines() {
        assert!(observe(line).contains("held:"))
    }
    let a: Activity<'_, ()> = Activity::new(Limits {
        slots: 1,
        visits: 0,
        depth: 2,
        text_units: 2,
    });
    let initial = a.initial();
    let callback: Listener<'_, ()> = Rc::new(|_, _, _| Ok(()));
    drop(a.subscribe(callback.clone()).unwrap());
    assert!(a.subscribe(callback).is_ok());
    assert!(matches!(
        a.subscribe(Rc::new(|_, _, _| Ok(()))),
        Err(Error::Limit)
    ));
    assert_eq!(a.publish(Patch::Progress(text("abc"))), Err(Error::Limit));
    assert!(Rc::ptr_eq(&a.current(), &initial));
    assert_eq!(a.publish(Patch::Progress(text(""))), Err(Error::Limit));
    assert!(!Rc::ptr_eq(&a.current(), &initial));
    assert_eq!(a.current().progress.as_deref(), Some(&[][..]));
}

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
#[test]
#[ignore = "full original activity module requires Node22 and original graph"]
fn pinned_activity_state_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-activity-state.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/activity-state.mjs"))
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
    assert_eq!(originals.len(), 256);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/activity-state-diff/run-{run}"));
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
        "256 complete activity snapshot/listener observations; {}",
        dir.display()
    );
}

#[test]
fn borrowed_fault_and_recursive_bounds_do_not_normalize_or_reset_state() {
    #[derive(Debug, PartialEq, Eq)]
    struct Borrowed<'a>(&'a [u16]);
    let value = [0xd800];
    let a = Activity::new(Limits {
        slots: 2,
        visits: 10,
        depth: 2,
        text_units: 10,
    });
    let callback: Listener<'_, Borrowed<'_>> = Rc::new(|_, _, _| Err(Borrowed(&value)));
    let stop = a.subscribe(callback).unwrap();
    assert_eq!(
        a.publish(Patch::Progress(text(""))),
        Err(Error::Listener(Borrowed(&value)))
    );
    assert_eq!(a.current().progress.as_deref(), Some(&[][..]));
    stop.release();
    assert_eq!(a.publish(Patch::Clear), Ok(()));
    let a: Activity<'_, ()> = Activity::new(Limits {
        slots: 2,
        visits: 10,
        depth: 1,
        text_units: 10,
    });
    drop(
        a.subscribe(Rc::new(|a, state, _| {
            assert_eq!(a.publish(Patch::Clear), Err(Error::Limit));
            assert!(Rc::ptr_eq(&a.current(), &state));
            Ok(())
        }))
        .unwrap(),
    );
    assert_eq!(a.publish(Patch::Progress(text("x"))), Ok(()));
    let a: Activity<'_, ()> = Activity::new(Limits {
        slots: 2,
        visits: 1,
        depth: 3,
        text_units: 10,
    });
    drop(
        a.subscribe(Rc::new(|a, _, _| {
            assert_eq!(
                a.publish(Patch::Progress(text("nested"))),
                Err(Error::Limit)
            );
            Ok(())
        }))
        .unwrap(),
    );
    assert_eq!(a.publish(Patch::Clear), Ok(()));
    assert_eq!(
        a.current().progress.as_deref(),
        Some(&"nested".encode_utf16().collect::<Vec<_>>()[..])
    );
}
