// SPDX-License-Identifier: AGPL-3.0-only
//! Independent actual geometry and prepared-worker ownership controls.
#![allow(clippy::unwrap_used, clippy::panic)]
use port_core::mesh_graph::{GraphLimits, MeshGraph, ParsedMesh, build_mesh_graph};
use port_engine::scan_worker;
use port_engine::schema_value::Value as V;
use scan_worker::{Change, CommitError, Error, Limits, Resident, Worker};
use std::sync::Arc;
const INPUT: &str =
    include_str!("../../../../../docs/port-plan/fixtures/scan-session-owned/cases.tsv");
fn rows() -> impl Iterator<Item = &'static str> {
    INPUT
        .lines()
        .filter(|r| r.split('\t').nth(1) == Some("grid-staging"))
}
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn string(s: &str) -> V {
    V::String(text(s))
}
fn number(n: usize) -> V {
    V::Number(n as f64)
}
fn object(v: Vec<(&str, V)>) -> V {
    V::Object(v.into_iter().map(|(k, v)| (text(k), v)).collect())
}
fn undefined() -> V {
    object(vec![("$undefined", V::Boolean(true))])
}
fn hex(b: &[u8]) -> String {
    const H: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(b.len() * 2);
    for &v in b {
        out.push(H[(v >> 4) as usize] as char);
        out.push(H[(v & 15) as usize] as char)
    }
    out
}
fn array32(v: impl Iterator<Item = [u8; 4]>, kind: &str) -> V {
    object(vec![
        ("$type", string(kind)),
        ("hex", string(&hex(&v.flatten().collect::<Vec<_>>()))),
    ])
}
fn graph(g: &MeshGraph) -> V {
    object(vec![
        (
            "positions",
            array32(g.positions.iter().map(|v| v.to_le_bytes()), "Float32Array"),
        ),
        (
            "indices",
            array32(g.indices.iter().map(|v| v.to_le_bytes()), "Uint32Array"),
        ),
        (
            "normals",
            array32(g.normals.iter().map(|v| v.to_le_bytes()), "Float32Array"),
        ),
        (
            "adjOffsets",
            array32(
                g.adjacency.offsets.iter().map(|v| v.to_le_bytes()),
                "Uint32Array",
            ),
        ),
        (
            "adjList",
            array32(
                g.adjacency.list.iter().map(|v| v.to_le_bytes()),
                "Uint32Array",
            ),
        ),
        ("vertexCount", number(g.vertex_count)),
        ("triangleCount", number(g.triangle_count)),
        ("bboxDiag", V::Number(g.bbox_diag)),
    ])
}
fn prepared(row: &str, scale: f64) -> Arc<MeshGraph> {
    let c = row.split('\t').collect::<Vec<_>>();
    let n = c.get(4).map_or(5, |v| v.parse::<usize>().unwrap());
    let mut seed = c.get(3).map(|v| v.parse::<u32>().unwrap());
    let mut points = Vec::new();
    for y in 0..n {
        for x in 0..n {
            let z = if let Some(s) = &mut seed {
                *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                ((f64::from(*s) / 4294967296. - 0.5) * 0.1) as f32
            } else {
                0.
            };
            points.push([x as f32, y as f32, z]);
        }
    }
    let mut p = Vec::new();
    for y in 0..n - 1 {
        for x in 0..n - 1 {
            let a = y * n + x;
            for i in [a, a + 1, a + n, a + 1, a + n + 1, a + n] {
                for v in points[i] {
                    p.push((f64::from(v) * scale) as f32)
                }
            }
        }
    }
    Arc::new(
        build_mesh_graph(
            ParsedMesh {
                positions: p,
                soup: true,
                indices: None,
                keep_winding: false,
            },
            GraphLimits {
                vertices: 1_000_000,
                indices: 3_000_000,
                neighbors: 6_000_000,
            },
            |_| {},
        )
        .unwrap(),
    )
}
type State = Worker<MeshGraph, Vec<u8>>;
fn frame(w: &State, name: &str, error: Option<&Error>) -> V {
    let versions = w.versions();
    object(vec![
        ("name", string(name)),
        (
            "error",
            error.map_or_else(undefined, |e| {
                object(vec![
                    ("code", undefined()),
                    ("message", string(e.source_message().unwrap())),
                    ("name", string("Error")),
                ])
            }),
        ),
        (
            "versions",
            object(vec![
                ("scan", number(versions.scan as usize)),
                ("nominal", number(versions.nominal as usize)),
            ]),
        ),
        ("nextRequestId", number(w.next_request() as usize)),
        (
            "worker",
            object(vec![
                ("graph", w.graph().as_deref().map_or(V::Null, graph)),
                ("nominalPresent", V::Boolean(w.nominal().is_some())),
                ("solidPresent", V::Boolean(w.solid().is_some())),
                (
                    "stagedScans",
                    V::Array(
                        w.staged_scans()
                            .iter()
                            .map(|(id, g)| V::Array(vec![number(*id as usize), graph(g)]))
                            .collect(),
                    ),
                ),
                (
                    "stagedNominals",
                    V::Array(
                        w.staged_nominals()
                            .iter()
                            .map(|(id, _)| number(*id as usize))
                            .collect(),
                    ),
                ),
            ]),
        ),
    ])
}
fn commit(w: &mut State, scan: Change, nominal: Change) -> Option<Error> {
    w.reserve_request().unwrap();
    match w.commit(scan, nominal, |_| Ok::<(), ()>(())) {
        Ok(()) => None,
        Err(CommitError::Lifecycle(e)) => Some(e),
        Err(CommitError::WorkerFailure(())) => panic!("no fault provider"),
    }
}
fn observe(row: &str) -> V {
    let mut w = State::new(Limits::default());
    let mut out = vec![];
    let a = w.reserve_request().unwrap();
    w.stage_scan(a, prepared(row, 1.)).unwrap();
    out.push(frame(&w, "prepareScan", None));
    let b = w.reserve_request().unwrap();
    w.stage_scan(b, prepared(row, 10.)).unwrap();
    out.push(frame(&w, "prepareScan", None));
    for (scan, nominal) in [
        (Change::Prepared(a), Change::Prepared(999999)),
        (Change::Prepared(a), Change::Unchanged),
        (Change::Prepared(a), Change::Unchanged),
    ] {
        let error = commit(&mut w, scan, nominal);
        out.push(frame(&w, "commitImport", error.as_ref()));
    }
    w.reserve_request().unwrap();
    w.discard(&[b]).unwrap();
    out.push(frame(&w, "discardImport", None));
    for (scan, nominal) in [
        (Change::Clear, Change::Clear),
        (Change::Unchanged, Change::Unchanged),
    ] {
        let error = commit(&mut w, scan, nominal);
        out.push(frame(&w, "commitImport", error.as_ref()));
    }
    w.discard(&[]).unwrap();
    out.push(frame(&w, "discardImport", None));
    if std::env::var_os("SCANRULER_SCAN_WORKER_SABOTAGE").is_some()
        && let V::Object(fields) = &mut out[3]
    {
        let (_, v) = fields
            .iter_mut()
            .find(|(k, _)| *k == text("nextRequestId"))
            .unwrap();
        *v = number(999);
    }
    V::Array(out)
}
fn wire(v: &V, out: &mut Vec<u8>) {
    let len = |out: &mut Vec<u8>, n: usize| out.extend((n as u32).to_le_bytes());
    match v {
        V::Undefined => out.push(0),
        V::Null => out.push(1),
        V::Boolean(v) => out.push(if *v { 2 } else { 3 }),
        V::Number(v) => {
            out.push(4);
            out.extend(v.to_le_bytes())
        }
        V::String(s) => {
            out.push(5);
            len(out, s.len());
            for v in s {
                out.extend(v.to_le_bytes())
            }
        }
        V::Array(a) => {
            out.push(6);
            len(out, a.len());
            for v in a {
                wire(v, out)
            }
        }
        V::Object(a) => {
            out.push(7);
            len(out, a.len());
            for (k, v) in a {
                wire(&V::String(k.clone()), out);
                wire(v, out)
            }
        }
        V::Bytes(_) => panic!("projection uses typed buffer descriptors"),
    }
}
#[test]
fn native_prepared_worker_profiles() {
    assert_eq!(rows().count(), 49);
    for row in rows() {
        let V::Array(frames) = observe(row) else {
            panic!("frames")
        };
        assert_eq!(frames.len(), 9)
    }
}
#[test]
fn actual_resources_budget_identity_prefix_and_generations() {
    let row = rows().next().unwrap();
    let mut w = State::new(Limits::default());
    let a = prepared(row, 1.);
    let nominal = prepared(row, 2.);
    assert!(a.resident_bytes().unwrap() > 0);
    w.stage_scan(1, a.clone()).unwrap();
    w.stage_nominal(2, nominal.clone()).unwrap();
    let cache = Arc::new(vec![1, 2, 3]);
    let weak = Arc::downgrade(&cache);
    w.set_solid(Some(cache)).unwrap();
    let ticket = w.versions();
    let outcome = w.commit(Change::Prepared(1), Change::Prepared(2), |state| {
        assert!(Arc::ptr_eq(state.graph().as_ref().unwrap(), &a));
        assert!(state.solid().is_none());
        assert!(state.nominal().is_none());
        Err("plugin failed")
    });
    assert_eq!(outcome, Err(CommitError::WorkerFailure("plugin failed")));
    assert!(weak.upgrade().is_none());
    assert!(w.is_current(ticket));
    assert_eq!(w.staged_nominals().len(), 1);
    assert!(w.staged_scans().is_empty());
    w.commit(Change::Unchanged, Change::Prepared(2), |_| Ok::<(), ()>(()))
        .unwrap();
    assert!(Arc::ptr_eq(w.nominal().as_ref().unwrap(), &nominal));
    assert!(!w.is_current(ticket));
    assert!(w.is_scan_current(ticket.scan));
    assert!(!w.is_nominal_current(ticket.nominal));
    w.stage_scan(3, a.clone()).unwrap();
    assert!(Arc::ptr_eq(&w.take_staged(3).unwrap(), &a));
    assert!(w.take_staged(3).is_none());
    let mut limited = State::new(Limits {
        prepared: 1,
        ..Limits::default()
    });
    limited.stage_scan(1, a.clone()).unwrap();
    assert_eq!(limited.stage_nominal(2, nominal), Err(Error::HostLimit));
    limited.stage_scan(1, a).unwrap();
    let mut requests = State::new(Limits {
        requests: 1,
        ..Limits::default()
    });
    assert_eq!(requests.reserve_request(), Ok(1));
    assert_eq!(requests.reserve_request(), Err(Error::HostLimit));
}
pub(super) fn compare_into(run: &std::path::Path) {
    use std::{
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
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
    let input = rows().collect::<Vec<_>>().join("\n") + "\n";
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/scan-worker-ownership.mjs"))
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
    let mut stdin = child.stdin.take().unwrap();
    let input_copy = input.clone();
    let writer = std::thread::spawn(move || stdin.write_all(input_copy.as_bytes()).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let source = String::from_utf8(output.stdout).unwrap();
    assert_eq!(source.lines().count(), 49);
    let mut mismatches = 0;
    for (row, line) in rows().zip(source.lines()) {
        let (id, h) = line.split_once('\t').unwrap();
        assert_eq!(id, format!("worker-{}", row.split('\t').next().unwrap()));
        let expected = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let mut actual = vec![];
        wire(&observe(row), &mut actual);
        let case = run.join(id);
        std::fs::create_dir_all(&case).unwrap();
        for (name, bytes) in [
            ("input.bin", row.as_bytes()),
            ("original.bin", &expected),
            ("rust.bin", &actual),
            ("original.stderr.txt", &output.stderr),
        ] {
            std::fs::write(case.join(name), bytes).unwrap()
        }
        if expected != actual {
            mismatches += 1;
            eprintln!("full lifecycle/geometry mismatch {id}")
        }
        let mut wrong = expected.clone();
        wrong[0] ^= 1;
        assert_ne!(wrong, expected);
        assert_ne!(&actual[..actual.len() - 1], expected);
    }
    eprintln!(
        "49 actual prepared-worker geometry/lifecycle profiles; mismatches={mismatches}; raw {}",
        run.display()
    );
    assert_eq!(mismatches, 0)
}

#[test]
fn aggregate_resource_and_discard_admission_do_not_partially_mutate() {
    let row = rows().next().unwrap();
    let a = prepared(row, 1.);
    let bytes = a.resident_bytes().unwrap();
    let mut w = State::new(Limits {
        resident_bytes: bytes,
        ..Limits::default()
    });
    w.stage_scan(1, a.clone()).unwrap();
    assert_eq!(w.stage_scan(2, a.clone()), Err(Error::HostLimit));
    assert_eq!(w.staged_scans().len(), 1);
    let mut w = State::new(Limits {
        discard_ids: 1,
        ..Limits::default()
    });
    w.stage_scan(1, a).unwrap();
    assert_eq!(w.discard(&[1, 2]), Err(Error::HostLimit));
    assert_eq!(w.staged_scans().len(), 1);
    w.discard(&[1]).unwrap();
    assert!(w.staged_scans().is_empty());
}
