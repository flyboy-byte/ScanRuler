// SPDX-License-Identifier: AGPL-3.0-only
//! Full original guard and editor-work observations.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::history_guards::{self, DraftWork, GuardError, Guards};
use std::{
    cell::RefCell,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/history-guards/cases.tsv");
fn draft(code: usize) -> Option<DraftWork<'static>> {
    (code != 0).then_some(DraftWork {
        edit_present: code == 2 || code == 8,
        picks: usize::from(code == 3),
        selection: usize::from(code == 6),
        references: if code == 4 {
            &[true]
        } else if code == 5 {
            &[false]
        } else {
            &[]
        },
    })
}
fn observe(line: &str) -> String {
    let p: Vec<_> = line.split('\t').collect();
    let mask: usize = p[1].parse().unwrap();
    let bit = |i| mask & (1usize << i) != 0usize;
    let d = draft(p[2].parse().unwrap());
    let f = draft(p[3].parse().unwrap());
    let sd = draft(p[4].parse().unwrap());
    let fd = draft(p[5].parse().unwrap());
    let part_mask: usize = p[6].parse().unwrap();
    let fault: i32 = p[7].parse().unwrap();
    let hook_fault = p[8] == "1";
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut callbacks: Vec<Box<dyn FnMut() -> Result<bool, String>>> = (0..3)
        .map(|i| {
            let log = log.clone();
            Box::new(move || {
                log.borrow_mut().push(format!("participant:{i}"));
                if fault == i {
                    Err(format!("participant-{i}"))
                } else {
                    Ok(part_mask & (1usize << i) != 0usize)
                }
            }) as Box<dyn FnMut() -> Result<bool, String>>
        })
        .collect();
    let mut refs: Vec<&mut dyn FnMut() -> Result<bool, String>> = callbacks
        .iter_mut()
        .map(|f| f as &mut dyn FnMut() -> Result<bool, String>)
        .collect();
    let g = Guards {
        restoring: bit(0),
        paused: usize::from(bit(1)),
        busy: bit(2),
        image_busy: bit(3),
        nominal_busy: bit(4),
        align_running: bit(5),
        map_running: bit(6),
        thickness_running: bit(7),
        align_draft: bit(8),
        section_draft: bit(9),
        flat_tool: bit(10),
        draft: d,
        flat_draft: f,
        dimension: sd,
        flat_dimension: fd,
    };
    let result = history_guards::blocked(
        &g,
        || {
            log.borrow_mut().push("hook".to_string());
            if hook_fault {
                Err("hook".to_string())
            } else {
                Ok(bit(11))
            }
        },
        &mut refs,
    );
    let (blocked, error) = match result {
        Ok(b) => (if b { "1" } else { "0" }.to_string(), "-".to_string()),
        Err(GuardError::Callback(e)) => ("-".to_string(), e),
        Err(e) => panic!("{e:?}"),
    };
    format!(
        "{}\t{blocked}\t{error}\t{}\t{}\t{}\t{}\t{}\n",
        p[0],
        usize::from(history_guards::draft_holds(d)),
        usize::from(history_guards::flat_draft_holds(f)),
        usize::from(history_guards::dim_holds(sd)),
        usize::from(history_guards::dim_holds(fd)),
        log.borrow().join(",")
    )
}
#[test]
fn opaque_faults_and_short_circuit() {
    let mut none: Vec<&mut dyn FnMut() -> Result<bool, Vec<u16>>> = Vec::new();
    assert_eq!(
        history_guards::blocked(&Guards::default(), || Err(vec![0xd800, 65]), &mut none),
        Err(GuardError::Callback(vec![0xd800, 65]))
    );
    assert_eq!(
        history_guards::blocked(
            &Guards {
                restoring: true,
                ..Guards::default()
            },
            || panic!("restoring"),
            &mut none
        ),
        Ok(true)
    );
    for line in INPUT.lines() {
        assert!(observe(line).starts_with(line.split('\t').next().unwrap()));
    }
    let mut callbacks: Vec<_> = (0..4097).map(|_| || Ok::<bool, Vec<u16>>(false)).collect();
    let mut refs: Vec<&mut dyn FnMut() -> Result<bool, Vec<u16>>> = callbacks
        .iter_mut()
        .map(|f| f as &mut dyn FnMut() -> Result<bool, Vec<u16>>)
        .collect();
    assert_eq!(
        history_guards::blocked(&Guards::default(), || Ok(false), &mut refs),
        Err(GuardError::Limit)
    );
    assert_eq!(
        history_guards::blocked(
            &Guards {
                paused: 1,
                ..Guards::default()
            },
            || panic!("paused"),
            &mut refs
        ),
        Ok(true)
    );
}
#[test]
#[ignore = "full original history guards comparison requires Node22 and pinned git object"]
fn pinned_history_guards_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let status = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-history-guards.mjs"))
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/history-guards.mjs"))
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
    assert_eq!(originals.len(), 212);
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/history-guards-diff/run-{run}"));
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
    println!("212 complete guard/work observations; {}", dir.display());
}
