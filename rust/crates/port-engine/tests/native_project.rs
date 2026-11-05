// SPDX-License-Identifier: AGPL-3.0-only
//! One original-versus-Rust M3d corpus, including the actual original project hook.
#![allow(clippy::unwrap_used, clippy::panic)]
use port_engine::{
    guarded::Fault,
    history_record::Cell as C,
    native_engine::Engine,
    project::{self, Project},
    scan_session::{SourceFile, StoreId},
    schema_value::Value as V,
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const INPUT: &str = include_str!("../../../../docs/port-plan/fixtures/native-project/cases.tsv");
fn t(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn st(s: &str) -> V {
    V::String(t(s))
}
fn obj(f: Vec<(&str, V)>) -> V {
    V::Object(f.into_iter().map(|(k, v)| (t(k), v)).collect())
}
fn get<'a>(v: &'a V, k: &str) -> &'a V {
    if let V::Object(f) = v
        && let Some((_, v)) = f.iter().find(|(n, _)| *n == t(k))
    {
        return v;
    }
    &V::Undefined
}
fn put(v: &mut V, k: &str, x: V) {
    if let V::Object(f) = v {
        if let Some((_, v)) = f.iter_mut().find(|(n, _)| *n == t(k)) {
            *v = x
        } else {
            f.push((t(k), x))
        }
    }
}
fn remove(v: &mut V, k: &str) {
    if let V::Object(f) = v {
        f.retain(|(n, _)| *n != t(k))
    }
}

fn cell(v: &C) -> V {
    match v {
        C::Undefined => obj(vec![("$undefined", V::Boolean(true))]),
        C::Null => V::Null,
        C::Boolean(b) => V::Boolean(*b),
        C::Number(n) => V::Number(*n),
        C::String(s) => V::String(s.clone()),
        C::Bytes(b) => V::Array(b.iter().map(|v| V::Number(f64::from(*v))).collect()),
        C::Words(b) => V::Array(b.iter().map(|v| V::Number(f64::from(*v))).collect()),
        C::Array(a) => V::Array(a.iter().map(cell).collect()),
        C::Object(f) => V::Object(f.iter().map(|(k, v)| (k.clone(), cell(v))).collect()),
    }
}
fn error(e: Fault) -> V {
    match e {
        Fault::Command(e) => obj(vec![
            ("code", st(e.code().as_str())),
            ("message", V::String(e.message_units().to_vec())),
            ("name", st("CommandError")),
        ]),
        _ => panic!("unexpected native host failure {e:?}"),
    }
}
fn file(v: &V) -> Vec<u8> {
    let V::Bytes(b) = get(get(v, "file"), "bytes") else {
        panic!("no project bytes")
    };
    b.clone()
}
fn source(s: &SourceFile) -> V {
    let mut v = obj(vec![
        ("name", V::String(s.name.clone())),
        ("bytes", cell(&C::Bytes(s.bytes.clone()))),
    ]);
    if let Some(u) = s.units {
        put(&mut v, "units", st(u.id()))
    }
    v
}
fn snapshot(e: &Engine) -> V {
    let d = e.document();
    let s = d.sources();
    let g = e.logical_graph();
    let graph = g.map_or(V::Null, |g| {
        let floats = |a: &[f32]| V::Array(a.iter().map(|v| V::Number(f64::from(*v))).collect());
        let ints = |a: &[u32]| V::Array(a.iter().map(|v| V::Number(f64::from(*v))).collect());
        obj(vec![
            ("positions", floats(&g.positions)),
            ("indices", ints(&g.indices)),
            ("normals", floats(&g.normals)),
            ("adjOffsets", ints(&g.adjacency.offsets)),
            ("adjList", ints(&g.adjacency.list)),
            ("vertexCount", V::Number(g.vertex_count as f64)),
            ("triangleCount", V::Number(g.triangle_count as f64)),
            ("bboxDiag", V::Number(g.bbox_diag)),
        ])
    });
    let h = e.history();
    obj(vec![
        (
            "sources",
            obj(vec![
                ("scan", s.scan.as_ref().map_or(V::Null, |s| source(s))),
                ("reference", V::Null),
                ("image", V::Null),
            ]),
        ),
        ("app", cell(&C::Object(d.store(StoreId::App).snapshot()))),
        (
            "deviation",
            cell(&C::Object(d.store(StoreId::Deviation).snapshot())),
        ),
        (
            "thickness",
            cell(&C::Object(d.store(StoreId::Thickness).snapshot())),
        ),
        ("mark", cell(&C::Object(d.store(StoreId::Mark).snapshot()))),
        ("graph", graph),
        (
            "history",
            obj(vec![
                (
                    "past",
                    V::Array(h.past.iter().map(|e| V::String(e.label.clone())).collect()),
                ),
                (
                    "future",
                    V::Array(
                        h.future
                            .iter()
                            .map(|e| V::String(e.label.clone()))
                            .collect(),
                    ),
                ),
            ]),
        ),
    ])
}
fn observe(row: &str, dir: &Path) -> V {
    let (id, units) = row.split_once('\t').unwrap();
    let e = Engine::new().unwrap();
    e.execute(
        "scan.open",
        obj(vec![
            (
                "bytes",
                V::Bytes(
                    include_bytes!("../../../../docs/port-plan/fixtures/native-project/scan.ply")
                        .to_vec(),
                ),
            ),
            ("name", st("scan.ply")),
            ("units", st(units)),
        ]),
    )
    .unwrap();
    if id != "scan" {
        e.execute(
            "element.fit_marked",
            obj(vec![
                ("kind", st("plane")),
                (
                    "vertices",
                    V::Array((0..25).map(|i| V::Number(i as f64)).collect()),
                ),
                ("sigma", V::Number(0.)),
            ]),
        )
        .unwrap();
    }
    if id == "undo" || id == "redo" {
        e.execute("history.undo", obj(vec![])).unwrap();
    }
    if id == "redo" {
        e.execute("history.redo", obj(vec![])).unwrap();
    }
    let initial = e.execute("project.save", obj(vec![])).unwrap();
    let mut p = project::unpack(&file(&initial)).unwrap();
    let m = &mut p.manifest;
    if ["unknown", "empty-unknown"].contains(&id) {
        put(
            m,
            "futurePlugin",
            obj(vec![
                ("text", V::String(vec![107, 101, 112, 116, 0xd800])),
                ("nested", obj(vec![("answer", V::Number(42.))])),
            ]),
        );
        p.members.push(("plugin.bin".into(), vec![0, 255, 17, 0]));
        put(m, "name", st("  Bench / one  "));
        put(m, "workspace", st("future-workspace"));
    }
    if id == "utf16-name" {
        let mut name = t("  Bench ");
        name.push(0xd800);
        name.extend(t(" / one  "));
        put(m, "name", V::String(name));
    }
    if id == "legacy" {
        let mut scan = get(m, "scan").clone();
        let V::Array(mut els) = get(&scan, "elements").clone() else {
            panic!()
        };
        let mut src = get(&els[0], "source").clone();
        remove(&mut src, "settings");
        put(&mut els[0], "source", src);
        put(&mut scan, "elements", V::Array(els));
        for k in ["showBackfaces", "sections", "nextSectionNumber"] {
            remove(&mut scan, k)
        }
        put(m, "scan", scan);
        let mut d = get(m, "deviation").clone();
        remove(&mut d, "mapFacingDeg");
        put(m, "deviation", d);
        let mut f = get(m, "flat").clone();
        for k in [
            "notes",
            "nextNoteId",
            "turns",
            "mirror",
            "sheets",
            "subject",
        ] {
            remove(&mut f, k)
        }
        put(m, "flat", f);
    }
    if id == "pose" {
        let mut scan = get(m, "scan").clone();
        put(
            &mut scan,
            "appliedAlignment",
            project::parse_json(br#"{"r":[0,-1,0,1,0,0,0,0,1],"t":[10,20,30]}"#).unwrap(),
        );
        put(m, "scan", scan)
    }
    if id == "read-as" {
        let mut scan = get(m, "scan").clone();
        put(&mut scan, "fileName", st("previous.stl"));
        put(m, "scan", scan)
    }
    if id == "empty" || id == "empty-unknown" {
        put(m, "scan", V::Null);
        p.members.remove(0);
    }
    if id == "newer" {
        put(m, "schemaVersion", V::Number(2.))
    }
    if id == "reserved" {
        put(
            m,
            "futurePlugin",
            obj(vec![("nested", obj(vec![("constructor", V::Number(1.))]))]),
        )
    }
    if id == "missing" {
        p.members.remove(0);
    }
    if id == "bad-mesh" {
        p.members[0].1 = b"ply\nnot a header\n".to_vec();
    }
    if id == "path" {
        p.members.push(("bad/file".into(), vec![1]));
    }
    if id == "duplicate" {
        p.members.push(("another.json".into(), vec![1]));
    }
    let mut bytes = project::pack(&p).unwrap();
    if id == "not-json" {
        let p = Project {
            manifest: V::Null,
            members: vec![("project.json".into(), b"{".to_vec())],
        };
        bytes = project::pack(&p).unwrap();
    }
    if id == "corrupt" {
        bytes = vec![0, 1, 2, 3];
    }
    if id == "duplicate" {
        for i in 0..=bytes.len() - 12 {
            if &bytes[i..i + 12] == b"another.json" {
                bytes[i..i + 12].copy_from_slice(b"project.json")
            }
        }
    }
    if id == "big-manifest" {
        let i = (0..bytes.len() - 28)
            .find(|&i| bytes[i..i + 4] == [0x50, 0x4b, 1, 2])
            .unwrap();
        bytes[i + 24..i + 28].copy_from_slice(&(64u32 * 1024 * 1024 + 1).to_le_bytes());
    }
    if id == "original-deflate" {
        bytes = include_bytes!(
            "../../../../docs/port-plan/fixtures/native-project/authored-deflated.scanruler"
        )
        .to_vec();
    }
    let (mut result, err) = match e.execute(
        "project.load",
        obj(vec![
            ("bytes", V::Bytes(bytes)),
            ("name", st("project.scanruler")),
            ("discard", V::Boolean(id != "held")),
        ]),
    ) {
        Ok(v) => (v, V::Null),
        Err(e) => (V::Null, error(e)),
    };
    remove(&mut result, "runningSince");
    remove(&mut result, "runningMs");
    if std::env::var_os("SCANRULER_M3D_CANARY").is_some() {
        e.document()
            .store(StoreId::App)
            .patch(vec![(t("nextId"), C::Number(-99.))])
            .unwrap();
    }
    let state = snapshot(&e);
    let (saved, save_error) = match e.execute("project.save", obj(vec![])) {
        Ok(v) => {
            let bytes = file(&v);
            std::fs::write(dir.join("saved.scanruler"), &bytes).unwrap();
            let p = project::unpack(&bytes).unwrap();
            let hex = |b: &[u8]| b.iter().map(|b| format!("{b:02x}")).collect::<String>();
            (
                obj(vec![
                    ("name", get(get(&v, "file"), "name").clone()),
                    ("holds", get(&v, "holds").clone()),
                    ("aligned", get(&v, "aligned").clone()),
                    ("manifest", p.manifest),
                    (
                        "members",
                        V::Array(
                            p.members
                                .into_iter()
                                .map(|(n, b)| V::Array(vec![st(&n), st(&hex(&b))]))
                                .collect(),
                        ),
                    ),
                ]),
                V::Null,
            )
        }
        Err(e) => (V::Null, error(e)),
    };
    obj(vec![
        ("id", st(id)),
        ("result", result),
        ("error", err),
        ("state", state),
        ("saved", saved),
        ("saveError", save_error),
    ])
}
#[test]
#[ignore = "frozen original requires Node22"]
fn pinned_native_project_comparison() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = root.join(format!("rust/target/native-project-diff/run-{stamp}"));
    std::fs::create_dir_all(&run).unwrap();
    let mut rust = String::new();
    for row in INPUT.lines() {
        let id = row.split('\t').next().unwrap();
        let d = run.join(id);
        std::fs::create_dir_all(&d).unwrap();
        let v = observe(row, &d);
        let s = project::manifest_text(&v);
        std::fs::write(d.join("rust.json"), &s).unwrap();
        rust.push_str(&s.replace('\n', ""));
        rust.push('\n');
    }
    std::fs::write(run.join("rust.jsonl"), rust).unwrap();
    let build = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-native-project.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/native-project.mjs"))
        .arg(&run)
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
    std::fs::write(run.join("original.stderr"), &out.stderr).unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::write(run.join("original.jsonl"), out.stdout).unwrap();
    std::fs::write(run.join("cases.tsv"), INPUT).unwrap();
    let status = Command::new("python3")
        .arg(root.join("docs/port-plan/oracle/compare-native-project.py"))
        .arg(&run)
        .status()
        .unwrap();
    eprintln!("M3d comparison raw {}", run.display());
    assert!(status.success());
}
