// SPDX-License-Identifier: AGPL-3.0-only
//! Complete independently driven pinned limited/inScene traces through retained native futures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    command_deadline::Completion,
    error::CommandError,
    reactions::{self, Limited},
    scene_queue::SceneQueue,
    tasks::{Limits, Owner},
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    io::Write,
    path::Path,
    pin::Pin,
    process::{Command, Stdio},
    rc::Rc,
    sync::Arc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/runtime-reactions/cases.tsv");
#[derive(Debug, Clone)]
enum Fault {
    Command(Arc<CommandError>),
    Plain(&'static str),
}
#[derive(Debug, Clone)]
enum State {
    Pending,
    Ok(&'static str),
    Error(Fault),
}
type Body = Pin<Box<dyn Future<Output = Result<&'static str, Fault>>>>;
struct World {
    owner: Owner<'static>,
    limited: RefCell<Option<Limited<&'static str, Fault>>>,
    states: RefCell<[State; 3]>,
    body: RefCell<State>,
    doc: Cell<i32>,
    log: RefCell<Vec<String>>,
}
fn hex(s: impl Iterator<Item = u16>) -> String {
    s.map(|v| format!("{v:04x}")).collect()
}
impl World {
    fn error(&self, error: &Fault) -> String {
        match error {
            Fault::Command(e) => {
                let same = self
                    .limited
                    .borrow()
                    .as_ref()
                    .and_then(Limited::abort_reason)
                    .is_some_and(|reason| Arc::ptr_eq(e, &reason));
                format!(
                    "error:{}:{}:{}:{}",
                    hex(e.name().encode_utf16()),
                    hex(e.code().as_str().encode_utf16()),
                    hex(e.message_units().iter().copied()),
                    usize::from(same)
                )
            }
            Fault::Plain(message) => format!(
                "error:{}::{}:0",
                hex("Error".encode_utf16()),
                hex(message.encode_utf16())
            ),
        }
    }
    fn render(&self, state: &State) -> String {
        match state {
            State::Pending => "pending".into(),
            State::Ok(v) => format!("ok:{v}"),
            State::Error(e) => self.error(e),
        }
    }
    fn frame(&self) -> String {
        let active = self
            .limited
            .borrow()
            .as_ref()
            .is_some_and(Limited::timer_active);
        let aborted = self
            .limited
            .borrow()
            .as_ref()
            .is_some_and(|l| l.abort_reason().is_some());
        format!(
            "{}|{}|{}|{}|{}|{}|0",
            self.states
                .borrow()
                .iter()
                .map(|s| self.render(s))
                .collect::<Vec<_>>()
                .join(","),
            self.render(&self.body.borrow()),
            self.doc.get(),
            usize::from(aborted),
            usize::from(active),
            self.log.borrow().join(",")
        )
    }
    fn settle(&self, result: Result<&'static str, Fault>) {
        if !matches!(*self.body.borrow(), State::Pending) {
            return;
        }
        *self.body.borrow_mut() = match &result {
            Ok(v) => State::Ok(v),
            Err(e) => State::Error(e.clone()),
        };
        self.doc.set(if result.is_ok() { 42 } else { -1 });
        self.log.borrow_mut().push(
            if result.is_ok() {
                "body:ok"
            } else {
                "body:error"
            }
            .into(),
        );
        self.limited.borrow().as_ref().unwrap().body_settled(result);
    }
    fn fire(&self, policy: usize) {
        let limited = self.limited.borrow().as_ref().unwrap().clone();
        if !limited.timer_active() {
            self.log.borrow_mut().push("timer:ignored".into());
            return;
        }
        self.log.borrow_mut().push("timer:fire".into());
        let event = limited.fire_begin().unwrap().unwrap();
        self.log.borrow_mut().push("abort".into());
        match policy {
            0 => {}
            1 => self.settle(Ok("fallback")),
            2 => self.settle(Err(Fault::Command(event.reason().clone()))),
            _ => panic!(),
        }
        limited.fire_end(event).unwrap();
    }
    fn flush(&self) {
        let turn = self.owner.turn().unwrap();
        assert_eq!(
            turn.ready, 0,
            "admitted turn must reach quiescence without guessed flush counts"
        );
    }
}
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let (policy, late, drop_ticket) = (parts[1].parse().unwrap(), parts[2] == "1", parts[3] == "1");
    let world = Rc::new(World {
        owner: Owner::new(Limits {
            tasks: 16,
            depth: 8,
            polls_per_turn: 128,
        })
        .unwrap(),
        limited: RefCell::new(None),
        states: RefCell::new([State::Pending, State::Pending, State::Pending]),
        body: RefCell::new(State::Pending),
        doc: Cell::new(0),
        log: RefCell::new(Vec::new()),
    });
    let queue = SceneQueue::new(8);
    let mut tickets = Vec::new();
    for i in 0..3 {
        let w = world.clone();
        let route = queue
            .run(true, move || -> Result<Body, Fault> {
                if i == 1 {
                    w.log.borrow_mut().push("second:start".into());
                    return Err(Fault::Plain("second failed"));
                }
                if i == 2 {
                    w.log.borrow_mut().push("third:start".into());
                    return Ok(Box::pin(std::future::ready(Ok("third"))));
                }
                w.log.borrow_mut().push("first:start".into());
                let weak = Rc::downgrade(&w);
                let limited = Limited::new(
                    &w.owner,
                    &"first".encode_utf16().collect::<Vec<_>>(),
                    10.,
                    1024,
                    move |_| {
                        weak.upgrade()
                            .unwrap()
                            .log
                            .borrow_mut()
                            .push("timer:clear".into());
                    },
                )
                .unwrap();
                limited.registered(1).unwrap();
                w.log.borrow_mut().push("timer:set".into());
                w.log.borrow_mut().push("body:start".into());
                let caller = limited.observe().unwrap();
                *w.limited.borrow_mut() = Some(limited.clone());
                limited.factory_returned().unwrap();
                Ok(Box::pin(async move {
                    match caller.await.unwrap() {
                        Completion::Body(result) => result,
                        Completion::Timeout(e) => Err(Fault::Command(e)),
                    }
                }))
            })
            .unwrap();
        let receiver = world.clone();
        let ticket = reactions::retain(
            &world.owner,
            move || route,
            move |outcome| {
                let state = match outcome {
                    Ok(v) => State::Ok(v),
                    Err(port_engine::import_queue::QueueError::Callback(e)) => State::Error(e),
                    _ => panic!("host queue failure"),
                };
                receiver.log.borrow_mut().push(format!(
                    "reply:{i}:{}",
                    if matches!(state, State::Ok(_)) {
                        "ok"
                    } else {
                        "error"
                    }
                ));
                receiver.states.borrow_mut()[i] = state;
            },
        )
        .unwrap();
        if drop_ticket {
            drop(ticket)
        } else {
            tickets.push(ticket)
        }
    }
    let mut frames = vec![world.frame()];
    world.flush();
    frames.push(world.frame());
    world.fire(policy);
    world.flush();
    frames.push(world.frame());
    // Both later callers must have answers while an ignored-abort body is still pending.
    assert!(
        world
            .states
            .borrow()
            .iter()
            .all(|s| !matches!(s, State::Pending))
    );
    if policy == 0 {
        assert!(matches!(*world.body.borrow(), State::Pending));
        assert_eq!(world.owner.active(), 1);
    }
    world.settle(if late {
        Err(Fault::Plain("late failed"))
    } else {
        Ok("late")
    });
    world.flush();
    frames.push(world.frame());
    world.fire(policy);
    world.flush();
    frames.push(world.frame());
    assert!(tickets.iter().all(|t| t.complete()));
    world.owner.finish().unwrap();
    format!("{}\t{}\n", parts[0], frames.join(";"))
}
#[test]
fn native_timeout_releases_real_scene_queue_without_dropping_late_body() {
    assert_eq!(INPUT.lines().count(), 12);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
}
#[test]
#[ignore = "live retained queue/deadline comparison requires Node22 and pinned original"]
fn pinned_runtime_reactions_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/runtime-reactions-diff/run-{stamp}"));
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
        .arg(root.join("docs/port-plan/oracle/build-reactions.mjs"))
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
        .arg(root.join("docs/port-plan/oracle/runtime-reactions.mjs"))
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
    let expected: Vec<_> = output.split_inclusive('\n').collect();
    assert_eq!(expected.len(), 12);
    let sabotage = std::env::var_os("SCANRULER_REACTIONS_SABOTAGE").is_some();
    for (line, expected) in INPUT.lines().zip(expected) {
        let id = line.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        let mut actual = observe(line);
        if sabotage {
            actual = actual.replacen("|42|", "|43|", 1);
        }
        for (name, data) in [
            ("input.bin", line.as_bytes()),
            ("original.bin", expected.as_bytes()),
            ("rust.bin", actual.as_bytes()),
            ("original.stderr.txt", original.stderr.as_slice()),
        ] {
            std::fs::write(case.join(name), data).unwrap();
        }
        assert_eq!(
            actual, expected,
            "{id}: full values/state/reason identity/effects and queued answers"
        );
        let mut wrong = actual.clone().into_bytes();
        *wrong.last_mut().unwrap() ^= 1;
        assert_ne!(wrong, expected.as_bytes(), "wrong value must fail");
        assert_ne!(
            &actual.as_bytes()[..actual.len() - 1],
            expected.as_bytes(),
            "missing output must fail"
        );
    }
    println!(
        "12 complete retained queue/deadline scenarios; raw {}",
        dir.display()
    );
}
