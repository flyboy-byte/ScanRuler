// SPDX-License-Identifier: AGPL-3.0-only
//! Private user scan comparison: opt-in, no source mesh data in retained observations.
#![allow(clippy::unwrap_used, clippy::panic)]
use port_engine::{native_engine::Engine, project, schema_value::Value as V};
use std::process::Command;
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn string(s: &str) -> V {
    V::String(text(s))
}
fn object(fields: Vec<(&str, V)>) -> V {
    V::Object(fields.into_iter().map(|(k, v)| (text(k), v)).collect())
}
fn xyz(values: [f64; 3]) -> V {
    V::Array(values.into_iter().map(V::Number).collect())
}
#[test]
#[ignore = "private real scan; SCANRULER_REAL_SCAN=1 and local Artec files required"]
fn pinned_private_vase_comparison() {
    let root = xtask::root();
    let folder = root.join("fixtures/private/artec-textured-vase");
    if std::env::var("SCANRULER_REAL_SCAN").as_deref() != Ok("1")
        || !["vase.ply", "vase.stl"]
            .iter()
            .all(|f| folder.join(f).is_file())
    {
        println!(
            "SKIPPED private vase: set SCANRULER_REAL_SCAN=1 and supply both local Artec files"
        );
        return;
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
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out = root.join(format!("rust/target/private-vase-diff/run-{stamp}"));
    std::fs::create_dir_all(&out).unwrap();
    for extension in ["ply", "stl"] {
        let name = format!("vase.{extension}");
        let engine = Engine::new().unwrap();
        let opened = engine
            .execute(
                "scan.open",
                object(vec![
                    (
                        "bytes",
                        V::Bytes(std::fs::read(folder.join(&name)).unwrap()),
                    ),
                    ("name", string(&name)),
                    ("units", string("mm")),
                ]),
            )
            .unwrap();
        let V::Object(fields) = opened else {
            panic!("invalid open response")
        };
        let mut scan = fields
            .into_iter()
            .find(|(k, _)| *k == text("scan"))
            .unwrap()
            .1;
        let V::Object(ref mut fields) = scan else {
            panic!("invalid scan summary")
        };
        fields.push((text("components"), V::Number(engine.components() as f64)));
        let query = engine
            .execute(
                "scan.query",
                object(vec![
                    (
                        "box",
                        object(vec![
                            ("min", xyz([-130., -130., -84.])),
                            ("max", xyz([130., 130., -80.])),
                        ]),
                    ),
                    (
                        "normal",
                        object(vec![
                            ("dir", xyz([0., 0., -1.])),
                            ("withinDeg", V::Number(15.)),
                        ]),
                    ),
                    ("mark", V::Boolean(true)),
                    ("fields", string("count")),
                ]),
            )
            .unwrap();
        let fit = engine
            .execute(
                "element.fit_marked",
                object(vec![
                    ("kind", string("plane")),
                    ("marked", V::Boolean(true)),
                    ("sigma", V::Number(0.)),
                ]),
            )
            .unwrap();
        let result = object(vec![("scan", scan), ("selection", query), ("fit", fit)]);
        std::fs::write(
            out.join(format!("{extension}.rust.json")),
            project::manifest_text(&result),
        )
        .unwrap();
        let original = Command::new("node")
            .arg(root.join("docs/port-plan/oracle/private-vase.mjs"))
            .arg(extension)
            .current_dir(&root)
            .output()
            .unwrap();
        std::fs::write(
            out.join(format!("{extension}.original.stderr")),
            &original.stderr,
        )
        .unwrap();
        assert!(
            original.status.success(),
            "{}",
            String::from_utf8_lossy(&original.stderr)
        );
        std::fs::write(
            out.join(format!("{extension}.original.json")),
            original.stdout,
        )
        .unwrap();
    }
    let comparison = Command::new("python3")
        .arg(root.join("docs/port-plan/oracle/compare-private-vase.py"))
        .arg(&out)
        .status()
        .unwrap();
    println!("Private vase summary comparison {}", out.display());
    assert!(comparison.success());
}
