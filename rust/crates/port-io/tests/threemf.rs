// SPDX-License-Identifier: AGPL-3.0-only
//! 3MF import (`io.threemf`) against the original's `parse3MF`.
//!
//! The ordinary tests pin hand-built packages. The ignored comparison generates 3MF packages
//! (units, namespaces, components, production-extension paths, colour groups, textures, slicer
//! parts, XML decoration), mutates their XML and their zip bytes, and runs the original and the
//! port on each: success or failure, the full error text, the progress text, positions, indices,
//! colours and the texture note. Host-limit refusals and original timeouts are counted apart.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::cast_possible_truncation)]

#[path = "support/threemf_corpus.rs"]
mod corpus;

use port_io::inflate::ZipLimits;
use port_io::obj_texture::{ObjCompanions, TextureImage, TextureLimits, TextureNote};
use port_io::threemf::{LimitKind, ThreeMfError, ThreeMfLimits, read_3mf};

/// Generous host limits for the tests.
pub fn limits() -> ThreeMfLimits {
    ThreeMfLimits {
        zip: ZipLimits {
            max_entry_bytes: 16 * 1024 * 1024,
            max_entries: 4096,
        },
        max_part_bytes: 64 * 1024 * 1024,
        max_vertices: 2_000_000,
        max_triangles: 2_000_000,
        max_work: 200_000_000,
        texture: TextureLimits {
            max_bytes: 256 * 1024 * 1024,
            max_vertices: 2_000_000,
            max_files: 4096,
            max_materials: 4096,
            max_name_bytes: 4096,
            max_work: 200_000_000,
        },
    }
}

/// A zip of stored (uncompressed) entries, in the given order; a repeated name stays repeated.
/// A zip of stored entries.
pub fn stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut z = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = z.len() as u32;
        z.extend(0x0403_4b50u32.to_le_bytes());
        z.extend([20, 0, 0, 8, 0, 0, 0, 0, 0, 0]);
        z.extend(0u32.to_le_bytes());
        z.extend((data.len() as u32).to_le_bytes());
        z.extend((data.len() as u32).to_le_bytes());
        z.extend((name.len() as u16).to_le_bytes());
        z.extend(0u16.to_le_bytes());
        z.extend(name.as_bytes());
        z.extend(*data);
        central.extend(0x0201_4b50u32.to_le_bytes());
        central.extend([20, 0, 20, 0, 0, 8, 0, 0, 0, 0, 0, 0]);
        central.extend(0u32.to_le_bytes());
        central.extend((data.len() as u32).to_le_bytes());
        central.extend((data.len() as u32).to_le_bytes());
        central.extend((name.len() as u16).to_le_bytes());
        central.extend([0u8; 12]);
        central.extend(offset.to_le_bytes());
        central.extend(name.as_bytes());
    }
    let cd = z.len() as u32;
    z.extend(&central);
    z.extend(0x0605_4b50u32.to_le_bytes());
    z.extend([0, 0, 0, 0]);
    z.extend((entries.len() as u16).to_le_bytes());
    z.extend((entries.len() as u16).to_le_bytes());
    z.extend((central.len() as u32).to_le_bytes());
    z.extend(cd.to_le_bytes());
    z.extend(0u16.to_le_bytes());
    z
}

fn model(unit: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?><model unit=\"{unit}\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">{body}</model>"
    )
}

const TRIANGLE: &str = "<resources><object id=\"1\" type=\"model\"><mesh><vertices>\
<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/>\
</vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources>\
<build><item objectid=\"1\"/></build>";

fn read(
    zip: &[u8],
) -> (
    Result<port_io::threemf::ThreeMfMesh, ThreeMfError>,
    Vec<String>,
) {
    let mut progress = Vec::new();
    let r = read_3mf(zip, None, limits(), |t| progress.push(t.to_string()));
    (r, progress)
}

#[test]
fn a_single_triangle_reads_in_millimetres() {
    let zip = stored_zip(&[("3D/3dmodel.model", model("inch", TRIANGLE).as_bytes())]);
    let (mesh, progress) = read(&zip);
    let mesh = mesh.unwrap();
    assert_eq!(
        mesh.positions,
        vec![0.0, 0.0, 0.0, 25.4, 0.0, 0.0, 0.0, 25.4, 0.0]
    );
    assert_eq!(mesh.indices, vec![0, 1, 2]);
    assert!(mesh.colors.is_none() && mesh.texture.is_none());
    assert_eq!(progress, ["Reading 3MF…", "Placing the 3MF's objects…"]);
}

#[test]
fn the_originals_refusals_keep_their_text() {
    let cases: [(Vec<u8>, &str, usize); 6] = [
        (
            b"not a zip".to_vec(),
            "This 3MF could not be unzipped — the file is damaged or not a 3MF.",
            1,
        ),
        (
            stored_zip(&[("readme.txt", b"x")]),
            "This 3MF holds no model.",
            1,
        ),
        (
            stored_zip(&[(
                "3D/3dmodel.model",
                model("millimeter", "<resources/>").as_bytes(),
            )]),
            "This 3MF holds no triangles of a part — point clouds and support-only files are not supported.",
            2,
        ),
        (
            stored_zip(&[("a.model", model("furlong", TRIANGLE).as_bytes())]),
            "3MF unit \"furlong\" is not one of micron, millimeter, centimeter, inch, foot or meter.",
            1,
        ),
        (
            stored_zip(&[(
                "a.model",
                model("millimeter", &TRIANGLE.replace("v3=\"2\"", "v3=\"3\""))
                    .as_bytes()
                    .to_vec()
                    .as_slice(),
            )]),
            "Invalid 3MF triangle: a corner names no vertex.",
            1,
        ),
        (
            stored_zip(&[(
                "a.model",
                model(
                    "millimeter",
                    &TRIANGLE
                        .replace("&", "&amp;")
                        .replace("id=\"1\" type", "id=\"&#x110000;\" type"),
                )
                .as_bytes()
                .to_vec()
                .as_slice(),
            )]),
            "Invalid code point 1114112",
            1,
        ),
    ];
    for (zip, message, progress) in cases {
        let (r, p) = read(&zip);
        assert_eq!(r.unwrap_err().message(), message);
        assert_eq!(p.len(), progress, "{message}");
    }
}

#[test]
fn components_transforms_and_mirroring() {
    // Object 2 is object 1 moved by (10,0,0) and mirrored in x; the mirror flips the winding back.
    let body = format!(
        "{}<build><item objectid=\"2\" transform=\"-1 0 0 0 1 0 0 0 1 10 0 0\"/></build>",
        TRIANGLE
            .replace("<build><item objectid=\"1\"/></build>", "")
            .replace(
                "</resources>",
                "<object id=\"2\"><components><component objectid=\"1\"/></components></object></resources>"
            )
    );
    let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", &body).as_bytes())]);
    let mesh = read(&zip).0.unwrap();
    assert_eq!(
        mesh.positions,
        vec![10.0, 0.0, 0.0, 9.0, 0.0, 0.0, 10.0, 1.0, 0.0]
    );
    assert_eq!(mesh.indices, vec![0, 2, 1]);
}

#[test]
fn a_colour_group_colours_vertices_only_when_it_says_something() {
    let coloured = |colors: &str| {
        let body = format!(
            "<resources><colorgroup id=\"9\">{colors}</colorgroup>{}",
            TRIANGLE
                .trim_start_matches("<resources>")
                .replace(
                    "<object id=\"1\"",
                    "<object id=\"1\" pid=\"9\" pindex=\"0\""
                )
                .replace(
                    "<triangle v1=\"0\" v2=\"1\" v3=\"2\"/>",
                    "<triangle v1=\"0\" v2=\"1\" v3=\"2\" p1=\"0\" p2=\"1\" p3=\"1\"/>"
                )
        );
        let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", &body).as_bytes())]);
        read(&zip).0.unwrap().colors
    };
    assert_eq!(
        coloured("<color color=\"#FF0000\"/><color color=\"#FF0000\"/>"),
        None
    );
    assert_eq!(
        coloured("<color color=\"#FF0000\"/><color color=\"#00ff00ff\"/>"),
        Some(vec![255, 0, 0, 0, 255, 0, 0, 255, 0])
    );
}

#[test]
fn a_missing_texture_is_noted_and_a_given_one_is_applied() {
    let body = "<resources><m:texture2d xmlns:m=\"x\" id=\"7\" path=\"/3D/Textures/Skin.PNG\"/>\
<texture2dgroup id=\"8\" texid=\"7\"><tex2coord u=\"0.25\" v=\"0.5\"/></texture2dgroup>\
<object id=\"1\" pid=\"8\" pindex=\"0\"><mesh><vertices><vertex x=\"0\" y=\"0\" z=\"0\"/>\
<vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices>\
<triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources>\
<build><item objectid=\"1\"/></build>";
    let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", body).as_bytes())]);
    let missing = read(&zip).0.unwrap();
    assert_eq!(
        missing.texture,
        Some(TextureNote::Missing(vec!["Skin.PNG".into()]))
    );
    assert!(missing.colors.is_none());
    let companions = ObjCompanions {
        materials: vec![],
        images: vec![TextureImage {
            name: "skin.png".into(),
            width: 1,
            height: 1,
            rgba: vec![10, 20, 30, 255],
        }],
    };
    let applied = read_3mf(&zip, Some(&companions), limits(), |_| {}).unwrap();
    assert_eq!(
        applied.texture,
        Some(TextureNote::Applied(vec!["skin.png".into()]))
    );
    assert_eq!(
        applied.colors,
        Some(vec![10, 20, 30, 10, 20, 30, 10, 20, 30])
    );
}

#[test]
fn slicer_parts_are_left_out() {
    let body = TRIANGLE.replace(
        "</resources>",
        "<object id=\"2\"><mesh><vertices><vertex x=\"5\" y=\"5\" z=\"5\"/><vertex x=\"6\" y=\"5\" z=\"5\"/>\
<vertex x=\"5\" y=\"6\" z=\"5\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources>",
    ).replace("<item objectid=\"1\"/>", "<item objectid=\"1\"/><item objectid=\"2\"/>");
    let config =
        "<config><object id=\"2\"><part id=\"2\" subtype=\"negative_part\"/></object></config>";
    let zip = stored_zip(&[
        ("3D/3dmodel.model", model("millimeter", &body).as_bytes()),
        ("Metadata/model_settings.config", config.as_bytes()),
    ]);
    assert_eq!(read(&zip).0.unwrap().indices, vec![0, 1, 2]);
}

#[test]
fn prototype_names_behave_as_the_originals_plain_objects_do() {
    // unit="constructor" is found on Object.prototype: the scale is not a number, so NaN.
    let zip = stored_zip(&[("a.model", model("constructor", TRIANGLE).as_bytes())]);
    assert!(read(&zip).0.unwrap().positions.iter().all(|v| v.is_nan()));
    // A relationship target naming `constructor` reaches TextDecoder.decode with a function.
    let rels =
        "<Relationships><Relationship Type=\"x/3dmodel\" Target=\"constructor\"/></Relationships>";
    let zip = stored_zip(&[
        ("_rels/.rels", rels.as_bytes()),
        ("a.model", model("millimeter", TRIANGLE).as_bytes()),
    ]);
    assert_eq!(read(&zip).0.unwrap_err(), ThreeMfError::HostType);
}

#[test]
fn host_limits_are_not_source_errors() {
    let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", TRIANGLE).as_bytes())]);
    let mut small = limits();
    small.max_vertices = 2;
    assert_eq!(
        read_3mf(&zip, None, small, |_| {}).unwrap_err(),
        ThreeMfError::Limit(LimitKind::Vertices)
    );
}

#[test]
fn a_component_fan_out_hits_the_work_budget() {
    // Each object places the next one twice: 2^30 placements. The original runs until it is killed;
    // the port stops at its work budget with a host refusal, not a source message.
    let mut resources = String::new();
    for k in 1..30 {
        resources.push_str(&format!(
            "<object id=\"{k}\"><components><component objectid=\"{}\"/><component objectid=\"{}\"/></components></object>",
            k + 1,
            k + 1
        ));
    }
    resources.push_str(&format!(
        "<object id=\"30\"><mesh>{}</mesh></object>",
        TRIANGLE
            .split("<mesh>")
            .nth(1)
            .unwrap()
            .split("</mesh>")
            .next()
            .unwrap()
    ));
    let body = format!("<resources>{resources}</resources><build><item objectid=\"1\"/></build>");
    let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", &body).as_bytes())]);
    let mut small = limits();
    small.max_work = 1_000_000;
    assert_eq!(
        read_3mf(&zip, None, small, |_| {}).unwrap_err(),
        ThreeMfError::Limit(LimitKind::Work)
    );
}

#[test]
fn a_large_mesh_reads_in_linear_time() {
    let n = 60_000;
    let mut v = String::new();
    for i in 0..n {
        v.push_str(&format!("<vertex x=\"{i}\" y=\"{}\" z=\"0\"/>", i % 7));
    }
    let mut t = String::new();
    for i in 0..n - 2 {
        t.push_str(&format!(
            "<triangle v1=\"{i}\" v2=\"{}\" v3=\"{}\"/>",
            i + 1,
            i + 2
        ));
    }
    let body = format!(
        "<resources><object id=\"1\"><mesh><vertices>{v}</vertices><triangles>{t}</triangles></mesh></object></resources><build><item objectid=\"1\"/></build>"
    );
    let zip = stored_zip(&[("3D/3dmodel.model", model("millimeter", &body).as_bytes())]);
    let started = std::time::Instant::now();
    let mesh = read_3mf(&zip, None, limits(), |_| {}).unwrap();
    assert_eq!(mesh.positions.len(), n * 3);
    assert_eq!(mesh.indices.len(), (n - 2) * 3);
    assert!(
        started.elapsed().as_secs() < 20,
        "took {:?}",
        started.elapsed()
    );
}

#[test]
#[ignore = "live comparison with the original's 3MF reader; needs Node 22 and the pinned git object"]
fn pinned_threemf_comparison() {
    corpus::run_comparison();
}

// Reduced review fixtures: budgets stay in bytes, never gigabytes.
#[test]
fn aggregate_part_budget_precedes_each_stored_copy() {
    let xml = model("millimeter", TRIANGLE);
    let zip = stored_zip(&[
        ("a.model", xml.as_bytes()),
        ("a.config", b"1234"),
        ("b.config", b"56789"),
    ]);
    let mut cap = limits();
    cap.max_part_bytes = xml.len() + 8;
    assert_eq!(
        read_3mf(&zip, None, cap, |_| {}).unwrap_err(),
        ThreeMfError::Limit(LimitKind::PartBytes)
    );
    cap.max_part_bytes += 1;
    assert!(read_3mf(&zip, None, cap, |_| {}).is_ok());
}

fn declared_deflate_zip(duplicate: bool, invalid_second: bool) -> Vec<u8> {
    // Empty fixed-Huffman stream, but each central entry declares eight output bytes.
    // inflate_into retains that allocation even though the returned length is zero.
    let second = if invalid_second {
        &[7u8][..]
    } else {
        &[3u8, 0][..]
    };
    let mut zip = stored_zip(&[
        ("a.config", &[3, 0]),
        (if duplicate { "a.config" } else { "b.config" }, second),
    ]);
    let mut pos = 0;
    while pos + 46 <= zip.len() {
        if zip[pos..].starts_with(&0x0201_4b50u32.to_le_bytes()) {
            zip[pos + 10..pos + 12].copy_from_slice(&8u16.to_le_bytes());
            zip[pos + 24..pos + 28].copy_from_slice(&8u32.to_le_bytes());
            pos += 46;
        } else {
            pos += 1;
        }
    }
    zip
}

#[test]
fn aggregate_part_budget_counts_declared_capacity_before_inflate() {
    for duplicate in [false, true] {
        for invalid_second in [false, true] {
            let zip = declared_deflate_zip(duplicate, invalid_second);
            let mut cap = limits();
            cap.max_part_bytes = 12;
            assert_eq!(
                read_3mf(&zip, None, cap, |_| {}).unwrap_err(),
                ThreeMfError::Limit(LimitKind::PartBytes),
                "duplicate={duplicate}, invalid_second={invalid_second}"
            );
        }
    }
}

#[test]
fn filtered_entries_do_not_spend_the_part_budget() {
    let xml = model("millimeter", TRIANGLE);
    let zip = stored_zip(&[("a.model", xml.as_bytes()), ("ignored.png", &[0; 1024])]);
    let mut cap = limits();
    cap.max_part_bytes = xml.len();
    assert!(read_3mf(&zip, None, cap, |_| {}).is_ok());
}

#[test]
fn lone_surrogate_in_directory_keeps_texture_basename_fallback() {
    let companions = ObjCompanions {
        materials: vec![],
        images: vec![TextureImage {
            name: "skin.png".into(),
            width: 1,
            height: 1,
            rgba: vec![10, 20, 30, 255],
        }],
    };
    for path in [
        "&#xD800;/Skin.PNG",
        "&#xDC00;\\Skin.PNG",
        "&#xD800;/ Skin.PNG ",
    ] {
        let body = TRIANGLE.replace("<resources>", &format!("<resources><texture2d id=\"7\" path=\"{path}\"/><texture2dgroup id=\"8\" texid=\"7\"><tex2coord u=\"0\" v=\"0\"/></texture2dgroup>"))
            .replace("type=\"model\"", "type=\"model\" pid=\"8\" pindex=\"0\"");
        let zip = stored_zip(&[("a.model", model("millimeter", &body).as_bytes())]);
        let mesh = read_3mf(&zip, Some(&companions), limits(), |_| {}).unwrap();
        assert_eq!(
            mesh.texture,
            Some(TextureNote::Applied(vec!["skin.png".into()])),
            "{path}"
        );
        assert_eq!(mesh.colors, Some(vec![10, 20, 30, 10, 20, 30, 10, 20, 30]));
    }
}

#[test]
fn stored_entries_obey_the_per_entry_limit_before_copying() {
    let zip = stored_zip(&[("a.config", b"12345")]);
    let mut cap = limits();
    cap.zip.max_entry_bytes = 4;
    assert_eq!(
        read_3mf(&zip, None, cap, |_| {}).unwrap_err(),
        ThreeMfError::Limit(LimitKind::Archive)
    );
}
