// SPDX-License-Identifier: AGPL-3.0-only
//! Full independent source/Rust registration/lookup/state traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::{
    plugin_busy::{Check, Limits, Plugins, RegisterError, Release, Text},
    watch_stores::StoreList,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/plugin-busy/cases.tsv");
fn text(s: &str) -> Text {
    Rc::from(s.encode_utf16().collect::<Vec<_>>())
}
fn hex(s: &[u16]) -> String {
    s.iter().map(|v| format!("{v:04x}")).collect()
}
type Registry = Plugins<'static, usize, usize, ()>;
struct World {
    registry: Registry,
    key: Text,
    other: Text,
    third: Text,
    arrays: Vec<StoreList<usize>>,
    stops: Vec<Rc<usize>>,
    checks: RefCell<Vec<Check<'static, ()>>>,
    releases: RefCell<Vec<Option<Release<'static, usize, usize, ()>>>>,
    calls: Cell<[usize; 3]>,
    fired: Cell<bool>,
    mode: usize,
}
impl World {
    fn install(
        &self,
        key: Text,
        c: usize,
        s: usize,
        t: Option<usize>,
    ) -> Result<String, RegisterError> {
        let check = self.checks.borrow()[c].clone();
        let release = self.registry.register(
            key,
            check,
            self.arrays[s].clone(),
            t.map(|n| self.stops[n].clone()),
        )?;
        self.releases.borrow_mut().push(Some(release));
        Ok("-".into())
    }
    fn release(&self, n: usize) {
        if let Some(r) = &self.releases.borrow()[n] {
            r.release()
        }
    }
    fn op(&self, op: &str) -> String {
        let result = match op {
            "i0" => self.install(
                self.key.clone(),
                if (2..=4).contains(&self.mode) { 2 } else { 0 },
                if self.mode == 5 { 0 } else { 1 },
                if self.mode == 5 {
                    None
                } else {
                    Some(if self.mode >= 2 { 1 } else { 0 })
                },
            ),
            "dup" | "i1" => self.install(self.key.clone(), 1, 2, Some(1)),
            "same" => self.install(self.key.clone(), 0, 2, None),
            "other" => self.install(
                self.other.clone(),
                1,
                if self.mode == 5 { 1 } else { 2 },
                if self.mode == 5 { Some(2) } else { None },
            ),
            "r0" => {
                self.release(0);
                Ok("-".into())
            }
            "r1" => {
                self.release(1);
                Ok("-".into())
            }
            "drop0" => {
                self.releases.borrow_mut()[0] = None;
                Ok("-".into())
            }
            "busy" => Ok(self.registry.busy().unwrap().map_or_else(
                || hex(&"<null>".encode_utf16().collect::<Vec<_>>()),
                |v| hex(&v),
            )),
            _ => panic!("unknown op"),
        };
        result.unwrap_or_else(|e| format!("!{}", hex(&e.message().unwrap().unwrap())))
    }
    fn frame(&self, result: &str) -> String {
        let entries = self.registry.snapshot().unwrap();
        let checks = self.checks.borrow();
        let check = entries
            .iter()
            .map(|e| {
                format!(
                    "{}:{}",
                    hex(&e.id),
                    checks.iter().position(|c| Rc::ptr_eq(c, &e.check)).unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let store = entries
            .iter()
            .map(|e| {
                format!(
                    "{}:{}",
                    hex(&e.id),
                    self.arrays
                        .iter()
                        .position(|s| Rc::ptr_eq(s, &e.stores))
                        .unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let stop = entries
            .iter()
            .filter_map(|e| {
                e.stop.as_ref().map(|stop| {
                    format!(
                        "{}:{}",
                        hex(&e.id),
                        self.stops.iter().position(|s| Rc::ptr_eq(s, stop)).unwrap()
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(",");
        let calls = self
            .calls
            .get()
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!("{result}|{check}|{store}|{stop}|{calls}")
    }
}
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let a: usize = parts[1].parse().unwrap();
    let b: usize = parts[2].parse().unwrap();
    let mode: usize = parts[3].parse().unwrap();
    let key: Text = match mode {
        0 => text("x"),
        1 => text(""),
        2 => text("__proto__"),
        3 => text("q\"u"),
        4 => text("💩"),
        _ => Rc::from(
            "unpaired:"
                .encode_utf16()
                .chain([0xd800])
                .collect::<Vec<_>>(),
        ),
    };
    let mut other = key.to_vec();
    other.push(66);
    let mut third = key.to_vec();
    third.push(67);
    let stores: Vec<_> = (0..3).map(Rc::new).collect();
    let world = Rc::new(World {
        registry: Plugins::new(Limits {
            slots: 100,
            visits: 100,
            id_units: 100,
            depth: 20,
        }),
        key,
        other: Rc::from(other),
        third: Rc::from(third),
        arrays: vec![
            Rc::from(vec![]),
            Rc::from(vec![stores[0].clone(), stores[1].clone()]),
            Rc::from(vec![
                stores[1].clone(),
                stores[0].clone(),
                stores[1].clone(),
            ]),
        ],
        stops: (0..3).map(Rc::new).collect(),
        checks: RefCell::new(Vec::new()),
        releases: RefCell::new(Vec::new()),
        calls: Cell::new([0; 3]),
        fired: Cell::new(false),
        mode,
    });
    for index in 0..3 {
        let weak = Rc::downgrade(&world);
        world.checks.borrow_mut().push(Rc::new(move || {
            let w = weak.upgrade().unwrap();
            let mut calls = w.calls.get();
            calls[index] += 1;
            w.calls.set(calls);
            if index == 2 {
                if !w.fired.replace(true) {
                    match mode {
                        2 => {
                            w.release(1);
                            w.install(w.other.clone(), 0, 2, Some(2)).unwrap();
                        }
                        3 => {
                            w.install(w.third.clone(), 1, 1, Some(0)).unwrap();
                        }
                        4 => {
                            w.release(0);
                            w.install(w.key.clone(), 0, 2, Some(2)).unwrap();
                        }
                        _ => {}
                    }
                }
                return Ok(None);
            }
            match if index == 0 { a } else { b } {
                0 => Ok(None),
                1 => Ok(Some(text(""))),
                2 => Ok(Some(Rc::from(
                    "Busy "
                        .encode_utf16()
                        .chain([0xd800, 0xd83d, 0xdca9])
                        .collect::<Vec<_>>(),
                ))),
                _ => Err(()),
            }
        }))
    }
    let programs = [
        vec!["i0", "dup", "busy", "r0", "i1", "r0", "busy"],
        vec!["i0", "r0", "same", "r0", "r0", "busy"],
        vec!["i0", "other", "busy", "busy", "r0", "busy"],
        vec!["i0", "other", "busy", "busy", "r0", "busy"],
        vec!["i0", "other", "busy", "busy", "r0", "busy"],
        vec!["i0", "drop0", "other", "busy", "dup", "r1", "busy"],
    ];
    let mut frames = vec![world.frame("-")];
    for op in &programs[mode] {
        let result = world.op(op);
        frames.push(world.frame(&result))
    }
    format!("{}\t{}\n", parts[0], frames.join(";"))
}
#[test]
fn bounds_and_source_duplicate_precedence() {
    let r: Registry = Plugins::new(Limits {
        slots: 1,
        visits: 0,
        id_units: 2,
        depth: 1,
    });
    let check: Check<'_, ()> = Rc::new(|| Ok(None));
    let array: StoreList<usize> = Rc::from(vec![]);
    let release = r
        .register(text("id"), check.clone(), array.clone(), None)
        .unwrap();
    assert!(matches!(
        r.register(text("id"), check.clone(), array.clone(), None),
        Err(RegisterError::Duplicate(_))
    ));
    assert!(r.busy().is_err());
    release.release();
    assert!(r.busy().unwrap().is_none());
    assert!(matches!(
        r.register(text("x"), check, array, None),
        Err(RegisterError::Limit)
    ));
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()))
    }
}
#[test]
fn recursive_lookup_is_bounded_and_depth_is_released() {
    let r: Registry = Plugins::new(Limits {
        slots: 10,
        visits: 10,
        id_units: 10,
        depth: 1,
    });
    let nested = r.clone();
    let seen = Rc::new(Cell::new(0));
    let calls = seen.clone();
    let check: Check<'_, ()> = Rc::new(move || {
        assert!(nested.busy().is_err());
        calls.set(calls.get() + 1);
        Ok(None)
    });
    let release = r
        .register(text("id"), check, Rc::from(vec![]), None)
        .unwrap();
    assert!(r.busy().unwrap().is_none());
    assert!(r.busy().unwrap().is_none());
    assert_eq!(seen.get(), 2);
    release.release();
}
#[test]
#[ignore = "complete original plugin busy registry requires Node22 and original graph"]
fn pinned_plugin_busy_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-plugin-busy.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/plugin-busy.mjs"))
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
    assert_eq!(originals.len(), 96);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/plugin-busy-diff/run-{run}"));
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
        "96 complete plugin identity/live lookup observations; {}",
        dir.display()
    );
}
