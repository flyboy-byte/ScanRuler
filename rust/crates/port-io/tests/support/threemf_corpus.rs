// SPDX-License-Identifier: AGPL-3.0-only
//! Corpus, mutation and comparison driver for `io.threemf`.
//!
//! Generated packages are structurally valid 3MF with the variety real files have (unit names,
//! namespace prefixes, component trees, production-extension paths, colour and texture groups,
//! slicer metadata, XML decoration). They are then damaged: token- and byte-level edits to the
//! XML, and byte edits to the zip. The original's `parse3MF` is the judge.
use super::{limits, stored_zip};
use port_io::obj_texture::{ObjCompanions, TextureImage, TextureNote};
use port_io::threemf::{ThreeMfError, ThreeMfMesh, read_3mf};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

#[derive(Clone)]
struct Img {
    name: String,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct Package {
    parts: Vec<(String, Vec<u8>)>,
    images: Vec<Img>,
    /// 0: stored by the test; otherwise a deflate level for the original's `zipSync`.
    level: u8,
}

// ---- Generation ---------------------------------------------------------------------------

struct Group {
    id: String,
    kind: GroupKind,
}
enum GroupKind {
    Colors(usize),
    Texture(usize),
}

struct Gen<'a> {
    rng: &'a mut Rng,
    prefix: &'static str,
}

impl Gen<'_> {
    fn tag(&self, name: &str) -> String {
        format!("{}{name}", self.prefix)
    }
    fn sp(&mut self) -> &'static str {
        self.rng.pick(&[" ", " ", " ", "  ", "\n  ", "\t"])
    }
    fn number(&mut self, x: f64) -> String {
        match self.rng.below(14) {
            0 => format!("{x:e}"),
            1 if x >= 0.0 => format!("+{x}"),
            2 => format!(" {x} "),
            3 => format!("{x:.2}"),
            4 => format!("{x}").replacen("0.", ".", 1),
            5 => format!("{}", x as i64),
            _ => format!("{x}"),
        }
    }
    fn coord(&mut self) -> String {
        let x = (self.rng.below(2000) as f64 - 1000.0) / 8.0;
        self.number(x)
    }
    /// An attribute with a random quote style and spacing.
    fn attr(&mut self, name: &str, value: &str) -> String {
        let q = if self.rng.chance(15) { '\'' } else { '"' };
        let value = if q == '\'' {
            value.replace('\'', "&apos;")
        } else {
            value.replace('"', "&quot;")
        };
        let value = if self.rng.chance(4) {
            value.replace('1', "&#49;")
        } else {
            value
        };
        let sp = self.sp();
        let eq = if self.rng.chance(5) { " = " } else { "=" };
        format!("{sp}{name}{eq}{q}{value}{q}")
    }
    fn element(&mut self, name: &str, attrs: &[(&str, String)], inner: Option<&str>) -> String {
        let tag = self.tag(name);
        let mut s = format!("<{tag}");
        let mut order: Vec<usize> = (0..attrs.len()).collect();
        if self.rng.chance(20) {
            order.reverse();
        }
        for i in order {
            let a = self.attr(attrs[i].0, &attrs[i].1);
            s.push_str(&a);
        }
        match inner {
            None if self.rng.chance(90) => s.push_str("/>"),
            None => s.push_str(&format!("></{tag}>")),
            Some(inner) => s.push_str(&format!(">{inner}</{tag}>")),
        }
        s
    }
    fn join(&mut self, parts: Vec<String>) -> String {
        let mut out = String::new();
        for p in parts {
            out.push_str(&p);
            match self.rng.below(12) {
                0 => out.push('\n'),
                1 => out.push_str("<!-- note -->"),
                2 => out.push_str("<?pi data?>"),
                3 => out.push_str("<![CDATA[ <vertex x=\"1\"/> ]]>"),
                4 => out.push_str("  "),
                _ => {}
            }
        }
        out
    }

    fn colour(&mut self) -> String {
        match self.rng.below(10) {
            0 => "#12345678".into(),
            1 => "red".into(),
            2 => "#abcdef".into(),
            _ => format!("#{:06X}", self.rng.below(0x0100_0000)),
        }
    }

    fn mesh_object(&mut self, id: &str, groups: &[Group]) -> String {
        let nv = 3 + self.rng.below(7);
        let mut verts = Vec::new();
        for _ in 0..nv {
            let (x, y, z) = (self.coord(), self.coord(), self.coord());
            verts.push(self.element("vertex", &[("x", x), ("y", y), ("z", z)], None));
        }
        let nt = 1 + self.rng.below(7);
        let mut tris = Vec::new();
        let object_group =
            (!groups.is_empty() && self.rng.chance(40)).then(|| self.rng.below(groups.len()));
        for _ in 0..nt {
            let mut attrs = Vec::new();
            for n in ["v1", "v2", "v3"] {
                let bad = self.rng.below(1000) < 3;
                let v = self.rng.below(nv) + if bad { nv } else { 0 };
                attrs.push((n, v.to_string()));
            }
            if !groups.is_empty() && self.rng.chance(45) {
                let g = &groups[self.rng.below(groups.len())];
                attrs.push(("pid", g.id.clone()));
                let size = match g.kind {
                    GroupKind::Colors(n) | GroupKind::Texture(n) => n,
                };
                let p1 = self.rng.below(size + 1);
                attrs.push(("p1", p1.to_string()));
                if self.rng.chance(60) {
                    attrs.push(("p2", self.rng.below(size + 1).to_string()));
                    attrs.push(("p3", self.rng.below(size + 1).to_string()));
                }
            } else if object_group.is_some() && self.rng.chance(50) {
                attrs.push(("p1", self.rng.below(4).to_string()));
            }
            tris.push(self.element("triangle", &attrs, None));
        }
        let v = self.element("vertices", &[], Some(&verts.concat()));
        let t = self.element("triangles", &[], Some(&tris.concat()));
        let mesh = self.element("mesh", &[], Some(&format!("{v}{t}")));
        let mut attrs = vec![("id", id.to_string())];
        let ty = match self.rng.below(24) {
            0 => Some("support"),
            1 => Some("solidsupport"),
            2 => Some("OTHER"),
            3 => Some("surface"),
            4 => Some("MODEL"),
            5 => Some("model"),
            _ => None,
        };
        if let Some(ty) = ty {
            attrs.push(("type", ty.into()));
        }
        if let Some(g) = object_group {
            attrs.push(("pid", groups[g].id.clone()));
            attrs.push(("pindex", self.rng.below(4).to_string()));
        }
        self.element("object", &attrs, Some(&mesh))
    }

    fn transform(&mut self) -> String {
        match self.rng.below(8) {
            0 => "-1 0 0 0 1 0 0 0 1 10 0 0".into(),
            1 => "1 0 0 0 1 0 0 0 1 5 -3 2".into(),
            2 => "0 1 0 -1 0 0 0 0 1 0 0 0".into(),
            3 => "2 0 0 0 2 0 0 0 -2 0 0 0".into(),
            4 if self.rng.chance(20) => "1 0 0 0 1 0 0 0 1 0 0".into(),
            5 => "1e0 0 0 0 1 0 0 0 1 0 0 0".into(),
            6 => "1  0 0  0 1 0\n0 0 1 0 0 0".into(),
            _ => "1 0 0 0 1 0 0 0 1 0 0 0".into(),
        }
    }
}

fn pixels(rng: &mut Rng, w: u32, h: u32) -> Vec<u8> {
    (0..w * h * 4).map(|_| rng.below(256) as u8).collect()
}

fn package(rng: &mut Rng) -> Package {
    let mut images = Vec::new();
    let prefix = *rng.pick(&["", "", "m:", "ns0:"]);
    let mut g = Gen { rng, prefix };
    let unit = match g.rng.below(14) {
        0 => None,
        1 => Some("micron"),
        2 => Some("CENTIMETER"),
        3 => Some("inch"),
        4 => Some("foot"),
        5 => Some("meter"),
        6 => Some("furlong"),
        7 if g.rng.chance(40) => Some(*g.rng.pick(&["constructor", "__proto__"])),
        _ => Some("millimeter"),
    };

    // Resources: colour and texture groups.
    let mut groups = Vec::new();
    let mut resources = Vec::new();
    if g.rng.chance(40) {
        let n = 1 + g.rng.below(4);
        let mut colours = Vec::new();
        let basematerials = g.rng.chance(50);
        for _ in 0..n {
            let c = g.colour();
            colours.push(if basematerials {
                g.element("base", &[("name", "m".into()), ("displaycolor", c)], None)
            } else {
                g.element("color", &[("color", c)], None)
            });
        }
        let name = if basematerials {
            "basematerials"
        } else {
            "colorgroup"
        };
        resources.push(g.element(name, &[("id", "10".into())], Some(&colours.concat())));
        groups.push(Group {
            id: "10".into(),
            kind: GroupKind::Colors(n),
        });
    }
    if g.rng.chance(50) {
        let path = *g.rng.pick(&[
            "/3D/Textures/skin.png",
            "Textures\\Skin.PNG",
            "tex.png",
            "/a/b/missing.jpg",
            "",
        ]);
        resources.push(g.element(
            "texture2d",
            &[
                ("id", "20".into()),
                ("path", path.into()),
                ("contenttype", "image/png".into()),
            ],
            None,
        ));
        let n = 1 + g.rng.below(4);
        let mut uvs = Vec::new();
        for _ in 0..n {
            let (u, v) = (
                g.rng.below(200) as f64 / 100.0 - 0.5,
                g.rng.below(200) as f64 / 100.0 - 0.5,
            );
            uvs.push(g.element(
                "tex2coord",
                &[("u", u.to_string()), ("v", v.to_string())],
                None,
            ));
        }
        let texid = if g.rng.chance(90) { "20" } else { "99" };
        resources.push(g.element(
            "texture2dgroup",
            &[("id", "21".into()), ("texid", texid.into())],
            Some(&uvs.concat()),
        ));
        groups.push(Group {
            id: "21".into(),
            kind: GroupKind::Texture(n),
        });
        if g.rng.chance(85) {
            let (w, h) = (1 + g.rng.below(4) as u32, 1 + g.rng.below(4) as u32);
            let name = *g.rng.pick(&[
                "skin.png",
                "SKIN.PNG",
                "C:\\scan\\Skin.png",
                "other.png",
                "textures/skin.png",
            ]);
            let rgba = pixels(g.rng, w, h);
            images.push(Img {
                name: name.into(),
                width: w,
                height: h,
                rgba,
            });
        }
    }

    // Objects: meshes, then composites over them (some through a second model file).
    let n_mesh = 1 + g.rng.below(3);
    let mut object_ids: Vec<String> = Vec::new();
    for k in 1..=n_mesh {
        let id = k.to_string();
        resources.push(g.mesh_object(&id, &groups));
        object_ids.push(id);
    }
    let mut parts = Vec::new();
    let external = g.rng.chance(35);
    if external {
        let child_unit = if g.rng.chance(40) { Some("inch") } else { None };
        let child_groups: Vec<Group> = Vec::new();
        let obj = g.mesh_object("1", &child_groups);
        let resources = g.element("resources", &[], Some(&obj));
        let mut attrs = Vec::new();
        if let Some(u) = child_unit {
            attrs.push(("unit", u.to_string()));
        }
        let child = g.element("model", &attrs, Some(&resources));
        parts.push((
            "3D/Objects/o1.model".to_string(),
            format!("<?xml version=\"1.0\"?>{child}").into_bytes(),
        ));
    }
    let composites = g.rng.below(3);
    for next in (n_mesh + 1..).take(composites) {
        let mut comps = Vec::new();
        for _ in 0..1 + g.rng.below(3) {
            let target = g.rng.pick(&object_ids).clone();
            let mut attrs = vec![("objectid", target)];
            if g.rng.chance(60) {
                attrs.push(("transform", g.transform()));
            }
            if external && g.rng.chance(40) {
                attrs[0].1 = "1".into();
                attrs.push((
                    "path",
                    g.rng
                        .pick(&[
                            "/3D/Objects/o1.model",
                            "3D/OBJECTS/O1.MODEL",
                            "/3D/Objects/none.model",
                        ])
                        .to_string(),
                ));
            }
            if g.rng.chance(4) {
                attrs[0].1 = next.to_string(); // a ring
            }
            comps.push(g.element("component", &attrs, None));
        }
        let inner = g.element("components", &[], Some(&comps.concat()));
        let id = next.to_string();
        resources.push(g.element("object", &[("id", id.clone())], Some(&inner)));
        object_ids.push(id);
    }

    let mut items = Vec::new();
    for _ in 0..1 + g.rng.below(3) {
        let mut attrs = vec![("objectid", g.rng.pick(&object_ids).clone())];
        if g.rng.chance(50) {
            attrs.push(("transform", g.transform()));
        }
        if g.rng.chance(3) {
            attrs[0].1 = "77".into();
        }
        items.push(g.element("item", &attrs, None));
    }
    let resources_xml = g.join(resources);
    let resources_el = g.element("resources", &[], Some(&resources_xml));
    let build = g.element("build", &[], Some(&items.concat()));
    let ns = if g.prefix.is_empty() {
        " xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\"".to_string()
    } else {
        format!(
            " xmlns:{}=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\"",
            g.prefix.trim_end_matches(':')
        )
    };
    let tag = g.tag("model");
    let unit_attr = unit.map(|u| format!(" unit=\"{u}\"")).unwrap_or_default();
    let mut xml = format!("<{tag}{unit_attr}{ns}>{resources_el}{build}</{tag}>");
    if g.rng.chance(70) {
        xml = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{xml}");
    }
    let mut bytes = xml.into_bytes();
    if g.rng.chance(8) {
        bytes.splice(0..0, [0xef, 0xbb, 0xbf]);
    }
    let model_name = *g.rng.pick(&[
        "3D/3dmodel.model",
        "3D/3dmodel.model",
        "3D/3dmodel.model",
        "3D/main.model",
        "Model.MODEL",
    ]);
    parts.push((model_name.to_string(), bytes));

    // _rels/.rels
    let target = *g.rng.pick(&[
        "/3D/3dmodel.model",
        "/3D/main.model",
        "3D/3DMODEL.MODEL",
        "/missing.model",
        "",
    ]);
    let target = if g.rng.chance(6) {
        *g.rng.pick(&["constructor", "__proto__"])
    } else {
        target
    };
    let ty = *g.rng.pick(&[
        "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel",
        "http://schemas.microsoft.com/3dmanufacturing/2013/01/3DMODEL",
        "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
    ]);
    match g.rng.below(10) {
        0 => {}
        1 => parts.push((
            "_rels/.rels".into(),
            b"<Relationships><Relationship Type=\"x/3dmodel\"/></Relationships>".to_vec(),
        )),
        _ => {
            let rel = format!(
                "<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Target=\"{target}\" Id=\"rel0\" Type=\"{ty}\"/></Relationships>"
            );
            parts.push(("_rels/.rels".into(), rel.into_bytes()));
        }
    }
    // Slicer metadata.
    if g.rng.chance(25) {
        let ids = object_ids.clone();
        let mut config = String::from("<config>");
        for id in &ids {
            if g.rng.chance(50) {
                let sub = g.rng.pick(&[
                    "normal_part",
                    "modifier_part",
                    "negative_part",
                    "support_blocker",
                ]);
                config.push_str(&format!("<object id=\"{id}\"><metadata key=\"name\" value=\"x\"/><part id=\"{id}\" subtype=\"{sub}\"/></object>"));
            }
        }
        config.push_str("</config>");
        parts.push(("Metadata/model_settings.config".into(), config.into_bytes()));
    }
    if g.rng.chance(20) {
        let mut config = String::from("<config>");
        for id in &object_ids {
            config.push_str(&format!("<object id=\"{id}\" instances_count=\"1\">"));
            for _ in 0..1 + g.rng.below(2) {
                let first = g.rng.below(4);
                let last = first + g.rng.below(3);
                let (key, value) = *g.rng.pick(&[
                    ("modifier", "1"),
                    ("modifier", "0"),
                    ("volume_type", "ModelPart"),
                    ("volume_type", "NegativeVolume"),
                    ("name", "x"),
                ]);
                config.push_str(&format!("<volume firstid=\"{first}\" lastid=\"{last}\"><metadata type=\"volume\" key=\"{key}\" value=\"{value}\"/></volume>"));
            }
            config.push_str("</object>");
        }
        config.push_str("</config>");
        parts.push((
            "Metadata/Slic3r_PE_model.config".into(),
            config.into_bytes(),
        ));
    }
    if g.rng.chance(30) {
        parts.push(("Metadata/thumbnail.png".into(), vec![1, 2, 3]));
    }
    if g.rng.chance(6) {
        let i = g.rng.below(parts.len());
        let (name, mut data) = parts[i].clone();
        let name = if g.rng.chance(50) {
            name.to_uppercase()
        } else {
            name
        };
        if g.rng.chance(50) {
            data = mutate_xml(g.rng, data);
        }
        let at = g.rng.below(parts.len() + 1);
        parts.insert(at, (name, data));
    }
    let level = if g.rng.chance(50) {
        0
    } else {
        *g.rng.pick(&[1, 6, 9])
    };
    Package {
        parts,
        images,
        level,
    }
}

// ---- Named cases --------------------------------------------------------------------------
// Hazards random generation rarely or never produces. They run unmutated on every comparison.

const V3: &str = "<vertices><vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles>";

fn tri(id: &str) -> String {
    format!("<object id=\"{id}\"><mesh>{V3}</mesh></object>")
}

fn doc(attrs: &str, body: &str) -> Vec<u8> {
    format!("<model{attrs}>{body}</model>").into_bytes()
}

fn named() -> Vec<Package> {
    let mut out = Vec::new();
    let mut add = |parts: Vec<(&str, Vec<u8>)>| {
        out.push(Package {
            parts: parts.into_iter().map(|(n, d)| (n.to_string(), d)).collect(),
            images: Vec::new(),
            level: 0,
        });
    };
    let main = "3D/3dmodel.model";
    let one = |body: String| vec![(main, doc("", &body))];
    // Lone and paired surrogates through numeric entities.
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#xD800;\"/></build>",
        tri("&#xD800;")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#xFFFD;\"/></build>",
        tri("&#xD800;")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#xD83D;&#xDE00;\"/></build>",
        tri("\u{1F600}")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#xDE00;&#xD83D;\"/></build>",
        tri("\u{1F600}")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#x10FFFF;\"/></build>",
        tri("&#X10ffff;")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#0;\"/></build>",
        tri("&#00000;")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#4294967296;\"/></build>",
        tri("1")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&LT;&Amp;&QUOT;\"/></build>",
        tri("&lt;&amp;&quot;")
    )));
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"&#x;&#;&amp\"/></build>",
        tri("1")
    )));
    // Number grammar in coordinates and indices.
    for x in [
        "0x10",
        " 1e1 ",
        "Infinity",
        "-Infinity",
        "1_0",
        "\u{661}\u{662}",
        ".",
        "+.5e-1",
        "",
        "0b11",
        "0o17",
        "1e",
        "\u{a0}7\u{feff}",
    ] {
        add(one(format!(
            "<resources><object id=\"1\"><mesh><vertices><vertex x=\"{x}\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources><build><item objectid=\"1\"/></build>"
        )));
    }
    for v in [
        "1e0", "+1", " 1", "0x1", "1.0", "1.5", "-0", "", "NaN", "1e1000",
    ] {
        add(one(format!(
            "<resources><object id=\"1\"><mesh>{}</mesh></object></resources><build><item objectid=\"1\"/></build>",
            V3.replace("v2=\"1\"", &format!("v2=\"{v}\""))
        )));
    }
    // Relationships: the first matching one without a Target, then one with.
    let rels = b"<Relationships><Relationship Type=\"a/3DMODEL\"/><Relationship Type=\"a/3dmodel\" Target=\"/3D/other.model\"/></Relationships>".to_vec();
    add(vec![
        ("_rels/.rels", rels),
        (main, doc("", "<build/>")),
        (
            "3D/other.model",
            doc(
                "",
                &format!(
                    "<resources>{}</resources><build><item objectid=\"1\"/></build>",
                    tri("1")
                ),
            ),
        ),
    ]);
    add(vec![("_rels/.rels", b"<Relationships><Relationship Type=\"a/3dmodel\" Target=\"3d/OTHER.model\"/></Relationships>".to_vec()), ("3D/other.model", doc("", &format!("<resources>{}</resources><build><item objectid=\"1\"/></build>", tri("1"))))]);
    add(vec![
        ("x.rels", b"<Relationships/>".to_vec()),
        (
            "some/dir/part.model",
            doc(
                "",
                &format!(
                    "<resources>{}</resources><build><item objectid=\"1\"/></build>",
                    tri("1")
                ),
            ),
        ),
    ]);
    // Part names under Unicode case folding.
    for (part, path) in [
        (
            "3D/Objects/\u{391}\u{3a3}.model",
            "/3d/objects/\u{3b1}\u{3c2}.model",
        ),
        (
            "3D/Objects/\u{391}\u{3a3}.model",
            "/3d/objects/\u{3b1}\u{3c3}.model",
        ),
        ("3D/Objects/\u{130}.model", "3d/objects/i\u{307}.model"),
        ("3D/Objects/\u{212a}.model", "3d/objects/k.model"),
        ("3D/Objects/\u{df}.model", "3d/objects/SS.model"),
    ] {
        add(vec![
            (
                main,
                doc(
                    "",
                    &format!(
                        "<resources><object id=\"2\"><components><component objectid=\"1\" path=\"{path}\"/></components></object></resources><build><item objectid=\"2\"/></build>"
                    ),
                ),
            ),
            (
                part,
                doc("", &format!("<resources>{}</resources>", tri("1"))),
            ),
        ]);
    }
    // Scanner behaviour on odd markup.
    let wrap = |body: &str| {
        one(format!(
            "<resources>{body}</resources><build><item objectid=\"1\"/></build>"
        ))
    };
    add(vec![(main, b"<!DOCTYPE model [<!ENTITY a \"b\">]><model><resources><object id=\"1\"><mesh><vertices><vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources><build><item objectid=\"1\"/></build></model>".to_vec())]);
    add(wrap(
        "<object id=1><mesh><vertices><vertex x=1 y=2 z=3/></vertices></mesh></object>",
    ));
    add(wrap(&format!(
        "<object id=\"1\"><mesh>{V3}</mesh></object><object id=\"1\"><components/></object>"
    )));
    add(wrap(
        "<object id=\"1\"><object id=\"3\"><mesh><vertices></object><vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object>",
    ));
    add(wrap(&format!(
        "<object id=\"1\"><mesh>{V3}</mesh><mesh><vertices><vertex x=\"9\" y=\"9\" z=\"9\"/></vertices></mesh></object>"
    )));
    add(wrap(
        "<object id=\"1\"><mesh><vertices><vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\" p1=\"0\" pid=\"5\"/></triangles></mesh></object>",
    ));
    add(wrap(&format!(
        "<!-- <object id=\"9\"> -->{}<![CDATA[<object id=\"8\">]]><?php echo ?>",
        tri("1")
    )));
    add(wrap(&format!(
        "<m:object xmlns:m=\"u\" m:id=\"1\" id=\"7\"><m:mesh>{V3}</m:mesh></m:object>"
    )));
    add(wrap(
        "<object id=\"1\"><mesh><vertices><vertex x='0' y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z='0'/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3='2'/></triangles></mesh></object>",
    ));
    add(wrap(&format!(
        "<object id=\"1\"><mesh>{}</mesh></object>",
        V3.replace(
            "<vertex x=\"0\" y=\"0\" z=\"0\"/>",
            "<vertex x = \"0\"   y\n=\n\"0\" z=\"0\" ></vertex>"
        )
    )));
    // Transforms and units.
    for t in [
        "",
        "   ",
        "1 0 0 0 1 0 0 0 1 0 0 0",
        "1 0 0 0 1 0 0 0 1 0 0 NaN",
        "1 0 0 0 1 0 0 0 1 0 0 0 0",
        "0x1 0 0 0 1 0 0 0 1 0 0 0",
        "1\u{a0}0 0 0 1 0 0 0 1 0 0 \u{2003}0",
        "1e308 0 0 0 1e308 0 0 0 1e308 1e308 0 0",
        "0 0 0 0 0 0 0 0 0 0 0 0",
        "-1 0 0 0 -1 0 0 0 -1 0 0 0",
    ] {
        add(wrap(&tri("1"))
            .into_iter()
            .map(|(n, d)| {
                (
                    n,
                    String::from_utf8(d)
                        .unwrap_or_default()
                        .replace(
                            "<item objectid=\"1\"/>",
                            &format!("<item objectid=\"1\" transform=\"{t}\"/>"),
                        )
                        .into_bytes(),
                )
            })
            .collect());
    }
    for unit in [
        "inch",
        "INCH",
        "Inch ",
        "mm",
        "",
        "micron",
        "constructor",
        "__proto__",
        "hasOwnProperty",
        "toString",
        "\u{212a}",
    ] {
        add(vec![(
            main,
            doc(
                &format!(" unit=\"{unit}\""),
                &format!(
                    "<resources>{}</resources><build><item objectid=\"1\"/></build>",
                    tri("1")
                ),
            ),
        )]);
    }
    // A component file inherits the root's unit unless it names its own.
    for child_unit in ["", " unit=\"meter\""] {
        add(vec![
            (
                main,
                doc(
                    " unit=\"inch\"",
                    "<resources><object id=\"2\"><components><component objectid=\"1\" path=\"/3D/Objects/c.model\"/></components></object></resources><build><item objectid=\"2\"/></build>",
                ),
            ),
            (
                "3D/Objects/c.model",
                doc(child_unit, &format!("<resources>{}</resources>", tri("1"))),
            ),
        ]);
    }
    // The same object placed twice, a path naming the root itself, a ring.
    add(one(format!(
        "<resources>{}</resources><build><item objectid=\"1\"/><item objectid=\"1\" transform=\"1 0 0 0 1 0 0 0 1 5 0 0\"/></build>",
        tri("1")
    )));
    add(one(format!(
        "<resources>{}<object id=\"2\"><components><component objectid=\"1\" path=\"/3D/3dmodel.model\"/></components></object></resources><build><item objectid=\"2\" path=\"3D/3DMODEL.model\"/></build>",
        tri("1")
    )));
    add(one("<resources><object id=\"1\"><components><component objectid=\"1\"/></components></object></resources><build><item objectid=\"1\"/></build>".into()));
    // Bytes: BOM, invalid UTF-8, leading whitespace and text.
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend(doc(
        "",
        &format!(
            "<resources>{}</resources><build><item objectid=\"1\"/></build>",
            tri("1")
        ),
    ));
    add(vec![(main, bom)]);
    let mut bad = doc(
        "",
        &format!(
            "<resources>{}</resources><build><item objectid=\"1\"/></build>",
            tri("1")
        ),
    );
    if let Some(at) = bad.windows(4).position(|w| w == b"\"1\"/") {
        bad.splice(at + 1..at + 2, [0xff, 0xc0, 0x80]);
    }
    add(vec![(main, bad)]);
    add(vec![(main, b"  \n garbage <model><resources><object id=\"1\"><mesh><vertices><vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\"/></triangles></mesh></object></resources><build><item objectid=\"1\"/></build>".to_vec())]);
    // Slicer metadata edge cases.
    add(vec![(main, doc("", &format!("<resources>{}{}</resources><build><item objectid=\"1\"/><item objectid=\"2\"/></build>", tri("1"), tri("2")))), ("Metadata/model_settings.config", b"<config><object id=\"1\"><part id=\"2\" subtype=\"\"/><part id=\"1\" subtype=\"modifier\"/></object></config>".to_vec())]);
    for (first, last, key, value) in [
        ("0", "0", "modifier", "1"),
        ("0", "-1", "modifier", "1"),
        ("1", "0", "volume_type", "X"),
        ("0.5", "2", "modifier", "1"),
        ("0", "9", "volume_type", "ModelPart"),
        ("0", "0", "volume_type", ""),
    ] {
        let config = format!(
            "<config><object id=\"1\"><volume firstid=\"{first}\" lastid=\"{last}\"><metadata type=\"volume\" key=\"{key}\" value=\"{value}\"/></volume></object></config>"
        );
        add(vec![
            (
                main,
                doc(
                    "",
                    &format!(
                        "<resources>{}</resources><build><item objectid=\"1\"/></build>",
                        tri("1")
                    ),
                ),
            ),
            ("Metadata/Slic3r_PE_model.config", config.into_bytes()),
        ]);
    }
    // Colours: alpha, case, trimmed text, fallbacks, and a colour group that is also a texture's.
    for colors in [
        "<color color=\" #AbCdEf \"/><color color=\"#00000000\"/>",
        "<color color=\"#12345\"/><color color=\"#1234567\"/>",
        "<color/><color color=\"\"/>",
        "<color color=\"#aaaaaa\"/><color color=\"#AAAAAA\"/>",
    ] {
        add(one(format!(
            "<resources><colorgroup id=\"5\">{colors}</colorgroup><object id=\"1\" pid=\"5\" pindex=\"0\"><mesh>{}</mesh></object></resources><build><item objectid=\"1\"/></build>",
            V3.replace("v3=\"2\"", "v3=\"2\" p1=\"0\" p2=\"1\" p3=\"2\"")
        )));
    }
    out
}

// ---- Mutation -----------------------------------------------------------------------------

const VALUES: &[&str] = &[
    "",
    "NaN",
    "Infinity",
    "-Infinity",
    "-1",
    "0",
    "-0",
    "1e999",
    "1e-999",
    "0x10",
    "0b11",
    "0o7",
    "99999999999999999999",
    "1.5",
    " 3 ",
    "abc",
    "1 2 3",
    "&#x110000;",
    "&#xD800;",
    "&#128512;",
    "&amp;",
    "&lt;",
    "&bogus;",
    "&#99999999999999999999;",
    "constructor",
    "__proto__",
    "3D/3dmodel.model",
    "/3D/Objects/o1.model",
    "2",
    "1",
    "0 0 0 0 0 0 0 0 0 0 0 0",
    "1 0 0 0 1 0 0 0 1 0 0 0",
    "1 0 0 0 1 0 0 0 1 0 0",
    "#FFF",
    "#ffffff",
    "#GGGGGG",
    "millimeter",
    "METER",
    "\u{17f}",
    "\u{ffff}",
    "1e308",
    "-1e308",
    "1.7976931348623157e308",
    "5e-324",
    "3.5e38",
    "1e39",
    "1e2",
    ".5",
    "5.",
    "+5",
    "4294967296",
    "-2147483648",
];
const TAGS: &[&str] = &[
    "model",
    "object",
    "mesh",
    "vertex",
    "triangle",
    "component",
    "item",
    "basematerials",
    "colorgroup",
    "base",
    "color",
    "texture2d",
    "texture2dgroup",
    "tex2coord",
    "build",
    "resources",
];
const PUNCT: &[u8] = b"<>\"'&/;=! ?-[]:#\n";

fn find_all(hay: &[u8], needle: &[u8]) -> Vec<usize> {
    hay.windows(needle.len())
        .enumerate()
        .filter(|(_, w)| *w == needle)
        .map(|(i, _)| i)
        .collect()
}

fn mutate_xml(rng: &mut Rng, mut b: Vec<u8>) -> Vec<u8> {
    for _ in 0..1 + rng.below(2) {
        if b.is_empty() {
            b.push(*rng.pick(PUNCT));
            continue;
        }
        let at = rng.below(b.len());
        match rng.below(11) {
            0 => b.truncate(at),
            1 => {
                let end = (at + 1 + rng.below(20)).min(b.len());
                b.drain(at..end);
            }
            2 => {
                let end = (at + 1 + rng.below(30)).min(b.len());
                let chunk = b[at..end].to_vec();
                b.splice(at..at, chunk);
            }
            3 => b.insert(at, *rng.pick(PUNCT)),
            4 => b[at] ^= 1 << rng.below(8),
            5 | 6 => {
                // Replace one attribute value with an edge value.
                let quote = if rng.chance(50) { b'"' } else { b'\'' };
                let starts = find_all(&b, &[b'=', quote]);
                if let Some(&s) = starts.get(rng.below(starts.len().max(1))) {
                    let from = s + 2;
                    if let Some(len) = b[from..].iter().position(|&c| c == quote) {
                        let value = rng.pick(VALUES).as_bytes().to_vec();
                        b.splice(from..from + len, value);
                    }
                }
            }
            7 => {
                let from = *rng.pick(TAGS);
                let to = *rng.pick(TAGS);
                let hits = find_all(&b, from.as_bytes());
                if let Some(&h) = hits.get(rng.below(hits.len().max(1))) {
                    b.splice(h..h + from.len(), to.bytes());
                }
            }
            8 => {
                let slashes = find_all(&b, b"/");
                if let Some(&s) = slashes.get(rng.below(slashes.len().max(1))) {
                    b.remove(s);
                }
            }
            9 => {
                let digits: Vec<usize> = (0..b.len()).filter(|&i| b[i].is_ascii_digit()).collect();
                if let Some(&d) = digits.get(rng.below(digits.len().max(1))) {
                    let value = rng.pick(VALUES).as_bytes().to_vec();
                    b.splice(d..d + 1, value);
                }
            }
            _ => {
                let value = rng.pick(VALUES).as_bytes().to_vec();
                b.splice(at..at, value);
            }
        }
    }
    b
}

const EDGES: [u32; 6] = [0, 1, 0xff, 0xffff, 0xffff_ffff, 0x8000_0000];

fn mutate_zip(rng: &mut Rng, mut b: Vec<u8>) -> Vec<u8> {
    for _ in 0..1 + rng.below(2) {
        if b.is_empty() {
            return b;
        }
        let at = if rng.chance(50) {
            b.len() - 1 - rng.below(b.len().min(120))
        } else {
            rng.below(b.len())
        };
        match rng.below(5) {
            0 => b.truncate(at),
            1 => b[at] = rng.below(256) as u8,
            2 => b[at] ^= 1 << rng.below(8),
            3 => {
                b.remove(at);
            }
            _ => {
                for (k, byte) in EDGES[rng.below(EDGES.len())]
                    .to_le_bytes()
                    .iter()
                    .enumerate()
                {
                    if at + k < b.len() {
                        b[at + k] = *byte;
                    }
                }
            }
        }
    }
    b
}

// ---- The original -------------------------------------------------------------------------

fn node(root: &Path, mode: &str, stdin: Vec<u8>) -> Vec<u8> {
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/threemf.cjs"))
        .arg(mode)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start node");
    let mut sin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        let _ = sin.write_all(&stdin);
    });
    let mut out = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut out).unwrap();
    writer.join().unwrap();
    assert!(
        child.wait().unwrap().success(),
        "the original's process failed"
    );
    out
}

fn put(out: &mut Vec<u8>, n: usize) {
    out.extend((n as u32).to_le_bytes());
}
fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put(out, b.len());
    out.extend(b);
}

#[derive(Debug)]
enum Original {
    Ok {
        progress: Vec<String>,
        positions: Vec<u32>,
        indices: Vec<u32>,
        colors: Option<Vec<u8>>,
        texture: Option<(u8, Vec<String>)>,
    },
    Err {
        progress: Vec<String>,
        type_error: bool,
        message: String,
    },
    TimedOut,
}

struct Reader<'a>(&'a [u8], usize);
impl Reader<'_> {
    fn u8(&mut self) -> u8 {
        self.1 += 1;
        self.0[self.1 - 1]
    }
    fn u32(&mut self) -> usize {
        let v = u32::from_le_bytes(self.0[self.1..self.1 + 4].try_into().unwrap());
        self.1 += 4;
        v as usize
    }
    fn bytes(&mut self) -> Vec<u8> {
        let n = self.u32();
        self.1 += n;
        self.0[self.1 - n..self.1].to_vec()
    }
    fn string(&mut self) -> String {
        String::from_utf8_lossy(&self.bytes()).into_owned()
    }
    fn words(&mut self) -> Vec<u32> {
        let n = self.u32();
        let v = (0..n)
            .map(|i| {
                u32::from_le_bytes(
                    self.0[self.1 + i * 4..self.1 + i * 4 + 4]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect();
        self.1 += n * 4;
        v
    }
}

fn parse_original(data: &[u8], count: usize) -> Vec<Original> {
    let mut r = Reader(data, 0);
    (0..count)
        .map(|_| {
            let status = r.u8();
            let n = r.u32();
            let progress: Vec<String> = (0..n).map(|_| r.string()).collect();
            match status {
                2 => Original::TimedOut,
                1 => {
                    let positions = r.words();
                    let indices = r.words();
                    let colors = (r.u8() == 1).then(|| r.bytes());
                    let texture = match r.u8() {
                        0 => None,
                        kind => {
                            let n = r.u32();
                            Some((kind, (0..n).map(|_| r.string()).collect()))
                        }
                    };
                    Original::Ok {
                        progress,
                        positions,
                        indices,
                        colors,
                        texture,
                    }
                }
                _ => {
                    let type_error = r.u8() == 1;
                    Original::Err {
                        progress,
                        type_error,
                        message: r.string(),
                    }
                }
            }
        })
        .collect()
}

// ---- Comparison ---------------------------------------------------------------------------

fn canonical(bits: u32) -> u32 {
    if f32::from_bits(bits).is_nan() {
        0x7fc0_0000
    } else {
        bits
    }
}

#[derive(Default)]
struct Tally {
    same_ok: usize,
    same_err: usize,
    host_rust: usize,
    host_kinds: std::collections::BTreeMap<String, usize>,
    host_original: usize,
    diverged: usize,
    with_colors: usize,
    applied: usize,
    missing: usize,
    refusals: std::collections::BTreeMap<String, usize>,
}

fn compare(
    rust: &Result<ThreeMfMesh, ThreeMfError>,
    progress: &[String],
    original: &Original,
    sabotage: bool,
) -> Result<bool, String> {
    match (rust, original) {
        (_, Original::TimedOut) => Ok(false),
        (Err(ThreeMfError::Limit(_) | ThreeMfError::Allocation | ThreeMfError::Texture(_)), _) => {
            Ok(false)
        }
        (
            Ok(m),
            Original::Ok {
                progress: op,
                positions,
                indices,
                colors,
                texture,
            },
        ) => {
            if progress != op {
                return Err(format!(
                    "progress differs: rust {progress:?} original {op:?}"
                ));
            }
            let mut mine: Vec<u32> = m.positions.iter().map(|v| canonical(v.to_bits())).collect();
            if sabotage && let Some(first) = mine.first_mut() {
                *first ^= 1;
            }
            let theirs: Vec<u32> = positions.iter().map(|&b| canonical(b)).collect();
            if mine != theirs {
                return Err(format!(
                    "positions differ: rust {} floats, original {}",
                    mine.len(),
                    theirs.len()
                ));
            }
            if &m.indices != indices {
                return Err("indices differ".into());
            }
            if &m.colors != colors {
                return Err(format!(
                    "colours differ: rust {:?} original {:?}",
                    m.colors, colors
                ));
            }
            let note = m.texture.as_ref().map(|t| match t {
                TextureNote::Applied(n) => (1u8, n.clone()),
                TextureNote::Missing(n) => (2u8, n.clone()),
            });
            if &note != texture {
                return Err(format!(
                    "texture note differs: rust {note:?} original {texture:?}"
                ));
            }
            Ok(true)
        }
        (
            Err(e),
            Original::Err {
                progress: op,
                type_error,
                message,
            },
        ) => {
            if progress != op {
                return Err(format!(
                    "progress differs on refusal: rust {progress:?} original {op:?}"
                ));
            }
            match e {
                ThreeMfError::HostType if *type_error => Ok(true),
                ThreeMfError::Source(_) if !*type_error && &e.message() == message => Ok(true),
                _ => Err(format!(
                    "refusal differs: rust {:?} original {message:?} (type error {type_error})",
                    e.message()
                )),
            }
        }
        (Ok(_), Original::Err { message, .. }) => {
            Err(format!("rust read it; the original refused: {message:?}"))
        }
        (Err(e), Original::Ok { .. }) => Err(format!(
            "rust refused ({}); the original read it",
            e.message()
        )),
    }
}

pub fn run_comparison() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let sabotage = std::env::var_os("SCANRULER_3MF_SABOTAGE").is_some();
    let seeds: Vec<u64> = std::env::var("SCANRULER_3MF_SEEDS")
        .unwrap_or_else(|_| "1,7,99".into())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let cases: usize = std::env::var("SCANRULER_3MF_CASES")
        .unwrap_or_else(|_| "1000".into())
        .parse()
        .unwrap();
    let mut tally = Tally::default();
    for seed in seeds {
        let mut rng = Rng(seed ^ 0x3df0_c0de);
        let mut packages: Vec<Package> = (0..cases).map(|_| package(&mut rng)).collect();
        let named_count = named().len();
        // Half the packages keep their XML; the rest get it damaged.
        for p in &mut packages {
            if rng.chance(45) {
                let i = rng.below(p.parts.len());
                let bytes = std::mem::take(&mut p.parts[i].1);
                p.parts[i].1 = mutate_xml(&mut rng, bytes);
            }
        }
        // The named cases go first, unmutated.
        packages.splice(0..0, named());
        // Zips: stored by the test, or deflated by the original's own zipSync.
        let mut pack = vec![0u8; 0];
        let deflated: Vec<usize> = (0..packages.len())
            .filter(|&i| packages[i].level > 0)
            .collect();
        put(&mut pack, deflated.len());
        for &i in &deflated {
            pack.push(packages[i].level);
            put(&mut pack, packages[i].parts.len());
            for (name, data) in &packages[i].parts {
                put_bytes(&mut pack, name.as_bytes());
                put_bytes(&mut pack, data);
            }
        }
        let packed = node(&root, "pack", pack);
        let mut reader = Reader(&packed, 0);
        let n = reader.u32();
        let mut deflated_zips: Vec<Vec<u8>> = (0..n).map(|_| reader.bytes()).collect();
        deflated_zips.reverse();
        let mut inputs: Vec<Vec<u8>> = Vec::new();
        for (k, p) in packages.iter().enumerate() {
            let zip = if p.level > 0 {
                deflated_zips.pop().unwrap()
            } else {
                let entries: Vec<(&str, &[u8])> = p
                    .parts
                    .iter()
                    .map(|(n, d)| (n.as_str(), d.as_slice()))
                    .collect();
                stored_zip(&entries)
            };
            inputs.push(if k >= named_count && rng.chance(12) {
                mutate_zip(&mut rng, zip)
            } else {
                zip
            });
        }
        let mut request = Vec::new();
        put(&mut request, inputs.len());
        for (zip, p) in inputs.iter().zip(&packages) {
            put_bytes(&mut request, zip);
            put(&mut request, p.images.len());
            for img in &p.images {
                put_bytes(&mut request, img.name.as_bytes());
                put(&mut request, img.width as usize);
                put(&mut request, img.height as usize);
                put_bytes(&mut request, &img.rgba);
            }
        }
        let originals = parse_original(&node(&root, "run", request), inputs.len());
        for (k, ((zip, p), original)) in inputs.iter().zip(&packages).zip(&originals).enumerate() {
            let companions = ObjCompanions {
                materials: Vec::new(),
                images: p
                    .images
                    .iter()
                    .map(|i| TextureImage {
                        name: i.name.clone(),
                        width: i.width,
                        height: i.height,
                        rgba: i.rgba.clone(),
                    })
                    .collect(),
            };
            let mut progress = Vec::new();
            let rust = read_3mf(zip, Some(&companions), limits(), |t| {
                progress.push(t.to_string())
            });
            if k < named_count && std::env::var_os("SCANRULER_3MF_VERBOSE").is_some() {
                let shown = match &rust {
                    Ok(m) => format!(
                        "read {} vertices, {} triangles",
                        m.positions.len() / 3,
                        m.indices.len() / 3
                    ),
                    Err(e) => format!("refused: {}", e.message()),
                };
                println!("named {k:3}: {shown}");
            }
            match compare(&rust, &progress, original, sabotage) {
                Ok(true) => match &rust {
                    Ok(m) => {
                        tally.same_ok += 1;
                        tally.with_colors += usize::from(m.colors.is_some());
                        match m.texture {
                            Some(TextureNote::Applied(_)) => tally.applied += 1,
                            Some(TextureNote::Missing(_)) => tally.missing += 1,
                            None => {}
                        }
                    }
                    Err(e) => {
                        tally.same_err += 1;
                        let key: String = e.message().chars().take(48).collect();
                        *tally.refusals.entry(key).or_default() += 1;
                    }
                },
                Ok(false) => {
                    if matches!(original, Original::TimedOut) {
                        tally.host_original += 1;
                    } else {
                        tally.host_rust += 1;
                        let kind = match (&rust, original) {
                            (Err(e), Original::Ok { .. }) => {
                                format!("rust {e:?} / original read it")
                            }
                            (Err(e), Original::Err { message, .. }) => {
                                format!(
                                    "rust {e:?} / original refused: {}",
                                    message.chars().take(40).collect::<String>()
                                )
                            }
                            _ => "other".into(),
                        };
                        *tally.host_kinds.entry(kind).or_default() += 1;
                    }
                }
                Err(why) => {
                    tally.diverged += 1;
                    let dir = std::env::temp_dir().join("scanruler-3mf-fuzz");
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(dir.join(format!("seed{seed}-case{k}.3mf")), zip).unwrap();
                    std::fs::write(
                        dir.join(format!("seed{seed}-case{k}.txt")),
                        format!(
                            "{why}\nimages: {:?}\n",
                            p.images
                                .iter()
                                .map(|i| (&i.name, i.width, i.height))
                                .collect::<Vec<_>>()
                        ),
                    )
                    .unwrap();
                    if tally.diverged <= 5 {
                        eprintln!("DIVERGENCE seed {seed} case {k}: {why}");
                    }
                }
            }
        }
    }
    println!("refusals compared: {:#?}", tally.refusals);
    println!(
        "host-limit cases by what the original did: {:#?}",
        tally.host_kinds
    );
    println!(
        "3mf identical_success={} (with colours {}, textures applied {}, missing {}) identical_refusal={} host_limit={} original_timeout={} divergences={}",
        tally.same_ok,
        tally.with_colors,
        tally.applied,
        tally.missing,
        tally.same_err,
        tally.host_rust,
        tally.host_original,
        tally.diverged
    );
    run_real_files();
    if sabotage {
        assert!(tally.diverged > 0, "sabotaged Rust output was not reported");
    } else {
        assert_eq!(
            tally.diverged, 0,
            "divergences; reproducers in the temp directory"
        );
        assert!(
            tally.same_ok > 0,
            "no package read successfully; the corpus is broken"
        );
    }
}

fn collect_3mf(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if let Ok(rd) = std::fs::read_dir(path) {
            let mut entries: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
            entries.sort();
            for e in entries {
                collect_3mf(&e, out);
            }
        }
    } else if path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("3mf"))
    {
        out.push(path.to_path_buf());
    }
}

/// Real packages named by `SCANRULER_3MF_REAL` (a `:`-separated list of files or folders), read
/// only: nothing is copied or stored, and a divergence records the path, not the bytes.
pub fn run_real_files() {
    let Some(list) = std::env::var_os("SCANRULER_3MF_REAL") else {
        return;
    };
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files = Vec::new();
    for p in std::env::split_paths(&list) {
        collect_3mf(&p, &mut files);
    }
    let datas: Vec<Vec<u8>> = files.iter().map(|f| std::fs::read(f).unwrap()).collect();
    let mut request = Vec::new();
    put(&mut request, datas.len());
    for d in &datas {
        put_bytes(&mut request, d);
        put(&mut request, 0);
    }
    let originals = parse_original(&node(&root, "run", request), datas.len());
    let (mut same, mut bad) = (0, 0);
    for ((file, data), original) in files.iter().zip(&datas).zip(&originals) {
        let mut progress = Vec::new();
        let rust = read_3mf(data, None, limits(), |t| progress.push(t.to_string()));
        let shown = match &rust {
            Ok(m) => format!(
                "{} vertices, {} triangles{}",
                m.positions.len() / 3,
                m.indices.len() / 3,
                if m.colors.is_some() { ", coloured" } else { "" }
            ),
            Err(e) => format!("refused: {}", e.message()),
        };
        match compare(&rust, &progress, original, false) {
            Ok(true) => {
                same += 1;
                println!(
                    "real {}: identical ({shown})",
                    file.file_name().unwrap().to_string_lossy()
                );
            }
            Ok(false) => println!(
                "real {}: host class ({shown})",
                file.file_name().unwrap().to_string_lossy()
            ),
            Err(why) => {
                bad += 1;
                println!("real {}: DIVERGENCE {why}", file.display());
            }
        }
    }
    println!("real packages: {} read by both, {bad} divergences", same);
    assert_eq!(bad, 0, "divergences on real packages");
}
