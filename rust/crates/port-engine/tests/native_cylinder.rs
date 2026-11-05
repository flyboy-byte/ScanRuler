// SPDX-License-Identifier: AGPL-3.0-only
//! Native element.fit_marked { kind: "cylinder" } against the pinned original,
//! step by step over the full store state, on fixed generated tube meshes.
#![allow(clippy::unwrap_used, clippy::panic)]
#[path = "support/native_observe.rs"]
#[allow(dead_code, unused_imports)]
mod observation;
use observation::*;
use port_engine::{native_engine::Engine, schema_value::Value as V};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/native-cylinder/cases.tsv");
fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn mesh(name: &str) -> Vec<u8> {
    std::fs::read(root().join(format!(
        "docs/port-plan/fixtures/native-cylinder/{name}.stl"
    )))
    .unwrap()
}
fn xyz(v: [f64; 3]) -> V {
    V::Array(v.into_iter().map(V::Number).collect())
}
fn observe_cylinder(row: &str) -> V {
    let c = row.split('\t').collect::<Vec<_>>();
    let engine = Engine::new().unwrap();
    let (result, error) = outcome(engine.execute(
        "scan.open",
        object(vec![
            ("bytes", V::Bytes(mesh(c[3]))),
            ("name", string(&format!("{}.stl", c[0]))),
            ("units", string(c[2])),
        ]),
    ));
    let k = c[1].strip_prefix("cyl-").unwrap();
    let n = c[4].parse::<usize>().unwrap();
    let mut args = vec![
        ("kind", string("cylinder")),
        ("vertices", V::Array((0..n).map(number).collect())),
        ("sigma", V::Number(c[5].parse().unwrap())),
    ];
    if k == "assumed" {
        args.push(("assumed", V::Number(20.)))
    }
    if let Some(o) = k.strip_prefix("orient-") {
        let mut orient = vec![("to", string(if o.ends_with("xy") { "XY" } else { "XZ" }))];
        if o.contains("inplane") {
            orient.push(("relation", string("inPlane")))
        }
        args.push(("orient", object(orient)))
    }
    if k.starts_with("extend-") {
        let n = V::Number;
        let fields = match k {
            "extend-grow" => vec![("start", n(1.)), ("end", n(2.))],
            "extend-trim" => vec![("start", n(-1000.)), ("end", n(-1000.))],
            "extend-zero" => vec![("start", n(0.)), ("end", n(0.))],
            "extend-empty" => vec![],
            "extend-bad-side" => vec![("uMin", n(1.))],
            _ => vec![("start", n(3.)), ("vMax", n(2.))],
        };
        args.push(("extend", object(fields)))
    }
    if k == "name" {
        args.push(("name", string("  Bore A  ")))
    }
    if k == "known" {
        args.push(("knownDiameter", V::Number(20.)))
    }
    if k.starts_with("box") {
        args.retain(|(key, _)| *key != "vertices");
        args.push((
            "box",
            object(vec![
                ("min", xyz([-100000.; 3])),
                ("max", xyz([100000.; 3])),
            ]),
        ));
    }
    if k == "box-facing" {
        args.push((
            "normal",
            object(vec![
                ("dir", xyz([1., 0., 0.])),
                ("withinDeg", V::Number(60.)),
            ]),
        ))
    }
    let input = object(args);
    let mut actions = vec![("element.fit_marked", input.clone())];
    if k == "two" {
        actions.push(("element.fit_marked", input))
    }
    if ["undo", "redo", "undo-redo-twice", "fail-undo"].contains(&k) {
        actions.push(("history.undo", object(vec![])))
    }
    if ["redo", "undo-redo-twice"].contains(&k) {
        actions.push(("history.redo", object(vec![])))
    }
    if k == "undo-redo-twice" {
        actions.push(("history.undo", object(vec![])));
        actions.push(("history.redo", object(vec![])))
    }
    let mut steps = vec![];
    for (name, input) in actions {
        let (result, error) = outcome(engine.execute(name, input));
        let V::Object(fields) = frame(&engine, result, error) else {
            panic!("frame")
        };
        let get = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| *key == text(k))
                .unwrap()
                .1
                .clone()
        };
        steps.push(object(vec![
            ("name", string(name)),
            ("result", get("result")),
            ("error", get("error")),
            ("versions", get("versions")),
            ("sources", get("sources")),
            ("stores", get("stores")),
        ]));
    }
    if std::env::var_os("SCANRULER_NATIVE_CYLINDER_CANARY").is_some() {
        engine
            .document()
            .store(port_engine::scan_session::StoreId::App)
            .patch(vec![(
                text("nextId"),
                port_engine::history_record::Cell::Number(-1.),
            )])
            .unwrap();
    }
    let V::Object(mut f) = frame(&engine, result, error) else {
        panic!("frame")
    };
    f.push((text("steps"), V::Array(steps)));
    V::Object(f)
}

#[test]
fn every_case_fits_or_refuses_natively() {
    for row in INPUT.lines() {
        let mut out = vec![];
        wire(&observe_cylinder(row), &mut out);
        assert!(!out.is_empty(), "{row}");
    }
}

fn field(v: &V, k: &str) -> V {
    let V::Object(f) = v else { panic!("object") };
    f.iter().find(|(key, _)| *key == text(k)).unwrap().1.clone()
}

/// A saved cylinder, finished every way, reopens as the same element.
#[test]
fn saved_cylinder_reopens_unchanged() {
    let engine = Engine::new().unwrap();
    engine
        .execute(
            "scan.open",
            object(vec![
                ("bytes", V::Bytes(mesh("noisy"))),
                ("name", string("noisy.stl")),
            ]),
        )
        .unwrap();
    let n = V::Number;
    engine
        .execute(
            "element.fit_marked",
            object(vec![
                ("kind", string("cylinder")),
                ("vertices", V::Array((0..144).map(number).collect())),
                ("assumed", n(20.)),
                ("orient", object(vec![("to", string("XY"))])),
                ("extend", object(vec![("start", n(1.)), ("end", n(2.))])),
            ]),
        )
        .unwrap();
    let elements = |e: &Engine| {
        pack(
            &e.document()
                .store(port_engine::scan_session::StoreId::App)
                .get("elements"),
        )
    };
    let before = elements(&engine);
    let saved = engine.execute("project.save", object(vec![])).unwrap();
    let bytes = field(&field(&saved, "file"), "bytes");
    let reopened = Engine::new().unwrap();
    reopened
        .execute(
            "project.load",
            object(vec![("bytes", bytes), ("name", string("bore.scanruler"))]),
        )
        .unwrap();
    assert_eq!(elements(&reopened), before);
    let V::Array(list) = before else {
        panic!("elements")
    };
    assert_eq!(field(&list[0], "kind"), string("cylinder"));
    assert_eq!(field(&list[0], "name"), string("Cylinder 1"));
}

/// Every case's command results and complete store state, after each step,
/// must be byte-identical to the pinned original's.
#[test]
#[ignore = "complete pinned registry/session/worker requires Node22"]
fn pinned_native_cylinder_comparison() {
    let root = root();
    let build = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-scan-session-owned.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/native-import-reference.mjs"))
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
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let source = String::from_utf8(out.stdout).unwrap();
    let rows = INPUT.lines().count();
    assert_eq!(source.lines().count(), rows);
    let run = root.join("rust/target/native-cylinder-diff");
    std::fs::create_dir_all(&run).unwrap();
    let mut failures = vec![];
    for (row, line) in INPUT.lines().zip(source.lines()) {
        let (id, h) = line.split_once('\t').unwrap();
        assert_eq!(row.split('\t').next().unwrap(), id);
        let expected = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut actual = vec![];
        wire(&observe_cylinder(row), &mut actual);
        if expected != actual {
            let (a, b) = (
                run.join(format!("{id}.original")),
                run.join(format!("{id}.rust")),
            );
            std::fs::write(&a, &expected).unwrap();
            std::fs::write(&b, &actual).unwrap();
            let shown = Command::new("python3")
                .arg(root.join("docs/port-plan/oracle/compare-native-plane.py"))
                .arg(&a)
                .arg(&b)
                .output()
                .unwrap();
            eprintln!("mismatch {id}: {}", String::from_utf8_lossy(&shown.stdout));
            failures.push(id.to_owned());
        }
    }
    eprintln!("{rows} native cylinder cases; mismatches: {failures:?}");
    assert!(failures.is_empty());
}
