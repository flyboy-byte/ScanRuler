// SPDX-License-Identifier: AGPL-3.0-only
//! Independent native registered-plane traces, including complete history patches.
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
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/native-plane/cases-m3b.tsv");
fn scan_bytes(row: &str) -> Vec<u8> {
    let c = row.split('\t').collect::<Vec<_>>();
    let n = c[4].parse::<usize>().unwrap();
    let mut seed = if c[3] == "flat" {
        None
    } else {
        Some(c[3].parse::<u32>().unwrap())
    };
    let mut points = vec![];
    for y in 0..n {
        for x in 0..n {
            let z = if let Some(s) = &mut seed {
                *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                ((f64::from(*s) / 4294967296. - 0.5) * 0.1) as f32
            } else {
                0.
            };
            points.push(format!(
                "{x} {y} {}",
                port_core::js_number::number_string(f64::from(z))
            ))
        }
    }
    let mut body = format!("solid {}\n", c[0]);
    for y in 0..n - 1 {
        for x in 0..n - 1 {
            let a = y * n + x;
            for [a, b, c] in [[a, a + 1, a + n], [a + 1, a + n + 1, a + n]] {
                body += &format!(
                    "facet normal 0 0 1\nouter loop\nvertex {}\nvertex {}\nvertex {}\nendloop\nendfacet\n",
                    points[a], points[b], points[c]
                );
            }
        }
    }
    body += "endsolid\n";
    body.into_bytes()
}
fn observe_plane(row: &str) -> V {
    let c = row.split('\t').collect::<Vec<_>>();
    let engine = Engine::new().unwrap();
    let (result, error) = if c[1].ends_with("no-scan") {
        (undefined(), undefined())
    } else {
        outcome(engine.execute(
            "scan.open",
            object(vec![
                ("bytes", V::Bytes(scan_bytes(row))),
                ("name", string(&format!("{}.stl", c[0]))),
                ("units", string(c[2])),
            ]),
        ))
    };
    let n = c[4].parse::<usize>().unwrap();
    let mut vertices = (0..n * n).collect::<Vec<_>>();
    match c[1] {
        "plane-short" => vertices.truncate(3),
        "plane-reverse" => vertices.reverse(),
        "plane-duplicate" => vertices.extend(vertices.clone()),
        "plane-invalid" => vertices.push(n * n),
        _ => {}
    }
    let mut args = vec![
        ("kind", string("plane")),
        (
            "vertices",
            V::Array(vertices.into_iter().map(number).collect()),
        ),
        ("sigma", V::Number(c[5].parse().unwrap())),
    ];
    match c[1] {
        "plane-known" => args.push(("knownDiameter", V::Number(5.))),
        "plane-assumed" => args.push(("assumed", V::Number(5.))),
        "plane-name" => args.push(("name", string("  Bench plane  "))),
        "plane-name-surrogate" => {
            args.push(("name", V::String([vec![0xd800], text("Bench")].concat())))
        }
        "plane-name-bom" => args.push(("name", string("\u{feff} Bench plane \u{feff}"))),
        "plane-name-blank" => args.push(("name", string(" \t "))),
        "plane-name-blank-bom" => args.push(("name", string("\u{feff}\u{feff}"))),
        "plane-missing" => args.retain(|(k, _)| *k != "vertices"),
        _ => {}
    }
    let shape = c[1];
    if let Some(key) = shape.strip_prefix("plane-undefined-") {
        let key = if key == "vertices-box" {
            "vertices"
        } else {
            key
        };
        if let Some((_, value)) = args.iter_mut().find(|(k, _)| *k == key) {
            *value = V::Undefined;
        } else {
            args.push((key, V::Undefined));
        }
        if shape == "plane-undefined-vertices-box" {
            let xyz = |v: [f64; 3]| V::Array(v.into_iter().map(V::Number).collect());
            args.push((
                "box",
                object(vec![
                    ("min", xyz([-100000.; 3])),
                    ("max", xyz([100000.; 3])),
                ]),
            ));
        }
    }
    if shape.starts_with("plane-extend-") {
        let fields = match shape {
            "plane-extend-grow" => vec![
                ("uMin", V::Number(1.)),
                ("uMax", V::Number(2.)),
                ("vMin", V::Number(3.)),
                ("vMax", V::Number(4.)),
            ],
            "plane-extend-trim" => vec![
                ("uMin", V::Number(-100.)),
                ("uMax", V::Number(-100.)),
                ("vMin", V::Number(-100.)),
                ("vMax", V::Number(-100.)),
            ],
            "plane-extend-reverse" => vec![
                ("uMax", V::Number(-100.)),
                ("uMin", V::Number(-100.)),
                ("vMax", V::Number(-100.)),
                ("vMin", V::Number(-100.)),
            ],
            "plane-extend-zero" => vec![
                ("uMin", V::Number(0.)),
                ("uMax", V::Number(0.)),
                ("vMin", V::Number(0.)),
                ("vMax", V::Number(0.)),
            ],
            "plane-extend-empty" => vec![],
            "plane-extend-bad-side" => vec![("start", V::Number(2.))],
            "plane-extend-partial-bad" => vec![("uMin", V::Number(3.)), ("start", V::Number(2.))],
            "plane-extend-index-key" => vec![("uMin", V::Number(3.)), ("0", V::Number(2.))],
            _ => vec![("uMin", V::Undefined)],
        };
        args.push(("extend", object(fields)));
    }
    if shape.starts_with("plane-orient-section") {
        use port_engine::history_record::Cell as C;
        let xyz =
            |v: [f64; 3]| C::Array(std::sync::Arc::new(v.into_iter().map(C::Number).collect()));
        let obj = |v: Vec<(&str, C)>| {
            C::Object(std::sync::Arc::new(
                v.into_iter().map(|(k, v)| (text(k), v)).collect(),
            ))
        };
        let section = obj(vec![
            ("id", C::Number(1.)),
            ("name", C::String(text("Section ref"))),
            ("color", C::String(text("#1877c0"))),
            ("visible", C::Boolean(true)),
            (
                "frame",
                obj(vec![
                    ("origin", xyz([10., 0., 0.])),
                    ("normal", xyz([1., 0., 0.])),
                    ("basisU", xyz([0., 1., 0.])),
                    ("basisV", xyz([0., 0., 1.])),
                ]),
            ),
            ("message", C::String(text("No scan is loaded."))),
        ]);
        engine
            .document()
            .store(port_engine::scan_session::StoreId::App)
            .patch(vec![
                (
                    text("sections"),
                    C::Array(std::sync::Arc::new(vec![section])),
                ),
                (text("nextId"), C::Number(2.)),
                (text("nextSectionNumber"), C::Number(2.)),
            ])
            .unwrap();
    }
    if shape.starts_with("plane-orient-") {
        let to = match shape {
            "plane-orient-xy" | "plane-orient-inplane-xy" => string("XY"),
            "plane-orient-yz" | "plane-orient-inplane-yz" => string("YZ"),
            "plane-orient-xz" => string("XZ"),
            "plane-orient-alias" => string(" zx_plane "),
            "plane-orient-bad-name" => string("Missing"),
            "plane-orient-bad-id" => V::Number(999.),
            "plane-orient-bad-surrogate" => V::String([vec![0xd800], text("Missing")].concat()),
            "plane-orient-reference-name" => string(" plane 1 "),
            "plane-orient-reference-string-id" => string("1"),
            "plane-orient-reference-unicode" => string("ος"),
            "plane-orient-section-name" => string("section ref"),
            _ => V::Number(1.),
        };
        let mut orient = vec![("to", to)];
        if shape.contains("inplane") {
            orient.push(("relation", string("inPlane")));
        }
        args.push(("orient", object(orient)));
    }
    if c[1].starts_with("plane-box-") {
        let xyz = |v: [f64; 3]| V::Array(v.into_iter().map(V::Number).collect());
        let mut min = [-100000.; 3];
        let mut max = [100000.; 3];
        match c[1] {
            "plane-box-empty" => {
                min = [10000.; 3];
                max = [10001.; 3]
            }
            "plane-box-short" => {
                min = [0., 0., -1.];
                max = [1., 1., 1.]
            }
            "plane-box-inverted" => {
                min = [1., 0., 0.];
                max = [0., 1., 1.]
            }
            _ => {}
        }
        if c[1] != "plane-box-both" {
            args.retain(|(k, _)| *k != "vertices")
        }
        args.push(("box", object(vec![("min", xyz(min)), ("max", xyz(max))])));
        let dir = match c[1] {
            "plane-box-facing" => Some([0., 0., 10.]),
            "plane-box-away" => Some([0., 0., -10.]),
            "plane-box-zero-normal" => Some([0.; 3]),
            "plane-box-facing-180" => Some([0., 0., -1.]),
            _ => None,
        };
        if let Some(dir) = dir {
            let mut normal = vec![("dir", xyz(dir))];
            if c[1] == "plane-box-facing-180" {
                normal.push(("withinDeg", V::Number(180.)))
            }
            args.push(("normal", object(normal)))
        }
    }
    if c[1] == "plane-normal-only" {
        args.push((
            "normal",
            object(vec![(
                "dir",
                V::Array(vec![V::Number(0.), V::Number(0.), V::Number(1.)]),
            )]),
        ))
    }
    if c[1] == "plane-marked-missing" {
        args.retain(|(k, _)| *k != "vertices");
        args.push(("marked", V::Boolean(true)))
    }
    let mut first = args.clone();
    first.retain(|(k, _)| *k != "orient");
    if shape == "plane-orient-reference-unicode" {
        first.push(("name", string("ΟΣ")));
    }
    let first = object(first);
    let input = object(args);
    let mut steps = vec![];
    let shape = c[1];
    let mut actions = vec![];
    if shape.starts_with("plane-orient-reference-") {
        actions.push(("element.fit_marked", first));
    }
    if !shape.starts_with("plane-empty-") && !shape.starts_with("plane-query-") {
        actions.push(("element.fit_marked", input.clone()))
    }
    if shape == "plane-two" {
        actions.push(("element.fit_marked", input.clone()))
    }
    if shape.starts_with("plane-query-") {
        let xyz = |v: [f64; 3]| V::Array(v.into_iter().map(V::Number).collect());
        let limit = c.get(6).map_or(3, |v| v.parse::<usize>().unwrap());
        let fields = c.get(7).copied().unwrap_or("all");
        let (mut min, mut max) = ([-100000.; 3], [100000.; 3]);
        match shape {
            "plane-query-empty" | "plane-query-empty-marked-fit" => {
                min = [10000.; 3];
                max = [10001.; 3]
            }
            "plane-query-inverted" => {
                min = [1., 0., 0.];
                max = [0., 1., 1.]
            }
            _ => {}
        }
        let mut query = vec![
            ("box", object(vec![("min", xyz(min)), ("max", xyz(max))])),
            ("limit", number(limit)),
        ];
        if fields != "default" {
            query.push(("fields", string(fields)))
        }
        if shape == "plane-query-ball" {
            query.retain(|(k, _)| *k != "box");
            query.push((
                "near",
                object(vec![("at", xyz([2., 2., 0.])), ("radius", V::Number(2.5))]),
            ))
        }
        if shape == "plane-query-facing" || shape == "plane-query-zero-normal" {
            query.push((
                "normal",
                object(vec![(
                    "dir",
                    xyz(if shape == "plane-query-facing" {
                        [0., 0., 10.]
                    } else {
                        [0.; 3]
                    }),
                )]),
            ))
        }
        if shape.contains("marked") {
            query.push(("mark", V::Boolean(true)))
        }
        if let Some(key) = shape.strip_prefix("plane-query-undefined-") {
            if let Some((_, value)) = query.iter_mut().find(|(k, _)| *k == key) {
                *value = V::Undefined;
            } else {
                query.push((key, V::Undefined));
            }
            if key == "box" {
                query.push((
                    "near",
                    object(vec![("at", xyz([2., 2., 0.])), ("radius", V::Number(100.))]),
                ));
            }
        }
        actions.push(("scan.query", object(query)));
        if shape == "plane-query-marked-stale" {
            actions.push((
                "scan.open",
                object(vec![
                    ("bytes", V::Bytes(scan_bytes(row))),
                    ("name", string("replacement.stl")),
                    ("units", string(c[2])),
                ]),
            ))
        }
        if shape.contains("marked") {
            actions.push((
                "element.fit_marked",
                object(vec![
                    ("kind", string("plane")),
                    ("marked", V::Boolean(true)),
                    ("sigma", V::Number(c[5].parse().unwrap())),
                ]),
            ))
        }
    }
    for (name, input) in actions {
        let (result, error) = outcome(engine.execute(name, input));
        let f = frame(&engine, result, error);
        let V::Object(fields) = f else {
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
    if std::env::var_os("SCANRULER_NATIVE_PLANE_CANARY").is_some() {
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
#[ignore = "complete pinned registry/session/worker requires Node22"]
fn pinned_native_plane_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root.join(format!("rust/target/native-plane-diff/run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    eprintln!("raw native-plane comparison {}", run.display());
    for (name, args) in [
        ("candidate.txt", vec!["rev-parse", "HEAD"]),
        ("candidate.diff", vec!["diff", "--binary"]),
    ] {
        let out = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(out.status.success());
        std::fs::write(run.join(name), out.stdout).unwrap()
    }
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
        .env(
            "SCANRULER_WORKER_SOURCE_FULL",
            run.join("original-command-traces.jsonl"),
        )
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
    assert_eq!(source.lines().count(), 205);
    let mut failures = 0;
    for (row, line) in INPUT.lines().zip(source.lines()) {
        let (id, h) = line.split_once('\t').unwrap();
        assert_eq!(row.split('\t').next().unwrap(), id);
        let expected = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut actual = vec![];
        wire(&observe_plane(row), &mut actual);
        let case = run.join(id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, b) in [
            ("input.bin", row.as_bytes()),
            ("original.bin", expected.as_slice()),
            ("rust.bin", actual.as_slice()),
        ] {
            std::fs::write(case.join(name), b).unwrap()
        }
        let compared = Command::new("python3")
            .arg(root.join("docs/port-plan/oracle/compare-native-plane.py"))
            .arg(case.join("original.bin"))
            .arg(case.join("rust.bin"))
            .output()
            .unwrap();
        std::fs::write(case.join("comparison.txt"), &compared.stdout).unwrap();
        if !compared.status.success() {
            failures += 1;
            eprintln!(
                "mismatch {id}: {}{}",
                String::from_utf8_lossy(&compared.stdout),
                String::from_utf8_lossy(&compared.stderr)
            );
        }
    }
    eprintln!(
        "205 actual registered select/plane controls, with history record patches, existing kernel tolerance; mismatches={failures}; raw {}",
        run.display()
    );
    assert_eq!(failures, 0)
}
