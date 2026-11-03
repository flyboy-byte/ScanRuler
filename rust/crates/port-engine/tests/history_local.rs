// SPDX-License-Identifier: AGPL-3.0-only
//! Real original local history selection and editor dispatch traces.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::history_local::{self, Direction, Dispatch, LocalError, LocalHistory};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/history-local/cases.tsv");
struct Editor {
    index: usize,
    mask: usize,
    active_fault: i32,
    body_fault: i32,
    workspace: Vec<u16>,
    current: Rc<RefCell<String>>,
    log: Rc<RefCell<Vec<String>>>,
}
impl LocalHistory<String> for Editor {
    fn active(&mut self) -> Result<bool, String> {
        self.log
            .borrow_mut()
            .push(format!("active:{}:{}", self.index, self.current.borrow()));
        if self.active_fault == self.index as i32 {
            return Err(format!("active-{}", self.index));
        }
        Ok(self.mask & (1 << self.index) != 0)
    }
    fn workspace(&self) -> &[u16] {
        &self.workspace
    }
    fn step(&mut self, d: Direction) -> Result<(), String> {
        let verb = match d {
            Direction::Undo => "undo",
            Direction::Redo => "redo",
        };
        self.log
            .borrow_mut()
            .push(format!("{verb}:{}:{}", self.index, self.current.borrow()));
        if self.body_fault == self.index as i32 {
            Err(format!("body-{}", self.index))
        } else {
            Ok(())
        }
    }
}
fn observe(line: &str) -> String {
    let parts: Vec<_> = line.split('\t').collect();
    let mask = parts[2].parse().unwrap();
    let guard: usize = parts[5].parse().unwrap();
    let current = Rc::new(RefCell::new("measure".to_string()));
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut editors: Vec<_> = ["flat", "reverse", "measure"]
        .iter()
        .enumerate()
        .map(|(i, w)| Editor {
            index: i,
            mask,
            active_fault: parts[3].parse().unwrap(),
            body_fault: parts[4].parse().unwrap(),
            workspace: w.encode_utf16().collect(),
            current: current.clone(),
            log: log.clone(),
        })
        .collect();
    let mut locals: Vec<&mut dyn LocalHistory<String>> = editors
        .iter_mut()
        .map(|e| e as &mut dyn LocalHistory<String>)
        .collect();
    let mut selected = "-".to_string();
    let mut error = "-".to_string();
    if parts[1] == "select" {
        match history_local::active(&mut locals) {
            Ok(Some(i)) => selected = i.to_string(),
            Ok(None) => selected = "none".into(),
            Err(LocalError::Callback(e)) => error = e,
            Err(e) => panic!("{e:?}"),
        }
    } else {
        let result = history_local::dispatch(
            guard != 0,
            &mut locals,
            if parts[1] == "undo" {
                Direction::Undo
            } else {
                Direction::Redo
            },
            |w| {
                *current.borrow_mut() = String::from_utf16(w).unwrap();
                Ok(())
            },
        );
        match result {
            Err(LocalError::Callback(e)) => error = e,
            Err(e) => panic!("{e:?}"),
            Ok(Dispatch::Global) => assert_eq!(mask, 0),
            _ => {}
        }
    }
    format!(
        "{}\t{selected}\t{error}\t{}\t0\t0\t0\t{}\t{}\n",
        parts[0],
        current.borrow(),
        usize::from(guard == 2),
        log.borrow().join(",")
    )
}
#[test]
fn typed_errors_guards_and_global_continuation() {
    assert_eq!(INPUT.lines().count(), 192);
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
    let mut none: Vec<&mut dyn LocalHistory<String>> = Vec::new();
    assert_eq!(
        history_local::dispatch(false, &mut none, Direction::Undo, |_| panic!("workspace")),
        Ok(Dispatch::Global)
    );
    assert_eq!(
        history_local::dispatch(true, &mut none, Direction::Undo, |_| panic!("workspace")),
        Ok(Dispatch::Blocked)
    );
    let a = observe(INPUT.lines().next().unwrap());
    let mut changed = a.clone().into_bytes();
    changed[0] ^= 1;
    assert_ne!(a.as_bytes(), changed);
    assert_ne!(a.as_bytes(), &a.as_bytes()[..a.len() - 1]);
}
#[test]
#[ignore = "full original local history comparison requires Node22 and pinned git object"]
fn pinned_history_local_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-history-local.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/history-local.mjs"))
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
    let dir = root.join(format!("rust/target/history-local-diff/run-{run}"));
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
        "192 complete local selection/dispatch observations; {}",
        dir.display()
    );
}

#[test]
fn opaque_faults_and_host_limits() {
    struct Fault;
    impl LocalHistory<Vec<u16>> for Fault {
        fn active(&mut self) -> Result<bool, Vec<u16>> {
            Err(vec![0xd800, 65])
        }
        fn workspace(&self) -> &[u16] {
            panic!("workspace after fault")
        }
        fn step(&mut self, _: Direction) -> Result<(), Vec<u16>> {
            panic!("action after fault")
        }
    }
    let mut fault = Fault;
    assert_eq!(
        history_local::active(&mut [&mut fault]),
        Err(LocalError::Callback(vec![0xd800, 65]))
    );
    let mut faults: Vec<_> = (0..=history_local::MAX_LOCALS).map(|_| Fault).collect();
    let mut refs: Vec<&mut dyn LocalHistory<Vec<u16>>> = faults
        .iter_mut()
        .map(|f| f as &mut dyn LocalHistory<Vec<u16>>)
        .collect();
    assert_eq!(history_local::active(&mut refs), Err(LocalError::Limit));
    assert_eq!(
        history_local::dispatch(true, &mut refs, Direction::Redo, |_| panic!("blocked")),
        Ok(Dispatch::Blocked)
    );
}
