// SPDX-License-Identifier: AGPL-3.0-only
//! Complete Unicode/code-point/context and provider ordering comparisons.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::js_string::{self, StringError};
use port_engine::command_label::{self, LabelError, Limits};
use std::{
    cell::RefCell,
    io::{BufRead, BufReader},
    path::Path,
    process::Command,
    rc::Rc,
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/command-label/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn pattern(i: usize) -> Vec<u16> {
    match i {
        0 => text("PLANE FIT"),
        1 => text("ΟΣ"),
        2 => vec![0x391, 0x345, 0x3a3],
        3 => vec![65, 0x3a3, 0x301],
        4 => text("İKẞ"),
        5 => vec![65, 0xd800, 0x3a3],
        6 => vec![65, 0x3a3, 0xdfff, 65],
        7 => vec![0xd800, 0xdc00, 0xd800, 120, 0xdfff],
        8 => vec![0, 10],
        9 => vec![],
        10 => text("ΜΆΙΟΣ"),
        _ => text("𝔄𐐀💩"),
    }
}
fn codepoint(out: &mut Vec<u16>, c: u32) {
    if c <= 0xffff {
        out.push(c as u16)
    } else {
        let n = c - 0x10000;
        out.push(0xd800 + (n >> 10) as u16);
        out.push(0xdc00 + (n & 1023) as u16)
    }
}
fn model(line: &str) -> (String, Vec<u16>, usize) {
    let p: Vec<_> = line.split('\t').collect();
    let kind: usize = p[1].parse().unwrap();
    let a: u32 = p[2].parse().unwrap();
    let b: u32 = p[3].parse().unwrap();
    if kind == 2 {
        return (p[0].into(), pattern(a as usize), b as usize);
    }
    let mut out = Vec::new();
    for c in a..b {
        if kind == 0 {
            codepoint(&mut out, c);
            out.push(0)
        } else {
            out.extend([65, 0x3a3]);
            codepoint(&mut out, c);
            out.push(0);
            codepoint(&mut out, c);
            out.extend([0x3a3, 0, 65, 0x3a3]);
            codepoint(&mut out, c);
            out.extend([0x3a3, 0]);
            codepoint(&mut out, c);
            out.extend([0x3a3, 65, 0]);
        }
    }
    (p[0].into(), out, 0)
}
fn bytes(units: &[u16]) -> Vec<u8> {
    units.iter().flat_map(|u| u.to_le_bytes()).collect()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
fn limits() -> Limits {
    Limits {
        title_units: 262144,
        output_units: 524288,
    }
}
fn observe(title: &[u16], mode: usize) -> (Vec<u8>, String, String) {
    struct Owner {
        title: Vec<u16>,
    }
    struct Input {
        _identity: bool,
    }
    let command = Owner {
        title: title.to_vec(),
    };
    let input = Input { _identity: true };
    let log = RefCell::new(Vec::new());
    #[derive(Debug)]
    struct Fault {
        _opaque: bool,
    }
    let fault = Rc::new(Fault { _opaque: true });
    let getter = |owner: &Owner| {
        assert!(std::ptr::eq(owner, &command));
        log.borrow_mut().push("title".into());
        if mode == 5 || mode == 6 || mode == 7 {
            Err(fault.clone())
        } else {
            Ok(owner.title.clone())
        }
    };
    let mut custom = |owner: &Owner, arg: &Input| {
        log.borrow_mut().push(format!(
            "label:{}:{}",
            usize::from(std::ptr::eq(owner, &command)),
            usize::from(std::ptr::eq(arg, &input))
        ));
        if mode == 4 {
            return Err(fault.clone());
        }
        Ok(match mode {
            1 | 2 | 6 => None,
            3 => Some(vec![]),
            7 => Some([text("CuStOm"), vec![0xd800]].concat()),
            _ => Some([text("custom"), vec![0xd800]].concat()),
        })
    };
    let label = if mode == 0 {
        None
    } else {
        Some(&mut custom as &mut command_label::Label<'_, Owner, Input, Rc<Fault>>)
    };
    let result = command_label::history_label(&command, &input, getter, label, limits());
    let (output, status) = match result {
        Ok(value) => (bytes(&value), "ok".into()),
        Err(LabelError::Callback(e)) => {
            assert!(Rc::ptr_eq(&e, &fault));
            (vec![], "error:same".into())
        }
        other => panic!("host failure: {other:?}"),
    };
    let effects = log.borrow().join(",");
    (output, status, effects)
}
#[test]
fn lossless_context_and_provider_boundaries() {
    assert_eq!(js_string::UNICODE_VERSION, (17, 0, 0));
    assert_eq!(
        js_string::lowercase(&[65, 0x3a3, 0xd800, 65], 100, 100).unwrap(),
        vec![97, 0x3c2, 0xd800, 97]
    );
    assert_eq!(
        js_string::lowercase(&[0x130], 1, 1),
        Err(StringError::OutputLimit)
    );
    assert_eq!(
        js_string::lowercase(&[65], 0, 10),
        Err(StringError::InputLimit)
    );
    for line in INPUT.lines().filter(|s| s.split('\t').nth(1) == Some("2")) {
        let (_, title, mode) = model(line);
        let (_, status, effects) = observe(&title, mode);
        if mode == 3 || mode == 5 || mode == 7 {
            assert_eq!(effects, "label:1:1");
            assert_eq!(status, "ok")
        }
        if mode == 4 {
            assert_eq!(effects, "label:1:1");
            assert_eq!(status, "error:same")
        }
        if mode == 6 {
            assert_eq!(effects, "label:1:1,title");
            assert_eq!(status, "error:same")
        }
    }
    let c = ();
    let i = ();
    let title = |_: &()| -> Result<Vec<u16>, ()> { panic!("title was eagerly read") };
    let mut callback = |_: &(), _: &()| Ok(Some(vec![]));
    let result = command_label::history_label(
        &c,
        &i,
        title,
        Some(&mut callback),
        Limits {
            title_units: 0,
            output_units: 7,
        },
    )
    .unwrap();
    assert_eq!(result, text("Agent: "));
}
#[test]
#[ignore = "original command-label Unicode/effect profile requires Node22 and pinned graph"]
fn pinned_command_label_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    assert!(
        Command::new("node")
            .arg(root.join("docs/port-plan/oracle/build-command-label.mjs"))
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/command-label-diff/run-{run}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("candidate.txt"), &sha.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let transcript = std::fs::File::create(dir.join("source.tsv")).unwrap();
    let error_file = std::fs::File::create(dir.join("source.stderr.txt")).unwrap();
    let stdin =
        std::fs::File::open(root.join("docs/port-plan/fixtures/command-label/cases.tsv")).unwrap();
    assert!(
        Command::new("node")
            .arg(root.join("docs/port-plan/oracle/command-label.mjs"))
            .current_dir(&root)
            .stdin(stdin)
            .stdout(transcript)
            .stderr(error_file)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        std::fs::metadata(dir.join("source.stderr.txt"))
            .unwrap()
            .len(),
        0
    );
    let reader = BufReader::new(std::fs::File::open(dir.join("source.tsv")).unwrap());
    let mut lines = reader.lines();
    let mut count = 0;
    for descriptor in INPUT.lines() {
        let line = lines.next().unwrap().unwrap();
        let p: Vec<_> = line.split('\t').collect();
        let (id, title, mode) = model(descriptor);
        assert_eq!(p[0], id);
        let input = bytes(&title);
        assert_eq!(
            input,
            unhex(p[1]),
            "independent input constructors differ: {id}"
        );
        let (original, status) = if let Some(hex) = p[2].strip_prefix("ok:") {
            (unhex(hex), "ok")
        } else {
            assert_eq!(p[2], "error:same");
            (vec![], "error:same")
        };
        let (actual, actual_status, effects) = observe(&title, mode);
        let case = dir.join(&id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, data) in [
            ("input.bin", input.as_slice()),
            ("descriptor.tsv", descriptor.as_bytes()),
            ("original.bin", original.as_slice()),
            ("rust.bin", actual.as_slice()),
            ("original.status.txt", status.as_bytes()),
            ("rust.status.txt", actual_status.as_bytes()),
            ("original.effects.txt", p[3].as_bytes()),
            ("rust.effects.txt", effects.as_bytes()),
        ] {
            std::fs::write(case.join(name), data).unwrap()
        }
        assert!(original == actual, "full UTF16 value differs: {id}");
        assert_eq!(status, actual_status, "{id}");
        assert_eq!(p[3], effects, "{id}");
        count += 1;
    }
    assert_eq!(count, 640);
    std::fs::remove_file(dir.join("source.tsv")).unwrap();
    assert!(lines.next().is_none());
    println!(
        "{count} complete command-label input/value/effect comparisons; {}",
        dir.display()
    );
}
