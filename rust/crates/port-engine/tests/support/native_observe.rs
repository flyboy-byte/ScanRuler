// SPDX-License-Identifier: AGPL-3.0-only
//! Shared observations of the actual native owner; original and native inputs stay independent.
#![allow(clippy::unwrap_used, clippy::panic)]
use port_core::mesh_graph::MeshGraph;
use port_engine::{
    history_record::Cell as C,
    native_engine::Engine,
    scan_session::{SourceFile, StoreId},
    schema_value::Value as V,
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
pub(crate) fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
pub(crate) fn string(s: &str) -> V {
    V::String(text(s))
}
pub(crate) fn number(n: usize) -> V {
    V::Number(n as f64)
}
pub(crate) fn object(v: Vec<(&str, V)>) -> V {
    V::Object(v.into_iter().map(|(k, v)| (text(k), v)).collect())
}
pub(crate) fn undefined() -> V {
    object(vec![("$undefined", V::Boolean(true))])
}
pub(crate) fn hex(b: &[u8]) -> String {
    const H: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(b.len() * 2);
    for &v in b {
        out.push(H[(v >> 4) as usize] as char);
        out.push(H[(v & 15) as usize] as char)
    }
    out
}
pub(crate) fn array32(v: impl Iterator<Item = [u8; 4]>, kind: &str) -> V {
    object(vec![
        ("$type", string(kind)),
        ("hex", string(&hex(&v.flatten().collect::<Vec<_>>()))),
    ])
}
pub(crate) fn graph(g: &MeshGraph) -> V {
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
pub(crate) fn wire(v: &V, out: &mut Vec<u8>) {
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

pub(crate) fn pack(c: &C) -> V {
    match c {
        C::Undefined => undefined(),
        C::Null => V::Null,
        C::Boolean(b) => V::Boolean(*b),
        C::Number(n) => packed_number(*n),
        C::String(s) => V::String(s.clone()),
        C::Bytes(b) => object(vec![
            ("$type", string("Uint8Array")),
            ("hex", string(&hex(b))),
        ]),
        C::Words(b) => array32(b.iter().map(|v| v.to_le_bytes()), "Uint32Array"),
        C::Array(v) => V::Array(v.iter().map(pack).collect()),
        C::Object(v) => V::Object(v.iter().map(|(k, v)| (k.clone(), pack(v))).collect()),
    }
}
pub(crate) fn source(s: &SourceFile) -> V {
    let mut f = vec![
        ("name", V::String(s.name.clone())),
        ("bytes", pack(&C::Bytes(s.bytes.clone()))),
    ];
    if let Some(u) = s.units {
        f.push(("units", string(u.id())))
    }
    assert!(s.companions.is_none());
    object(f)
}
pub(crate) fn bytes(row: &str) -> Vec<u8> {
    let c = row.split('\t').collect::<Vec<_>>();
    if c[1].starts_with("ply-") {
        return ply_bytes(c[1]);
    }
    if c[1] == "malformed" {
        return b"not an STL".to_vec();
    }
    let facet = |a: &str, b: &str, c: &str| {
        format!(
            "facet normal 0 0 1\nouter loop\nvertex {a}\nvertex {b}\nvertex {c}\nendloop\nendfacet\n"
        )
    };
    let mut faces = facet("0 0 0", "1 0 0", "0 1 0");
    if c[1] == "quad" {
        faces += &facet("1 0 0", "1 1 0", "0 1 0")
    }
    format!("solid {}\n{faces}endsolid\n", c[0]).into_bytes()
}
pub(crate) fn observe(row: &str) -> V {
    let c = row.split('\t').collect::<Vec<_>>();
    let engine = Engine::new().unwrap();
    if c[1].starts_with("held-") {
        let mut fields = vec![];
        let items = |n| {
            C::Array(std::sync::Arc::new(
                (0..n)
                    .map(|_| C::Object(std::sync::Arc::new(vec![])))
                    .collect(),
            ))
        };
        if ["held-mixed", "held-project", "held-discard"].contains(&c[1]) {
            fields.push((text("elements"), items(2)))
        }
        if c[1] != "held-sections" {
            fields.push((text("dimensions"), items(1)))
        }
        if c[1] != "held-dimensions" {
            let section = |i| {
                C::Object(std::sync::Arc::new(vec![
                    (text("id"), C::Number(i as f64)),
                    (text("name"), C::String(text(&format!("Section {i}")))),
                    (text("color"), C::String(text("#ffffff"))),
                    (text("visible"), C::Boolean(true)),
                    (
                        text("frame"),
                        C::Object(std::sync::Arc::new(
                            [
                                ("origin", [0., 0., 0.]),
                                ("normal", [0., 0., 1.]),
                                ("basisU", [1., 0., 0.]),
                                ("basisV", [0., 1., 0.]),
                            ]
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    text(k),
                                    C::Array(std::sync::Arc::new(
                                        v.into_iter().map(C::Number).collect(),
                                    )),
                                )
                            })
                            .collect(),
                        )),
                    ),
                    (text("message"), C::String(text("No scan is loaded."))),
                ]))
            };
            fields.push((
                text("sections"),
                C::Array(std::sync::Arc::new((1..=3).map(section).collect())),
            ))
        }
        if c[1] == "held-project" {
            fields.push((text("projectName"), C::String(text("Bench “A”"))))
        }
        engine.document().store(StoreId::App).patch(fields).unwrap();
    }

    let out = engine.execute(
        "scan.open",
        object(vec![
            ("bytes", V::Bytes(bytes(row))),
            (
                "name",
                string(&format!(
                    "{}.{}",
                    c[0],
                    if c[1].starts_with("ply-") {
                        "ply"
                    } else {
                        "stl"
                    }
                )),
            ),
            ("units", string(c[2])),
            ("discard", V::Boolean(c[1] == "held-discard")),
        ]),
    );
    let (result, error) = outcome(out);
    frame(&engine, result, error)
}
pub(crate) fn outcome(out: Result<V, port_engine::guarded::Fault>) -> (V, V) {
    match out {
        Ok(v) => (v, undefined()),
        Err(port_engine::guarded::Fault::Command(e)) => (
            undefined(),
            object(vec![
                ("code", string(e.code().as_str())),
                ("message", V::String(e.message_units().to_vec())),
                ("name", string("CommandError")),
            ]),
        ),
        Err(port_engine::guarded::Fault::Host(e)) => (
            undefined(),
            object(vec![
                ("code", string("$nativeHostFailure")),
                ("message", string(&format!("{e:?}"))),
                ("name", string("NativeHostFailure")),
            ]),
        ),
        Err(e) => panic!("unexpected non-command fault: {e:?}"),
    }
}
pub(crate) fn frame(engine: &Engine, result: V, error: V) -> V {
    let doc = engine.document();
    if std::env::var_os("SCANRULER_NATIVE_IMPORT_CANARY").is_some() {
        doc.store(StoreId::App)
            .patch(vec![(text("vertexCount"), C::Number(-99.))])
            .unwrap()
    }
    let s = doc.sources();
    let sources = object(vec![
        ("scan", s.scan.as_ref().map_or(V::Null, |s| source(s))),
        (
            "reference",
            s.reference.as_ref().map_or(V::Null, |s| source(s)),
        ),
        ("image", s.image.as_ref().map_or(V::Null, |s| source(s))),
    ]);
    let mut stores = vec![];
    for (name, id) in [
        ("app", StoreId::App),
        ("deviation", StoreId::Deviation),
        ("thickness", StoreId::Thickness),
        ("mark", StoreId::Mark),
    ] {
        let fields = doc.store(id).snapshot();
        stores.push((
            name,
            V::Object(fields.iter().map(|(k, v)| (k.clone(), pack(v))).collect()),
        ))
    }
    let h = engine.history();

    stores.push((
        "history",
        object(vec![
            ("past", entries(&h.past)),
            ("future", entries(&h.future)),
            ("restoring", V::Boolean(h.restoring)),
            ("importing", V::Boolean(h.importing)),
            ("revision", number(h.revision as usize)),
        ]),
    ));
    let versions = engine.versions();
    object(vec![
        ("result", packed(result)),
        ("error", error),
        (
            "versions",
            object(vec![
                ("scan", number(versions.scan as usize)),
                ("nominal", number(versions.nominal as usize)),
            ]),
        ),
        ("nextRequestId", number(engine.next_request_id() as usize)),
        ("sources", sources),
        ("stores", object(stores)),
        (
            "mesh",
            doc.asset().map_or(V::Null, |a| {
                object(vec![
                    (
                        "positions",
                        array32(a.positions.iter().map(|v| v.to_le_bytes()), "Float32Array"),
                    ),
                    (
                        "indices",
                        array32(a.indices.iter().map(|v| v.to_le_bytes()), "Uint32Array"),
                    ),
                ])
            }),
        ),
        (
            "worker",
            object(vec![
                (
                    "graph",
                    engine
                        .logical_graph()
                        .as_ref()
                        .map_or(V::Null, |g| graph(g)),
                ),
                (
                    "nominalPresent",
                    V::Boolean(engine.inspect_worker(|w| w.nominal().is_some())),
                ),
                (
                    "solidPresent",
                    V::Boolean(engine.inspect_worker(|w| w.solid().is_some())),
                ),
                (
                    "stagedScans",
                    engine.inspect_worker(|w| {
                        V::Array(
                            w.staged_scans()
                                .iter()
                                .map(|(id, g)| V::Array(vec![number(*id as usize), graph(g)]))
                                .collect(),
                        )
                    }),
                ),
                (
                    "stagedNominals",
                    engine.inspect_worker(|w| {
                        V::Array(
                            w.staged_nominals()
                                .iter()
                                .map(|(id, _)| number(*id as usize))
                                .collect(),
                        )
                    }),
                ),
            ]),
        ),
    ])
}

pub(crate) fn ply_bytes(shape: &str) -> Vec<u8> {
    if shape == "ply-malformed" {
        return b"ply\nnot a header\n".to_vec();
    }
    let encoding = match shape {
        "ply-le" => "binary_little_endian",
        "ply-be" => "binary_big_endian",
        _ => "ascii",
    };
    let mut bytes=format!("ply\nformat {encoding} 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n").into_bytes();
    if encoding == "ascii" {
        bytes.extend_from_slice(b"0 0 0\n1 0 0\n0 1 0\n1 1 0\n3 0 1 2\n3 1 3 2\n");
        return bytes;
    }
    for v in [0_f32, 0., 0., 1., 0., 0., 0., 1., 0., 1., 1., 0.] {
        bytes.extend_from_slice(&if shape == "ply-le" {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        })
    }
    for tri in [[0_i32, 1, 2], [1, 3, 2]] {
        bytes.push(3);
        for v in tri {
            bytes.extend_from_slice(&if shape == "ply-le" {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            })
        }
    }
    bytes
}

pub(crate) fn entries(entries: &[port_engine::history_record::Entry]) -> V {
    V::Array(
        entries
            .iter()
            .map(|e| {
                object(vec![
                    (
                        "before",
                        V::Object(e.before.iter().map(|(k, v)| (k.clone(), pack(v))).collect()),
                    ),
                    (
                        "after",
                        V::Object(e.after.iter().map(|(k, v)| (k.clone(), pack(v))).collect()),
                    ),
                    ("label", V::String(e.label.clone())),
                    ("workspace", V::String(e.workspace.clone())),
                    ("group", number(e.group as usize)),
                ])
            })
            .collect(),
    )
}

pub(crate) fn packed_number(n: f64) -> V {
    if n == 0. && n.is_sign_negative() {
        object(vec![("$number", string("-0"))])
    } else if n.is_nan() {
        object(vec![("$number", string("NaN"))])
    } else if n.is_infinite() {
        object(vec![(
            "$number",
            string(if n.is_sign_negative() {
                "-Infinity"
            } else {
                "Infinity"
            }),
        )])
    } else {
        V::Number(n)
    }
}
pub(crate) fn packed(v: V) -> V {
    match v {
        V::Number(n) => packed_number(n),
        V::Array(v) => V::Array(v.into_iter().map(packed).collect()),
        V::Object(v) => V::Object(v.into_iter().map(|(k, v)| (k, packed(v))).collect()),
        _ => v,
    }
}
