// SPDX-License-Identifier: AGPL-3.0-only
//! Full original Watchable interactions with independent native store probes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    watch_session::{Listener, Unsubscribe, WatchError, WatchStop, Watchable, watch_session},
    watch_stores::{StoreList, StoreSources},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::{Rc, Weak},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/watch-session/cases.tsv");
struct Fault(Vec<u16>);
type FaultRef = Rc<Fault>;
type ProbeListener = Listener<'static, FaultRef>;
struct State {
    log: Vec<String>,
    active: Vec<Vec<(usize, ProbeListener)>>,
    subs: [usize; 4],
    unsubs: BTreeMap<usize, usize>,
    serial: usize,
    listener_calls: usize,
    faults: Vec<FaultRef>,
    sub_fail: i32,
    stop_fail: i32,
    stop_mode: usize,
    immediate: usize,
    listener_fail: bool,
    mutate: bool,
    changed: bool,
    expected: Option<Weak<dyn Fn() -> Result<(), FaultRef>>>,
}
struct Store {
    id: usize,
    state: Rc<RefCell<State>>,
    source: Weak<StoreSources<Store>>,
    stores: Weak<RefCell<Vec<Rc<Store>>>>,
}
impl Watchable<'static, FaultRef> for Store {
    fn subscribe(
        self: Rc<Self>,
        listener: ProbeListener,
    ) -> Result<Unsubscribe<'static, FaultRef>, FaultRef> {
        let id = self.id;
        let (key, change, immediate) = {
            let mut s = self.state.borrow_mut();
            let same = Rc::ptr_eq(&listener, &s.expected.as_ref().unwrap().upgrade().unwrap());
            s.log.push(format!("sub:{id}:{}", usize::from(same)));
            s.subs[id] += 1;
            if s.sub_fail == id as i32 && s.subs[id] == 1 {
                return Err(s.faults[id].clone());
            }
            let key = s.serial;
            s.serial += 1;
            s.active[id].push((key, listener.clone()));
            let change = id == 0 && s.mutate && !s.changed;
            if change {
                s.changed = true
            }
            (key, change, s.immediate & (1 << id) != 0)
        };
        if change {
            drop(self.source.upgrade().unwrap().configure(Rc::from(vec![
                self.stores.upgrade().unwrap().borrow()[3].clone(),
            ])))
        }
        if immediate {
            listener()?
        }
        let state = self.state.clone();
        Ok(Rc::new(move || {
            let mut s = state.borrow_mut();
            s.log.push(format!("unsub:{id}:{key}"));
            let calls = s.unsubs.entry(key).or_default();
            *calls += 1;
            let calls = *calls;
            if s.stop_fail == id as i32 && (s.stop_mode == 2 || calls == 1) {
                return Err(s.faults[4 + id].clone());
            }
            s.active[id].retain(|(k, _)| *k != key);
            Ok(())
        }))
    }
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|n| format!("{n:04x}")).collect()
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let roles: Vec<Vec<usize>> = p[1..5]
        .iter()
        .map(|s| {
            if s.is_empty() {
                vec![]
            } else {
                s.split(',').map(|n| n.parse().unwrap()).collect()
            }
        })
        .collect();
    let v: Vec<i32> = p[5..].iter().map(|n| n.parse().unwrap()).collect();
    let faults: Vec<_> = (0..9)
        .map(|i| {
            Rc::new(Fault(
                if i < 4 {
                    format!("sub-{i}")
                } else if i < 8 {
                    format!("unsub-{}", i - 4)
                } else {
                    "listener".to_string()
                }
                .encode_utf16()
                .collect(),
            ))
        })
        .collect();
    let state = Rc::new(RefCell::new(State {
        log: vec![],
        active: (0..4).map(|_| vec![]).collect(),
        subs: [0; 4],
        unsubs: BTreeMap::new(),
        serial: 0,
        listener_calls: 0,
        faults: faults.clone(),
        sub_fail: v[0],
        stop_fail: v[1],
        stop_mode: v[2] as usize,
        immediate: v[3] as usize,
        listener_fail: v[4] != 0,
        mutate: v[5] != 0,
        changed: false,
        expected: None,
    }));
    let source = Rc::new(StoreSources::new());
    let stores = Rc::new(RefCell::new(Vec::new()));
    for id in 0..4 {
        stores.borrow_mut().push(Rc::new(Store {
            id,
            state: state.clone(),
            source: Rc::downgrade(&source),
            stores: Rc::downgrade(&stores),
        }))
    }
    let lists: Vec<StoreList<_>> = roles
        .iter()
        .map(|list| {
            Rc::from(
                list.iter()
                    .map(|&i| stores.borrow()[i].clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    drop(source.configure(lists[0].clone()));
    let listener: ProbeListener = {
        let state = state.clone();
        Rc::new(move || {
            let mut s = state.borrow_mut();
            s.listener_calls += 1;
            let n = s.listener_calls;
            s.log.push(format!("listener:{n}"));
            if s.listener_fail && n == 1 {
                Err(s.faults[8].clone())
            } else {
                Ok(())
            }
        })
    };
    state.borrow_mut().expected = Some(Rc::downgrade(&listener));
    let frame = |result: &str, stop: bool| {
        let root = source
            .current()
            .iter()
            .map(|s| s.id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let s = state.borrow();
        let active = s
            .active
            .iter()
            .map(|a| {
                a.iter()
                    .map(|(key, _)| key.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .collect::<Vec<_>>()
            .join("/");
        format!(
            "{result}|{}|{root}|{active}|{}",
            usize::from(stop),
            s.log.join(",")
        )
    };
    let mut stop: Option<WatchStop<'static, FaultRef>> = None;
    let mut frames = vec![frame("-", false)];
    for op in ["watch", "emit", "stop", "emit", "stop", "drop", "emit"] {
        let result: Result<(), FaultRef> = match op {
            "watch" => match watch_session(
                source.as_ref(),
                &lists[1],
                &lists[2..],
                listener.clone(),
                1024,
            ) {
                Ok(next) => {
                    stop = Some(next);
                    Ok(())
                }
                Err(WatchError::Callback(e)) => Err(e),
                Err(_) => panic!("host refusal"),
            },
            "stop" => {
                if let Some(stop) = &stop {
                    stop.stop()
                } else {
                    Ok(())
                }
            }
            "emit" => {
                let mut result = Ok(());
                for id in 0..4 {
                    let listeners = state.borrow().active[id]
                        .iter()
                        .map(|(_, l)| l.clone())
                        .collect::<Vec<_>>();
                    for callback in listeners {
                        if let Err(e) = callback() {
                            result = Err(e);
                            break;
                        }
                    }
                    if result.is_err() {
                        break;
                    }
                }
                result
            }
            _ => {
                stop = None;
                Ok(())
            }
        };
        let result = match result {
            Ok(()) => "ok".to_string(),
            Err(e) => format!(
                "throw:Error:{}:{}",
                hex(&e.0),
                faults.iter().position(|f| Rc::ptr_eq(f, &e)).unwrap()
            ),
        };
        frames.push(frame(&result, stop.is_some()));
    }
    // Close only the probe's observation scope, without invoking any source cleanup.
    state.borrow_mut().active.clear();
    format!("{}\t{}\n", p[0], frames.join(";"))
}
#[test]
fn pre_admission_and_duplicate_unsubscribe_functions() {
    struct Simple {
        calls: Rc<Cell<usize>>,
        stop: Unsubscribe<'static, ()>,
    }
    impl Watchable<'static, ()> for Simple {
        fn subscribe(
            self: Rc<Self>,
            _: Listener<'static, ()>,
        ) -> Result<Unsubscribe<'static, ()>, ()> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.stop.clone())
        }
    }
    let calls = Rc::new(Cell::new(0));
    let stopped = Rc::new(Cell::new(0));
    let shared: Unsubscribe<'static, ()> = {
        let s = stopped.clone();
        Rc::new(move || {
            s.set(s.get() + 1);
            Ok(())
        })
    };
    let a = Rc::new(Simple {
        calls: calls.clone(),
        stop: shared.clone(),
    });
    let b = Rc::new(Simple {
        calls: calls.clone(),
        stop: shared,
    });
    let source = StoreSources::new();
    drop(source.configure(Rc::from(vec![a, b])));
    let listener: Listener<'static, ()> = Rc::new(|| Ok(()));
    assert!(matches!(
        watch_session(&source, &[], &[], listener.clone(), 1),
        Err(WatchError::Limit)
    ));
    assert_eq!(calls.get(), 0);
    let stop = watch_session(&source, &[], &[], listener, 2).unwrap();
    stop.stop().unwrap();
    assert_eq!(stopped.get(), 2);
    stop.stop().unwrap();
    assert_eq!(stopped.get(), 4);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
fn borrowed_non_clone_subscribe_fault_survives() {
    #[derive(Debug, PartialEq, Eq)]
    struct Fault<'a>(&'a [u16]);
    struct Failing<'a>(&'a [u16]);
    impl<'a> Watchable<'a, Fault<'a>> for Failing<'a> {
        fn subscribe(
            self: Rc<Self>,
            _: Listener<'a, Fault<'a>>,
        ) -> Result<Unsubscribe<'a, Fault<'a>>, Fault<'a>> {
            Err(Fault(self.0))
        }
    }
    let data = [0xd800];
    let source = StoreSources::new();
    drop(source.configure(Rc::from(vec![Rc::new(Failing(&data))])));
    match watch_session(&source, &[], &[], Rc::new(|| Ok(())), 1) {
        Err(WatchError::Callback(e)) => assert_eq!(e, Fault(&data)),
        _ => panic!("expected unchanged source fault"),
    }
}
#[test]
#[ignore = "full original watchSession requires Node22 and original module"]
fn pinned_watch_session_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-watch-session.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/watch-session.mjs"))
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
    assert_eq!(originals.len(), 144);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/watch-session-diff/run-{run}"));
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
        "144 complete subscription/listener/cleanup state observations; {}",
        dir.display()
    );
}
