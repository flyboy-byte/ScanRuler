// SPDX-License-Identifier: AGPL-3.0-only
//! 3MF import: the original's `parse3MF` (`src/core/parsers/threemf.ts` at
//! `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`), body for body.
//!
//! A 3MF is a zip (read by [`crate::inflate`], which reproduces `fflate`) holding XML models. The
//! original reads them with a small scanner of its own, not an XML parser, so malformed XML is
//! read the way that scanner reads it. This port keeps JavaScript string semantics where they show:
//!
//! - text is UTF-16 (`Vec<u16>`), so a numeric entity naming a lone surrogate survives, and
//!   `&#x110000;` throws the original's `RangeError: Invalid code point 1114112`;
//! - numbers are read with `Number(text)` (the full String-to-Number grammar, not `parseFloat`);
//! - `Object.prototype` leaks through the original's plain-object lookups: a unit named
//!   `constructor` or `__proto__` yields a NaN scale, and a part path `constructor` or `__proto__`
//!   reaches `TextDecoder.decode` with a non-buffer, a `TypeError` whose text depends on the
//!   JavaScript engine ([`ThreeMfError::HostType`]);
//! - object, group and mesh identities follow the original's variables (`object`, `mesh`, `group`),
//!   including its behaviour on mis-nested or repeated elements.
//!
//! Host limits ([`ThreeMfLimits`]) are separate from the original's refusals: the original has no
//! cap on archive, model or placement size, so a refusal for size is [`ThreeMfError::Limit`], never
//! a source message.

use crate::inflate::{InflateError, ZipLimits, is_threemf_part, unzip_filtered};
use crate::obj::number as js_number;
use crate::obj_texture::{
    ObjCompanions, TextureError, TextureImage, TextureLimits, TextureNote, base_name,
    sample_texture,
};
use crate::ply_ascii::js_number_string;
use crate::stl_dispatch::decode_utf8;
use std::collections::{HashMap, HashSet};

/// Explicit host budgets. None of these exists in the original.
#[derive(Debug, Clone, Copy)]
pub struct ThreeMfLimits {
    /// Zip entry size and entry count.
    pub zip: ZipLimits,
    /// Total bytes of all kept parts (`.model`, `.rels`, `.config`).
    pub max_part_bytes: usize,
    /// Vertices read from one model file, and placed in the output.
    pub max_vertices: usize,
    /// Triangles read from one model file, and placed in the output.
    pub max_triangles: usize,
    /// Scanner units and placement visits across the whole read.
    pub max_work: usize,
    /// Limits for the shared texture helpers (`baseName`).
    pub texture: TextureLimits,
}

/// Which host budget a refusal ran into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LimitKind {
    /// A zip entry's declared size or the entry count ([`ZipLimits`]).
    Archive,
    /// The total bytes of the kept parts.
    PartBytes,
    /// Vertices in one model file or in the placed result.
    Vertices,
    /// Triangles in one model file or in the placed result.
    Triangles,
    /// The work budget (scanner units and placement visits).
    Work,
}

/// A 3MF read, or why not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreeMfError {
    /// The original's `Error` (or `RangeError`) message, as UTF-16 (it can hold a lone surrogate).
    Source(Vec<u16>),
    /// The original's `TypeError` from handing `TextDecoder.decode` a non-buffer (an
    /// `Object.prototype` name used as a part path). Its message is engine-specific.
    HostType,
    /// A host budget was exceeded; the original has no such limit.
    Limit(LimitKind),
    /// Storage could not be reserved.
    Allocation,
    /// A shared texture helper refused.
    Texture(TextureError),
}

impl ThreeMfError {
    /// The message text, lone surrogates replaced as a UTF-8 writer would.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Source(m) => String::from_utf16_lossy(m),
            Self::HostType => "TypeError: part is not an ArrayBuffer or view".into(),
            Self::Limit(kind) => format!("3MF exceeds the caller's resource budget ({kind:?})."),
            Self::Allocation => "Could not allocate 3MF storage.".into(),
            Self::Texture(e) => format!("3MF texture host refusal: {e:?}"),
        }
    }
}

impl std::fmt::Display for ThreeMfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

/// The indexed mesh `parse3MF` returns.
#[derive(Debug, Clone, PartialEq)]
pub struct ThreeMfMesh {
    /// XYZ, narrowed from Numbers to float32 at the end.
    pub positions: Vec<f32>,
    /// Triangle corners.
    pub indices: Vec<u32>,
    /// Three bytes per vertex, only when a texture applied or more than one colour was read.
    pub colors: Option<Vec<u8>>,
    /// Applied image names, or the names of the images that were missing.
    pub texture: Option<TextureNote>,
}

type S = Vec<u16>;
type Matrix = [f64; 12];

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
const UNCOLOURED: u8 = 180;
const MAX_DEPTH: usize = 32;
const NO_PID: u32 = u32::MAX;

fn source(message: &str) -> ThreeMfError {
    ThreeMfError::Source(message.encode_utf16().collect())
}

fn u(s: &str) -> S {
    s.encode_utf16().collect()
}

struct Work(usize);
impl Work {
    fn spend(&mut self, n: usize) -> Result<(), ThreeMfError> {
        self.0 = self
            .0
            .checked_sub(n)
            .ok_or(ThreeMfError::Limit(LimitKind::Work))?;
        Ok(())
    }
}

// ---- JavaScript string helpers over UTF-16 -------------------------------------------------

/// `/\s/` and `String.prototype.trim` agree on this set.
fn is_space16(c: u16) -> bool {
    matches!(c, 9..=13 | 32 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f
        | 0x205f | 0x3000 | 0xfeff)
}

fn trim16(s: &[u16]) -> &[u16] {
    let start = s.iter().position(|&c| !is_space16(c)).unwrap_or(s.len());
    let end = s
        .iter()
        .rposition(|&c| !is_space16(c))
        .map_or(start, |i| i + 1);
    &s[start..end]
}

fn eq16(a: &[u16], b: &str) -> bool {
    a.len() == b.len() && a.iter().zip(b.bytes()).all(|(x, y)| *x == u16::from(y))
}

fn find_unit(text: &[u16], unit: u16, from: usize) -> Option<usize> {
    let from = from.min(text.len());
    text[from..]
        .iter()
        .position(|&c| c == unit)
        .map(|i| i + from)
}

fn find_seq(text: &[u16], seq: &str, from: usize) -> Option<usize> {
    let pat: Vec<u16> = seq.encode_utf16().collect();
    let from = from.min(text.len());
    text[from..]
        .windows(pat.len())
        .position(|w| w == pat.as_slice())
        .map(|i| i + from)
}

fn starts_with16(text: &[u16], at: usize, seq: &str) -> bool {
    let n = seq.len();
    text.get(at..at + n).is_some_and(|w| eq16(w, seq))
}

/// `String.prototype.toLowerCase`, keeping lone surrogates.
fn lower16(s: &[u16]) -> S {
    if s.iter().all(|&c| c < 128) {
        return s
            .iter()
            .map(|&c| if (65..=90).contains(&c) { c + 32 } else { c })
            .collect();
    }
    let mut out = S::new();
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut S| {
        out.extend(run.to_lowercase().encode_utf16());
        run.clear();
    };
    for r in char::decode_utf16(s.iter().copied()) {
        match r {
            Ok(c) => run.push(c),
            Err(e) => {
                flush(&mut run, &mut out);
                out.push(e.unpaired_surrogate());
            }
        }
    }
    flush(&mut run, &mut out);
    out
}

/// The original's `partName`: no leading slashes, lower case.
fn part_name16(path: &[u16]) -> S {
    let start = path.iter().position(|&c| c != 47).unwrap_or(path.len());
    lower16(&path[start..])
}

fn part_name(path: &str) -> String {
    path.trim_start_matches('/').to_lowercase()
}

fn num(s: &[u16]) -> f64 {
    js_number(&String::from_utf16_lossy(s))
}

/// `Number(a.get(name))`: a missing attribute is NaN.
fn num_attr(attrs: &Attrs, name: &str) -> f64 {
    attrs.get(name).map_or(f64::NAN, num)
}

/// The original's `integer`: the number when it is an integer, otherwise -1.
fn integer(text: Option<&[u16]>) -> f64 {
    let n = text.map_or(f64::NAN, num);
    if n.is_finite() && n.fract() == 0.0 {
        n
    } else {
        -1.0
    }
}

fn index_of_slot(i: f64, len: usize) -> Option<usize> {
    (i >= 0.0 && i < len as f64).then_some(i as usize)
}

// ---- XML scanning ---------------------------------------------------------------------------

#[derive(Default)]
struct Attrs {
    items: Vec<(S, S)>,
}
impl Attrs {
    fn clear(&mut self) {
        self.items.clear();
    }
    fn set(&mut self, name: S, value: S) {
        match self.items.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = value,
            None => self.items.push((name, value)),
        }
    }
    fn get(&self, name: &str) -> Option<&[u16]> {
        self.items
            .iter()
            .find(|(n, _)| eq16(n, name))
            .map(|(_, v)| v.as_slice())
    }
    fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

trait Visitor {
    fn open(&mut self, name: &[u16], attrs: &Attrs, self_closing: bool)
    -> Result<(), ThreeMfError>;
    fn close(&mut self, name: &[u16]) -> Result<(), ThreeMfError>;
}

fn local_name(name: &[u16]) -> &[u16] {
    match name.iter().position(|&c| c == 58) {
        Some(colon) => &name[colon + 1..],
        None => name,
    }
}

/// The five named entities and numeric ones (`unescapeXml`). A code point beyond U+10FFFF throws
/// the original's `RangeError`.
fn unescape(s: &[u16]) -> Result<S, ThreeMfError> {
    if !s.contains(&38) {
        return Ok(s.to_vec());
    }
    let digit = |c: u16, hex: bool| match c {
        48..=57 => true,
        65..=70 | 97..=102 => hex,
        _ => false,
    };
    let mut out = S::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == 38
            && let Some((end, replacement)) = entity(s, i, &digit)?
        {
            out.extend(replacement);
            i = end;
            continue;
        }
        out.push(s[i]);
        i += 1;
    }
    Ok(out)
}

fn entity(
    s: &[u16],
    at: usize,
    digit: &impl Fn(u16, bool) -> bool,
) -> Result<Option<(usize, S)>, ThreeMfError> {
    let j = at + 1;
    let semi = |k: usize| s.get(k) == Some(&59);
    if s.get(j) == Some(&35) {
        let hex = matches!(s.get(j + 1), Some(120 | 88));
        let start = if hex { j + 2 } else { j + 1 };
        let mut k = start;
        while s.get(k).is_some_and(|&c| digit(c, hex)) {
            k += 1;
        }
        if k == start || !semi(k) {
            return Ok(None);
        }
        let digits = String::from_utf16_lossy(&s[start..k]);
        let value = if hex {
            js_number(&format!("0x{digits}"))
        } else {
            js_number(&digits)
        };
        if !(0.0..=1_114_111.0).contains(&value) || value.fract() != 0.0 {
            let mut m = u("Invalid code point ");
            m.extend(js_number_string(value).encode_utf16());
            return Err(ThreeMfError::Source(m));
        }
        let cp = value as u32;
        let replacement = match char::from_u32(cp) {
            Some(c) => c.to_string().encode_utf16().collect(),
            None => vec![cp as u16],
        };
        return Ok(Some((k + 1, replacement)));
    }
    for (name, text) in [
        ("lt", "<"),
        ("gt", ">"),
        ("amp", "&"),
        ("quot", "\""),
        ("apos", "'"),
    ] {
        let n = name.len();
        if let Some(w) = s.get(j..j + n)
            && w.iter()
                .zip(name.bytes())
                .all(|(x, y)| (*x | 0x20) == u16::from(y))
            && semi(j + n)
        {
            return Ok(Some((j + n + 1, u(text))));
        }
    }
    Ok(None)
}

/// `scanXml`: every element in order, with local names and unescaped attributes.
fn scan_xml(text: &[u16], work: &mut Work, v: &mut impl Visitor) -> Result<(), ThreeMfError> {
    work.spend(text.len())?;
    let len = text.len();
    let is_space = |c: u16| matches!(c, 32 | 9 | 10 | 13);
    let mut attrs = Attrs::default();
    let mut at = find_unit(text, 60, 0);
    while let Some(a) = at {
        if a >= len {
            break;
        }
        let next = text.get(a + 1).copied();
        if next == Some(33) {
            let end = if starts_with16(text, a, "<!--") {
                find_seq(text, "-->", a + 4)
            } else if starts_with16(text, a, "<![CDATA[") {
                find_seq(text, "]]>", a)
            } else {
                find_unit(text, 62, a)
            };
            let Some(end) = end else { return Ok(()) };
            at = find_unit(text, 60, end + 1);
            continue;
        }
        if next == Some(63) {
            let Some(end) = find_seq(text, "?>", a) else {
                return Ok(());
            };
            at = find_unit(text, 60, end + 2);
            continue;
        }
        // The tag's end: the first '>' outside a quoted value.
        let mut end = a + 1;
        let mut quote = 0u16;
        while end < len {
            let ch = text[end];
            if quote != 0 {
                if ch == quote {
                    quote = 0;
                }
            } else if ch == 34 || ch == 39 {
                quote = ch;
            } else if ch == 62 {
                break;
            }
            end += 1;
        }
        if end >= len {
            return Ok(());
        }
        if next == Some(47) {
            v.close(local_name(trim16(&text[a + 2..end])))?;
        } else {
            let self_closing = text.get(end - 1) == Some(&47);
            let stop = if self_closing { end - 1 } else { end };
            let mut i = a + 1;
            while i < stop && !is_space(text[i]) {
                i += 1;
            }
            let name = local_name(&text[a + 1..i]);
            attrs.clear();
            while i < stop {
                while i < stop && is_space(text[i]) {
                    i += 1;
                }
                let name_start = i;
                while i < stop && text[i] != 61 && !is_space(text[i]) {
                    i += 1;
                }
                let attr = &text[name_start..i];
                while i < stop && is_space(text[i]) {
                    i += 1;
                }
                if text.get(i) != Some(&61) {
                    i += 1;
                    continue;
                }
                i += 1;
                while i < stop && is_space(text[i]) {
                    i += 1;
                }
                let q = text.get(i).copied();
                let Some(q) = q.filter(|&q| q == 34 || q == 39) else {
                    break;
                };
                // A closing quote beyond `stop` is a break in the original, so search no further.
                let Some(value_end) = text[i + 1..stop.max(i + 1)]
                    .iter()
                    .position(|&c| c == q)
                    .map(|p| p + i + 1)
                else {
                    break;
                };
                if !attr.is_empty() {
                    attrs.set(
                        local_name(attr).to_vec(),
                        unescape(&text[i + 1..value_end])?,
                    );
                }
                i = value_end + 1;
            }
            v.open(name, &attrs, self_closing)?;
            if self_closing {
                v.close(name)?;
            }
        }
        at = find_unit(text, 60, end + 1);
    }
    Ok(())
}

// ---- Values ---------------------------------------------------------------------------------

/// `"#RRGGBB"` or `"#RRGGBBAA"` as 0xRRGGBB, or -1.
fn parse_color(text: Option<&[u16]>) -> i32 {
    let Some(t) = text.map(trim16) else { return -1 };
    let hex = |c: u16| matches!(c, 48..=57 | 65..=70 | 97..=102);
    if !(t.len() == 7 || t.len() == 9) || t[0] != 35 || !t[1..].iter().all(|&c| hex(c)) {
        return -1;
    }
    let mut v = 0i32;
    for &c in &t[1..7] {
        let d = match c {
            48..=57 => c - 48,
            65..=70 => c - 55,
            _ => c - 87,
        };
        v = v * 16 + i32::from(d);
    }
    v
}

fn parse_matrix(text: Option<&[u16]>) -> Result<Matrix, ThreeMfError> {
    let Some(text) = text.filter(|t| !t.is_empty()) else {
        return Ok(IDENTITY);
    };
    let trimmed = String::from_utf16_lossy(trim16(text));
    let tokens: Vec<&str> = if trimmed.is_empty() {
        vec![""]
    } else {
        trimmed
            .split(crate::stl_dispatch::js_space)
            .filter(|t| !t.is_empty())
            .collect()
    };
    if tokens.len() != 12 {
        return Err(source("Invalid 3MF transform."));
    }
    let mut m = [0.0; 12];
    for (slot, t) in m.iter_mut().zip(tokens) {
        *slot = js_number(t);
        if !slot.is_finite() {
            return Err(source("Invalid 3MF transform."));
        }
    }
    Ok(m)
}

/// `a` then `b`, with the original's operation order.
fn compose(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = [0.0; 12];
    for r in 0..4 {
        for c in 0..3 {
            out[r * 3 + c] = a[r * 3] * b[c]
                + a[r * 3 + 1] * b[3 + c]
                + a[r * 3 + 2] * b[6 + c]
                + if r == 3 { b[9 + c] } else { 0.0 };
        }
    }
    out
}

fn determinant(m: &Matrix) -> f64 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

/// `MM_PER_UNIT[unit.toLowerCase()]`, including the `Object.prototype` names that are not
/// `undefined` (the product with such a scale is NaN).
fn unit_scale(unit: &[u16]) -> Option<f64> {
    let lower = String::from_utf16_lossy(&lower16(unit));
    // A lone surrogate becomes U+FFFD in the lossy copy, which matches no name either way.
    Some(match lower.as_str() {
        "micron" => 0.001,
        "millimeter" => 1.0,
        "centimeter" => 10.0,
        "inch" => 25.4,
        "foot" => 304.8,
        "meter" => 1000.0,
        "constructor" | "__proto__" => f64::NAN,
        _ => return None,
    })
}

// ---- Model files ------------------------------------------------------------------------------

enum Group {
    Colors(Vec<i32>),
    Texture { texture: S, uv: Vec<f64> },
}

#[derive(Default)]
struct MeshObject {
    vertices: Vec<f64>,
    triangles: Vec<u32>,
    /// Per triangle: index into the model's `pid_names`, or `NO_PID`.
    pid: Vec<u32>,
    p: Vec<f64>,
}

struct Component {
    objectid: S,
    path: Option<S>,
    transform: Matrix,
}

struct ModelObject {
    ty: S,
    pid: Option<S>,
    pindex: Option<f64>,
    mesh: Option<usize>,
    components: Vec<Component>,
}

struct Item {
    objectid: S,
    path: Option<S>,
    transform: Matrix,
}

struct ModelFile {
    scale: f64,
    objects: Vec<ModelObject>,
    object_by_id: HashMap<S, usize>,
    meshes: Vec<MeshObject>,
    groups: Vec<Group>,
    group_by_id: HashMap<S, usize>,
    textures: HashMap<S, S>,
    build: Vec<Item>,
    pid_names: Vec<S>,
    pid_index: HashMap<S, u32>,
    /// `pid_names` resolved to groups once the file is read.
    pid_group: Vec<Option<usize>>,
}

struct ModelReader<'a> {
    model: ModelFile,
    object: Option<usize>,
    mesh: Option<usize>,
    group: Option<usize>,
    limits: &'a ThreeMfLimits,
}

impl ModelReader<'_> {
    fn pid_slot(&mut self, pid: &S) -> u32 {
        if let Some(&i) = self.model.pid_index.get(pid) {
            return i;
        }
        let i = self.model.pid_names.len() as u32;
        self.model.pid_names.push(pid.clone());
        self.model.pid_index.insert(pid.clone(), i);
        i
    }
}

impl Visitor for ModelReader<'_> {
    fn open(&mut self, name: &[u16], a: &Attrs, _self_closing: bool) -> Result<(), ThreeMfError> {
        let id = |a: &Attrs| a.get("id").map(<[u16]>::to_vec).unwrap_or_default();
        let n = String::from_utf16_lossy(name);
        match n.as_str() {
            "model" => {
                if let Some(unit) = a.get("unit") {
                    let Some(scale) = unit_scale(unit) else {
                        let mut m = u("3MF unit \"");
                        m.extend(unit);
                        m.extend(u(
                            "\" is not one of micron, millimeter, centimeter, inch, foot or meter.",
                        ));
                        return Err(ThreeMfError::Source(m));
                    };
                    self.model.scale = scale;
                }
            }
            "object" => {
                let pindex = integer(a.get("pindex"));
                let object = ModelObject {
                    ty: lower16(a.get("type").unwrap_or(&u("model"))),
                    pid: a.get("pid").map(<[u16]>::to_vec),
                    pindex: (pindex >= 0.0).then_some(pindex),
                    mesh: None,
                    components: Vec::new(),
                };
                self.model.objects.push(object);
                let at = self.model.objects.len() - 1;
                self.model.object_by_id.insert(id(a), at);
                self.object = Some(at);
            }
            "mesh" => {
                if let Some(o) = self.object {
                    self.model.meshes.push(MeshObject::default());
                    let at = self.model.meshes.len() - 1;
                    self.model.objects[o].mesh = Some(at);
                    self.mesh = Some(at);
                }
            }
            "vertex" => {
                if let Some(m) = self.mesh {
                    let (x, y, z) = (num_attr(a, "x"), num_attr(a, "y"), num_attr(a, "z"));
                    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                        return Err(source("Invalid 3MF vertex."));
                    }
                    let mesh = &mut self.model.meshes[m];
                    if mesh.vertices.len() / 3 >= self.limits.max_vertices {
                        return Err(ThreeMfError::Limit(LimitKind::Vertices));
                    }
                    mesh.vertices.extend([x, y, z]);
                }
            }
            "triangle" => {
                if let Some(m) = self.mesh {
                    let v1 = integer(a.get("v1"));
                    let v2 = integer(a.get("v2"));
                    let v3 = integer(a.get("v3"));
                    let count = (self.model.meshes[m].vertices.len() / 3) as f64;
                    if v1 < 0.0 || v2 < 0.0 || v3 < 0.0 || v1 >= count || v2 >= count || v3 >= count
                    {
                        return Err(source("Invalid 3MF triangle: a corner names no vertex."));
                    }
                    if self.model.meshes[m].triangles.len() / 3 >= self.limits.max_triangles {
                        return Err(ThreeMfError::Limit(LimitKind::Triangles));
                    }
                    let object = self.object.map(|o| &self.model.objects[o]);
                    let object_pid = object.and_then(|o| o.pid.clone());
                    let object_pindex = object.and_then(|o| o.pindex);
                    let pid = a.get("pid").map(<[u16]>::to_vec).or(object_pid);
                    let p1 = if a.has("p1") {
                        integer(a.get("p1"))
                    } else {
                        object_pindex.unwrap_or(-1.0)
                    };
                    let p2 = if a.has("p2") {
                        integer(a.get("p2"))
                    } else {
                        p1
                    };
                    let p3 = if a.has("p3") {
                        integer(a.get("p3"))
                    } else {
                        p1
                    };
                    let slot = match pid {
                        Some(pid) if p1 >= 0.0 => self.pid_slot(&pid),
                        _ => NO_PID,
                    };
                    let mesh = &mut self.model.meshes[m];
                    mesh.triangles.extend([v1 as u32, v2 as u32, v3 as u32]);
                    mesh.pid.push(slot);
                    mesh.p.extend([p1, p2, p3]);
                }
            }
            "component" => {
                if let Some(o) = self.object {
                    let component = Component {
                        objectid: a.get("objectid").map(<[u16]>::to_vec).unwrap_or_default(),
                        path: a.get("path").map(<[u16]>::to_vec),
                        transform: parse_matrix(a.get("transform"))?,
                    };
                    self.model.objects[o].components.push(component);
                }
            }
            "item" => {
                let item = Item {
                    objectid: a.get("objectid").map(<[u16]>::to_vec).unwrap_or_default(),
                    path: a.get("path").map(<[u16]>::to_vec),
                    transform: parse_matrix(a.get("transform"))?,
                };
                self.model.build.push(item);
            }
            "basematerials" | "colorgroup" => {
                self.model.groups.push(Group::Colors(Vec::new()));
                let at = self.model.groups.len() - 1;
                self.model.group_by_id.insert(id(a), at);
                self.group = Some(at);
            }
            "base" | "color" => {
                if let Some(g) = self.group
                    && let Group::Colors(rgb) = &mut self.model.groups[g]
                {
                    rgb.push(parse_color(a.get(if n == "base" {
                        "displaycolor"
                    } else {
                        "color"
                    })));
                }
            }
            "texture2d" => {
                self.model.textures.insert(
                    id(a),
                    a.get("path").map(<[u16]>::to_vec).unwrap_or_default(),
                );
            }
            "texture2dgroup" => {
                let texture = a.get("texid").map(<[u16]>::to_vec).unwrap_or_default();
                self.model.groups.push(Group::Texture {
                    texture,
                    uv: Vec::new(),
                });
                let at = self.model.groups.len() - 1;
                self.model.group_by_id.insert(id(a), at);
                self.group = Some(at);
            }
            "tex2coord" => {
                if let Some(g) = self.group
                    && let Group::Texture { uv, .. } = &mut self.model.groups[g]
                {
                    let (u_, v_) = (num_attr(a, "u"), num_attr(a, "v"));
                    uv.push(if u_.is_finite() { u_ } else { 0.0 });
                    uv.push(if v_.is_finite() { v_ } else { 0.0 });
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn close(&mut self, name: &[u16]) -> Result<(), ThreeMfError> {
        if eq16(name, "object") {
            self.object = None;
        } else if eq16(name, "mesh") {
            self.mesh = None;
        } else if eq16(name, "basematerials")
            || eq16(name, "colorgroup")
            || eq16(name, "texture2dgroup")
        {
            self.group = None;
        }
        Ok(())
    }
}

/// One model file of the package, read. `default_scale` is the root model's.
fn read_model(
    text: &[u16],
    default_scale: f64,
    limits: &ThreeMfLimits,
    work: &mut Work,
) -> Result<ModelFile, ThreeMfError> {
    let mut reader = ModelReader {
        model: ModelFile {
            scale: default_scale,
            objects: Vec::new(),
            object_by_id: HashMap::new(),
            meshes: Vec::new(),
            groups: Vec::new(),
            group_by_id: HashMap::new(),
            textures: HashMap::new(),
            build: Vec::new(),
            pid_names: Vec::new(),
            pid_index: HashMap::new(),
            pid_group: Vec::new(),
        },
        object: None,
        mesh: None,
        group: None,
        limits,
    };
    scan_xml(text, work, &mut reader)?;
    let mut model = reader.model;
    model.pid_group = model
        .pid_names
        .iter()
        .map(|p| model.group_by_id.get(p).copied())
        .collect();
    Ok(model)
}

// ---- Slicer metadata --------------------------------------------------------------------------

#[derive(Default)]
struct SlicerParts {
    objects: HashSet<S>,
    triangles: HashMap<S, Vec<(f64, f64)>>,
}

struct BambuVisitor<'a>(&'a mut HashSet<S>);
impl Visitor for BambuVisitor<'_> {
    fn open(&mut self, name: &[u16], a: &Attrs, _: bool) -> Result<(), ThreeMfError> {
        if eq16(name, "part")
            && let Some(subtype) = a.get("subtype")
            && !eq16(subtype, "normal_part")
        {
            self.0
                .insert(a.get("id").map(<[u16]>::to_vec).unwrap_or_default());
        }
        Ok(())
    }
    fn close(&mut self, _: &[u16]) -> Result<(), ThreeMfError> {
        Ok(())
    }
}

#[derive(Default)]
struct PrusaVisitor {
    object: S,
    volume: Option<(f64, f64)>,
    other: bool,
    runs: HashMap<S, Vec<(f64, f64)>>,
}
impl Visitor for PrusaVisitor {
    fn open(&mut self, name: &[u16], a: &Attrs, _: bool) -> Result<(), ThreeMfError> {
        if eq16(name, "object") {
            self.object = a.get("id").map(<[u16]>::to_vec).unwrap_or_default();
        } else if eq16(name, "volume") {
            self.volume = Some((integer(a.get("firstid")), integer(a.get("lastid"))));
            self.other = false;
        } else if eq16(name, "metadata")
            && self.volume.is_some()
            && a.get("type").is_some_and(|t| eq16(t, "volume"))
        {
            let key = a.get("key");
            let value = a.get("value").unwrap_or(&[]);
            let modifier = key.is_some_and(|k| eq16(k, "modifier")) && eq16(value, "1");
            let typed = key.is_some_and(|k| eq16(k, "volume_type")) && !eq16(value, "ModelPart");
            if modifier || typed {
                self.other = true;
            }
        }
        Ok(())
    }
    fn close(&mut self, name: &[u16]) -> Result<(), ThreeMfError> {
        if eq16(name, "volume")
            && let Some(v) = self.volume
        {
            if self.other && v.0 >= 0.0 && v.1 >= v.0 {
                self.runs.entry(self.object.clone()).or_default().push(v);
            }
            self.volume = None;
        }
        Ok(())
    }
}

// ---- Package ------------------------------------------------------------------------------------

/// The kept parts by lower-cased part name, in the order a JS object lists them.
struct Files {
    entries: Vec<(String, Vec<u8>)>,
    index: HashMap<String, usize>,
}

/// What `files[key]` is in the original.
enum Found<'a> {
    Bytes(&'a [u8]),
    /// `Object.prototype.constructor` or `__proto__`: truthy, but not a buffer.
    Garbage,
}

impl Files {
    fn get(&self, key: &str) -> Option<Found<'_>> {
        if key == "constructor" || key == "__proto__" {
            return Some(Found::Garbage);
        }
        self.index
            .get(key)
            .map(|&i| Found::Bytes(&self.entries[i].1))
    }
    fn get16(&self, key: &[u16]) -> Option<Found<'_>> {
        String::from_utf16(key).ok().and_then(|k| self.get(&k))
    }
}

fn text_of(found: &Found<'_>) -> Result<S, ThreeMfError> {
    match found {
        Found::Garbage => Err(ThreeMfError::HostType),
        Found::Bytes(b) => {
            let text = decode_utf8(b).map_err(|_| ThreeMfError::Allocation)?;
            Ok(text.encode_utf16().collect())
        }
    }
}

struct RelsVisitor(Option<S>);
impl Visitor for RelsVisitor {
    fn open(&mut self, name: &[u16], a: &Attrs, _: bool) -> Result<(), ThreeMfError> {
        if self.0.is_none() && eq16(name, "Relationship") {
            let ty = a.get("Type").unwrap_or(&[]);
            let tail = b"/3dmodel";
            let matches = ty.len() >= tail.len()
                && ty[ty.len() - tail.len()..]
                    .iter()
                    .zip(tail)
                    .all(|(c, t)| *c < 128 && (*c as u8).eq_ignore_ascii_case(t));
            if matches {
                self.0 = a.get("Target").map(<[u16]>::to_vec);
            }
        }
        Ok(())
    }
    fn close(&mut self, _: &[u16]) -> Result<(), ThreeMfError> {
        Ok(())
    }
}

/// The root model's part name, as `_rels/.rels` names it.
fn root_model_path(files: &Files, work: &mut Work) -> Result<String, ThreeMfError> {
    let mut target = None;
    if let Some(rels) = files.get("_rels/.rels") {
        let mut v = RelsVisitor(None);
        scan_xml(&text_of(&rels)?, work, &mut v)?;
        target = v.0;
    }
    if let Some(t) = target {
        let key = part_name16(&t);
        if files.get16(&key).is_some()
            && let Ok(key) = String::from_utf16(&key)
        {
            return Ok(key);
        }
    }
    let fallback = "3d/3dmodel.model";
    if files.get(fallback).is_some() {
        return Ok(fallback.into());
    }
    files
        .entries
        .iter()
        .map(|(k, _)| k)
        .find(|k| k.ends_with(".model"))
        .cloned()
        .ok_or_else(|| source("This 3MF holds no model."))
}

// ---- Placement ----------------------------------------------------------------------------------

struct Placer<'a> {
    files: &'a Files,
    limits: &'a ThreeMfLimits,
    work: &'a mut Work,
    /// Model files by part name; index 0 is the root.
    models: Vec<ModelFile>,
    model_index: HashMap<String, usize>,
    slicer: SlicerParts,
    positions: Vec<f64>,
    indices: Vec<u32>,
    vertex_color: Vec<i32>,
    textures: HashMap<S, Option<usize>>,
    missing: Vec<S>,
    applied: Vec<String>,
    images: &'a [TextureImage],
    by_part: HashMap<String, usize>,
    by_base: HashMap<String, usize>,
}

impl Placer<'_> {
    fn model_at(&mut self, path: Option<&S>) -> Result<usize, ThreeMfError> {
        let Some(path) = path.filter(|p| !p.is_empty()) else {
            return Ok(0);
        };
        let key = part_name16(path);
        let key_text = String::from_utf16(&key).ok();
        if let Some(i) = key_text.as_ref().and_then(|k| self.model_index.get(k)) {
            return Ok(*i);
        }
        let Some(found) = self.files.get16(&key) else {
            let mut m = u("This 3MF names ");
            m.extend(path);
            m.extend(u(", which it does not hold."));
            return Err(ThreeMfError::Source(m));
        };
        let text = text_of(&found)?;
        let model = read_model(&text, self.models[0].scale, self.limits, self.work)?;
        self.models.push(model);
        let at = self.models.len() - 1;
        if let Some(k) = key_text {
            self.model_index.insert(k, at);
        }
        Ok(at)
    }

    fn image_for(&mut self, path: &S) -> Result<Option<usize>, ThreeMfError> {
        if let Some(found) = self.textures.get(path) {
            return Ok(*found);
        }
        let found = match String::from_utf16(path) {
            Ok(p) => match self.by_part.get(&part_name(&p)) {
                Some(&i) => Some(i),
                None => {
                    let base = base_name(&p, self.limits.texture).map_err(ThreeMfError::Texture)?;
                    self.by_base.get(&base).copied()
                }
            },
            // A name holding a lone surrogate equals no given file name.
            Err(_) => None,
        };
        self.textures.insert(path.clone(), found);
        match found {
            Some(i) => {
                let name = &self.images[i].name;
                let last = name.rsplit('/').next().unwrap_or(name).to_string();
                if !self.applied.contains(&last) {
                    self.applied.push(last);
                }
            }
            None => {
                let start = path.iter().rposition(|&c| c == 47).map_or(0, |i| i + 1);
                let last = path[start..].to_vec();
                if !self.missing.contains(&last) {
                    self.missing.push(last);
                }
            }
        }
        Ok(found)
    }

    fn place(
        &mut self,
        model: usize,
        id: &S,
        transform: &Matrix,
        depth: usize,
    ) -> Result<(), ThreeMfError> {
        self.work.spend(1)?;
        if depth > MAX_DEPTH {
            return Err(source("This 3MF's components name each other in a ring."));
        }
        let Some(&oi) = self.models[model].object_by_id.get(id) else {
            let mut m = u("This 3MF names object ");
            m.extend(id);
            m.extend(u(", which it does not hold."));
            return Err(ThreeMfError::Source(m));
        };
        {
            let ty = &self.models[model].objects[oi].ty;
            let not_part = eq16(ty, "support") || eq16(ty, "solidsupport") || eq16(ty, "other");
            if not_part || self.slicer.objects.contains(id) {
                return Ok(());
            }
        }
        let components = self.models[model].objects[oi].components.len();
        for ci in 0..components {
            let (path, objectid, local) = {
                let c = &self.models[model].objects[oi].components[ci];
                (c.path.clone(), c.objectid.clone(), c.transform)
            };
            let child = match path.as_ref().filter(|p| !p.is_empty()) {
                Some(p) => self.model_at(Some(p))?,
                None => model,
            };
            self.place(child, &objectid, &compose(&local, transform), depth + 1)?;
        }
        let Some(mi) = self.models[model].objects[oi].mesh else {
            return Ok(());
        };
        self.place_mesh(model, mi, id, transform)
    }

    fn place_mesh(
        &mut self,
        model: usize,
        mi: usize,
        id: &S,
        m: &Matrix,
    ) -> Result<(), ThreeMfError> {
        let skip: Vec<(f64, f64)> = self.slicer.triangles.get(id).cloned().unwrap_or_default();
        let (nv, nt) = {
            let mesh = &self.models[model].meshes[mi];
            (mesh.vertices.len() / 3, mesh.triangles.len() / 3)
        };
        self.work
            .spend(nv.saturating_add(nt.saturating_mul(1 + skip.len())))?;
        if self.positions.len() / 3 + nv > self.limits.max_vertices {
            return Err(ThreeMfError::Limit(LimitKind::Vertices));
        }
        if self.indices.len() / 3 + nt > self.limits.max_triangles {
            return Err(ThreeMfError::Limit(LimitKind::Triangles));
        }
        let s = self.models[model].scale;
        let base = self.positions.len() / 3;
        {
            let v = &self.models[model].meshes[mi].vertices;
            for i in (0..v.len()).step_by(3) {
                self.positions
                    .push((v[i] * m[0] + v[i + 1] * m[3] + v[i + 2] * m[6] + m[9]) * s);
                self.positions
                    .push((v[i] * m[1] + v[i + 1] * m[4] + v[i + 2] * m[7] + m[10]) * s);
                self.positions
                    .push((v[i] * m[2] + v[i + 1] * m[5] + v[i + 2] * m[8] + m[11]) * s);
                self.vertex_color.push(-1);
            }
        }
        // A mirroring transform turns the triangles inside out; turned back.
        let flip = determinant(m) < 0.0;
        for k in 0..nt {
            if skip.iter().any(|(a, b)| k as f64 >= *a && k as f64 <= *b) {
                continue;
            }
            let (t, pid_slot) = {
                let mesh = &self.models[model].meshes[mi];
                (
                    [
                        mesh.triangles[k * 3],
                        mesh.triangles[k * 3 + 1],
                        mesh.triangles[k * 3 + 2],
                    ],
                    mesh.pid[k],
                )
            };
            let (b, c) = if flip { (t[2], t[1]) } else { (t[1], t[2]) };
            self.indices.push(base as u32 + t[0]);
            self.indices.push(base as u32 + b);
            self.indices.push(base as u32 + c);
            let group = if pid_slot == NO_PID {
                None
            } else {
                self.models[model].pid_group[pid_slot as usize]
            };
            let Some(group) = group else { continue };
            for (corner, &vertex_in_mesh) in t.iter().enumerate() {
                let vertex = base + vertex_in_mesh as usize;
                if self.vertex_color[vertex] >= 0 {
                    continue;
                }
                let p = self.models[model].meshes[mi].p[k * 3 + corner];
                enum Pick {
                    Color(Option<i32>),
                    Texture(S),
                }
                let pick = match &self.models[model].groups[group] {
                    Group::Colors(rgb) => Pick::Color(index_of_slot(p, rgb.len()).map(|i| rgb[i])),
                    Group::Texture { texture, .. } => Pick::Texture(texture.clone()),
                };
                match pick {
                    Pick::Color(Some(rgb)) if rgb >= 0 => self.vertex_color[vertex] = rgb,
                    Pick::Color(_) => {}
                    Pick::Texture(texture) => {
                        let path = self.models[model]
                            .textures
                            .get(&texture)
                            .filter(|p| !p.is_empty())
                            .cloned();
                        let img = match path {
                            Some(path) => self.image_for(&path)?,
                            None => None,
                        };
                        let Some(img) = img else { continue };
                        let uv = match &self.models[model].groups[group] {
                            Group::Texture { uv, .. } => uv,
                            Group::Colors(_) => continue,
                        };
                        if p >= 0.0 && p * 2.0 + 1.0 < uv.len() as f64 {
                            let at = (p * 2.0) as usize;
                            let (uu, vv) = (uv[at], uv[at + 1]);
                            let mut rgb = [0u8; 3];
                            sample_texture(&self.images[img], uu, vv, &mut rgb, 0)
                                .map_err(ThreeMfError::Texture)?;
                            self.vertex_color[vertex] = (i32::from(rgb[0]) << 16)
                                | (i32::from(rgb[1]) << 8)
                                | i32::from(rgb[2]);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// `parse3MF`: a 3MF package as an indexed mesh in millimetres.
///
/// `companions` are the already decoded images given with the file (textures named by the model
/// are matched to them). Progress text is reported exactly as the original reports it.
///
/// # Errors
/// The original's refusals as [`ThreeMfError::Source`] with its exact message, or a host refusal.
pub fn read_3mf(
    bytes: &[u8],
    companions: Option<&ObjCompanions>,
    limits: ThreeMfLimits,
    mut on_progress: impl FnMut(&str),
) -> Result<ThreeMfMesh, ThreeMfError> {
    on_progress("Reading 3MF…");
    let zipped = match unzip_filtered(bytes, is_threemf_part, limits.zip) {
        Ok(z) => z,
        Err(InflateError::ResourceLimit) => return Err(ThreeMfError::Limit(LimitKind::Archive)),
        Err(_) => {
            return Err(source(
                "This 3MF could not be unzipped — the file is damaged or not a 3MF.",
            ));
        }
    };
    let mut total = 0usize;
    let mut files = Files {
        entries: Vec::new(),
        index: HashMap::new(),
    };
    for (name, data) in zipped {
        total = total.saturating_add(data.len());
        if total > limits.max_part_bytes {
            return Err(ThreeMfError::Limit(LimitKind::PartBytes));
        }
        let key = part_name(&name);
        match files.index.get(&key) {
            Some(&i) => files.entries[i].1 = data,
            None => {
                files.index.insert(key.clone(), files.entries.len());
                files.entries.push((key, data));
            }
        }
    }
    let mut work = Work(limits.max_work);
    let root_path = root_model_path(&files, &mut work)?;
    let root_text = match files.get(&root_path) {
        Some(found) => text_of(&found)?,
        None => return Err(ThreeMfError::HostType),
    };
    let root = read_model(&root_text, 1.0, &limits, &mut work)?;

    let mut slicer = SlicerParts::default();
    if let Some(found) = files.get("metadata/model_settings.config") {
        scan_xml(
            &text_of(&found)?,
            &mut work,
            &mut BambuVisitor(&mut slicer.objects),
        )?;
    }
    if let Some(found) = files.get("metadata/slic3r_pe_model.config") {
        let mut v = PrusaVisitor::default();
        scan_xml(&text_of(&found)?, &mut work, &mut v)?;
        slicer.triangles = v.runs;
    }

    let given: &[TextureImage] = companions.map_or(&[], |c| c.images.as_slice());
    let mut by_part = HashMap::new();
    let mut by_base = HashMap::new();
    for (i, img) in given.iter().enumerate() {
        by_part.insert(part_name(&img.name), i);
        by_base.insert(
            base_name(&img.name, limits.texture).map_err(ThreeMfError::Texture)?,
            i,
        );
    }

    let mut model_index = HashMap::new();
    model_index.insert(root_path, 0);
    let build: Vec<(S, Option<S>, Matrix)> = root
        .build
        .iter()
        .map(|i| (i.objectid.clone(), i.path.clone(), i.transform))
        .collect();
    let mut placer = Placer {
        files: &files,
        limits: &limits,
        work: &mut work,
        models: vec![root],
        model_index,
        slicer,
        positions: Vec::new(),
        indices: Vec::new(),
        vertex_color: Vec::new(),
        textures: HashMap::new(),
        missing: Vec::new(),
        applied: Vec::new(),
        images: given,
        by_part,
        by_base,
    };
    on_progress("Placing the 3MF's objects…");
    for (objectid, path, transform) in &build {
        let model = placer.model_at(path.as_ref())?;
        placer.place(model, objectid, transform, 0)?;
    }
    if placer.indices.is_empty() {
        return Err(source(
            "This 3MF holds no triangles of a part — point clouds and support-only files are not supported.",
        ));
    }

    let positions: Vec<f32> = placer.positions.iter().map(|&v| v as f32).collect();
    // Colour only when it says something: a texture, or more than one colour.
    let mut distinct = HashSet::new();
    for &c in placer.vertex_color.iter().filter(|&&c| c >= 0) {
        distinct.insert(c);
    }
    let colors = (!placer.applied.is_empty() || distinct.len() > 1).then(|| {
        let mut out = Vec::with_capacity(placer.vertex_color.len() * 3);
        for &c in &placer.vertex_color {
            if c >= 0 {
                out.extend([(c >> 16) as u8, ((c >> 8) & 0xff) as u8, (c & 0xff) as u8]);
            } else {
                out.extend([UNCOLOURED; 3]);
            }
        }
        out
    });
    let texture = if !placer.applied.is_empty() {
        Some(TextureNote::Applied(placer.applied))
    } else if !placer.missing.is_empty() {
        Some(TextureNote::Missing(
            placer
                .missing
                .iter()
                .map(|m| String::from_utf16_lossy(m))
                .collect(),
        ))
    } else {
        None
    };
    Ok(ThreeMfMesh {
        positions,
        indices: placer.indices,
        colors,
        texture,
    })
}
