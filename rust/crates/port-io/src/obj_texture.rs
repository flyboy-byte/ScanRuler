// SPDX-License-Identifier: AGPL-3.0-only
//! Complete pinned OBJ material/decoded-RGBA texture helpers. Image decoding and
//! platform file collection are separate boundaries; no foreign image code here.
use crate::stl_dispatch::js_space;

/// Explicit host limits, not alternative source formulas or silent truncation.
#[derive(Debug, Clone, Copy)]
pub struct TextureLimits {
    /// Aggregate borrowed input bytes.
    pub max_bytes: usize,
    /// Maximum output vertices.
    pub max_vertices: usize,
    /// Maximum given files and declared libraries of each kind.
    pub max_files: usize,
    /// Maximum material/map entries.
    pub max_materials: usize,
    /// Maximum UTF-8 bytes in a name/path.
    pub max_name_bytes: usize,
    /// Text, lookup-byte and vertex visits across the whole operation.
    pub max_work: usize,
}
/// Safe host refusals; source helpers have no typed refusal on these inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    /// Input, count or derived size exceeds its caller budget.
    Limit,
    /// Work budget exhausted.
    Work,
    /// Output cannot hold the three sampled bytes.
    Output,
    /// Fallible buffer reservation failed.
    Allocation,
}
/// Decoded RGBA rows from top to bottom. Sampling ignores alpha as the source does.
#[derive(Debug, Clone)]
pub struct TextureImage {
    /// Original given name, retained in the applied note.
    pub name: String,
    /// Pixel width; zero/short arrays preserve source undefined-sample behavior.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Four bytes per pixel for a normal decoded image.
    pub rgba: Vec<u8>,
}
/// A given MTL file as decoded text.
#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    /// Original given path/name.
    pub name: String,
    /// Decoded original text.
    pub text: String,
}
/// Already decoded companion files; original file-array order matters.
#[derive(Debug, Clone, Default)]
pub struct ObjCompanions {
    /// MTL files, with last duplicate normalized filename winning.
    pub materials: Vec<MaterialLibrary>,
    /// Decoded images, including their original order before name lookup.
    pub images: Vec<TextureImage>,
}
/// References gathered by the complete OBJ parser, not welded or repaired here.
#[derive(Debug, Clone, Default)]
pub struct ObjTextureRefs {
    /// Texture coordinate pairs; missing/nonfinite direct-helper samples stay observable.
    pub uvs: Vec<f64>,
    /// First textured corner per vertex, -1 when absent.
    pub vertex_uv: Vec<i32>,
    /// Material per first textured corner, negative for no material.
    pub vertex_material: Vec<i32>,
    /// Material names by index.
    pub material_names: Vec<String>,
    /// Named libraries in original order.
    pub libraries: Vec<String>,
}
/// Exact note variant and ordered original names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureNote {
    /// Missing filenames in first-seen order.
    Missing(Vec<String>),
    /// Actually sampled original image names in vertex encounter order.
    Applied(Vec<String>),
}
/// Source result; None from bake means no texture note at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakedTexture {
    /// Missing note has no color array; applied may have a legitimate empty array.
    pub colors: Option<Vec<u8>>,
    /// Missing or applied, preserving order and empty names.
    pub note: TextureNote,
}

struct Budget {
    limits: TextureLimits,
    left: usize,
}
impl Budget {
    fn new(limits: TextureLimits) -> Self {
        Self {
            limits,
            left: limits.max_work,
        }
    }
    fn spend(&mut self, n: usize) -> Result<(), TextureError> {
        self.left = self.left.checked_sub(n).ok_or(TextureError::Work)?;
        Ok(())
    }
    fn text(&mut self, s: &str) -> Result<(), TextureError> {
        if s.len() > self.limits.max_bytes {
            return Err(TextureError::Limit);
        }
        self.spend(s.len().saturating_add(1))
    }
    fn name(&mut self, s: &str) -> Result<(), TextureError> {
        if s.len() > self.limits.max_name_bytes {
            return Err(TextureError::Limit);
        }
        self.text(s)
    }
}
fn push<T>(v: &mut Vec<T>, value: T) -> Result<(), TextureError> {
    v.try_reserve(1).map_err(|_| TextureError::Allocation)?;
    v.push(value);
    Ok(())
}
fn own(s: &str) -> Result<String, TextureError> {
    let mut v = String::new();
    v.try_reserve_exact(s.len())
        .map_err(|_| TextureError::Allocation)?;
    v.push_str(s);
    Ok(v)
}
fn tail(s: &str) -> &str {
    s.rsplit(['/', '\\']).next().unwrap_or(s)
}
fn base(s: &str, b: &mut Budget) -> Result<String, TextureError> {
    b.name(s)?;
    // Standard Unicode lowercase includes the source's contextual final sigma.
    // Names are independently capped, so this bounded temporary cannot scale
    // with raster sizes or vertex counts. No Unicode normalization is applied.
    Ok(tail(s).trim_matches(js_space).to_lowercase())
}
/// Last path component, source trim and Unicode lowercase.
pub fn base_name(path: &str, limits: TextureLimits) -> Result<String, TextureError> {
    base(path, &mut Budget::new(limits))
}

fn find<V>(
    entries: &[(String, V)],
    key: &str,
    b: &mut Budget,
) -> Result<Option<usize>, TextureError> {
    for (i, (k, _)) in entries.iter().enumerate() {
        b.spend(k.len().min(key.len()).saturating_add(1))?;
        if k == key {
            return Ok(Some(i));
        }
    }
    Ok(None)
}
fn set<V>(
    entries: &mut Vec<(String, V)>,
    key: &str,
    value: V,
    max: usize,
    b: &mut Budget,
) -> Result<(), TextureError> {
    if let Some(i) = find(entries, key, b)? {
        entries[i].1 = value;
    } else {
        if entries.len() >= max {
            return Err(TextureError::Limit);
        }
        push(entries, (own(key)?, value))?;
    }
    Ok(())
}
fn insert_name(names: &mut Vec<String>, name: &str, b: &mut Budget) -> Result<(), TextureError> {
    b.name(name)?;
    for old in names.iter() {
        b.spend(old.len().min(name.len()).saturating_add(1))?;
        if old == name {
            return Ok(());
        }
    }
    push(names, own(name)?)
}
fn option_count(s: &str) -> Option<usize> {
    match s {
        "-blendu" | "-blendv" | "-cc" | "-clamp" | "-imfchan" | "-texres" | "-bm" | "-boost"
        | "-type" => Some(1),
        "-mm" => Some(2),
        "-o" | "-s" | "-t" => Some(3),
        _ => None,
    }
}
fn option_value(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "on" | "off" | "r" | "g" | "b" | "m" | "l" | "z"
    ) || {
        let rest = s.strip_prefix('-').unwrap_or(s);
        !rest.is_empty() && rest.bytes().all(|c| c.is_ascii_digit() || c == b'.')
    }
}
fn parse_mtl_inner(text: &str, b: &mut Budget) -> Result<Vec<(String, String)>, TextureError> {
    b.text(text)?;
    let mut out = Vec::new();
    let mut current = None;
    for raw in text.split('\n') {
        b.spend(1)?;
        let line = raw.split('#').next().unwrap_or("").trim_matches(js_space);
        if let Some(name) = line.strip_prefix("newmtl") {
            let name = name.trim_matches(js_space);
            b.name(name)?;
            current = Some(name);
        } else if let Some(name) = current.filter(|_| {
            line.get(..6)
                .is_some_and(|s| s.eq_ignore_ascii_case("map_Kd"))
                && line
                    .get(6..)
                    .and_then(|s| s.chars().next())
                    .is_some_and(js_space)
        }) {
            let mut tokens = Vec::new();
            for t in line.split(js_space).filter(|s| !s.is_empty()).skip(1) {
                push(&mut tokens, t)?;
            }
            let mut i = 0;
            while i < tokens.len() && tokens[i].starts_with('-') {
                let Some(takes) = option_count(&tokens[i].to_ascii_lowercase()) else {
                    break;
                };
                i += 1;
                for _ in 0..takes {
                    if i < tokens.len().saturating_sub(1) && option_value(tokens[i]) {
                        i += 1;
                    } else {
                        break;
                    }
                }
            }
            if i < tokens.len() {
                let size = tokens[i..]
                    .iter()
                    .try_fold(0usize, |n, s| {
                        n.checked_add(s.len()).and_then(|n| n.checked_add(1))
                    })
                    .ok_or(TextureError::Limit)?;
                if size.saturating_sub(1) > b.limits.max_name_bytes {
                    return Err(TextureError::Limit);
                }
                let mut file = String::new();
                file.try_reserve_exact(size)
                    .map_err(|_| TextureError::Allocation)?;
                for t in &tokens[i..] {
                    if !file.is_empty() {
                        file.push(' ');
                    }
                    file.push_str(t);
                }
                set(&mut out, name, file, b.limits.max_materials, b)?;
            }
        }
    }
    Ok(out)
}
/// Original ordered material map, including option skipping and replacement order.
pub fn parse_mtl(text: &str, limits: TextureLimits) -> Result<Vec<(String, String)>, TextureError> {
    parse_mtl_inner(text, &mut Budget::new(limits))
}
/// Source mtllib splitting: separate names only when every token ends in .mtl.
pub fn mtllib_names(rest: &str, limits: TextureLimits) -> Result<Vec<String>, TextureError> {
    let mut b = Budget::new(limits);
    b.text(rest)?;
    let text = rest.trim_matches(js_space);
    let mut tokens = Vec::new();
    for t in text.split(js_space).filter(|s| !s.is_empty()) {
        push(&mut tokens, t)?;
    }
    let split = tokens.len() > 1
        && tokens.iter().all(|t| {
            t.get(t.len().saturating_sub(4)..)
                .is_some_and(|s| s.eq_ignore_ascii_case(".mtl"))
        });
    let mut out = Vec::new();
    if split {
        for t in tokens {
            b.name(t)?;
            if out.len() >= limits.max_files {
                return Err(TextureError::Limit);
            }
            push(&mut out, own(t)?)?;
        }
    } else if !text.is_empty() {
        b.name(text)?;
        if limits.max_files == 0 {
            return Err(TextureError::Limit);
        }
        push(&mut out, own(text)?)?;
    }
    Ok(out)
}
fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}
fn channel(img: &TextureImage, index: f64) -> f64 {
    if index.is_finite() && index >= 0.0 && index.fract() == 0.0 && index < img.rgba.len() as f64 {
        f64::from(img.rgba[index as usize])
    } else {
        f64::NAN
    }
}
fn round_byte(v: f64) -> u8 {
    // Do not use floor(v + 0.5): addition can round a just-below-half value up.
    let lower = v.floor();
    let rounded = if v - lower < 0.5 { lower } else { lower + 1.0 };
    if rounded.is_finite() {
        rounded.rem_euclid(256.0) as u8
    } else {
        0
    }
}
/// Complete source bilinear sampler, repeating U and clamping the normalized V rows.
/// A too-short output is a separate atomic host refusal. Undefined input channels
/// propagate NaN to the source Uint8 zero conversion, not a fabricated pixel.
pub fn sample_texture(
    img: &TextureImage,
    u: f64,
    v: f64,
    out: &mut [u8],
    offset: usize,
) -> Result<(), TextureError> {
    let end = offset.checked_add(3).ok_or(TextureError::Output)?;
    if end > out.len() {
        return Err(TextureError::Output);
    }
    let w = f64::from(img.width);
    let h = f64::from(img.height);
    let fu = u - u.floor();
    let fv = v - v.floor();
    let x = fu * w - 0.5;
    let y = (1.0 - fv) * h - 0.5;
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = x - x0;
    let ty = y - y0;
    let xa = ((x0 % w) + w) % w;
    let xb = (xa + 1.0) % w;
    let ya = js_min(js_max(y0, 0.0), h - 1.0);
    let yb = js_min(js_max(y0 + 1.0, 0.0), h - 1.0);
    let p00 = (ya * w + xa) * 4.0;
    let p10 = (ya * w + xb) * 4.0;
    let p01 = (yb * w + xa) * 4.0;
    let p11 = (yb * w + xb) * 4.0;
    for c in 0..3 {
        let c1 = c as f64;
        let top = channel(img, p00 + c1) * (1.0 - tx) + channel(img, p10 + c1) * tx;
        let bottom = channel(img, p01 + c1) * (1.0 - tx) + channel(img, p11 + c1) * tx;
        out[offset + c] = round_byte(top * (1.0 - ty) + bottom * ty);
    }
    Ok(())
}
fn total_bytes(
    refs: &ObjTextureRefs,
    given: Option<&ObjCompanions>,
    limits: TextureLimits,
) -> Result<(), TextureError> {
    if refs.vertex_uv.len() > limits.max_vertices
        || refs.material_names.len() > limits.max_materials
        || refs.libraries.len() > limits.max_files
    {
        return Err(TextureError::Limit);
    }
    let mut size = refs
        .uvs
        .len()
        .checked_mul(8)
        .and_then(|s| {
            refs.vertex_uv
                .len()
                .checked_add(refs.vertex_material.len())
                .and_then(|n| n.checked_mul(4))
                .and_then(|n| n.checked_add(s))
        })
        .ok_or(TextureError::Limit)?;
    let mut add = |n: usize| -> Result<(), TextureError> {
        size = size.checked_add(n).ok_or(TextureError::Limit)?;
        if size > limits.max_bytes {
            Err(TextureError::Limit)
        } else {
            Ok(())
        }
    };
    add(0)?;
    for s in refs.material_names.iter().chain(&refs.libraries) {
        if s.len() > limits.max_name_bytes {
            return Err(TextureError::Limit);
        }
        add(s.len())?;
    }
    if let Some(g) = given {
        if g.materials.len() > limits.max_files || g.images.len() > limits.max_files {
            return Err(TextureError::Limit);
        }
        for m in &g.materials {
            if m.name.len() > limits.max_name_bytes {
                return Err(TextureError::Limit);
            }
            add(m.name.len())?;
            add(m.text.len())?;
        }
        for i in &g.images {
            if i.name.len() > limits.max_name_bytes {
                return Err(TextureError::Limit);
            }
            add(i.name.len())?;
            add(i.rgba.len())?;
        }
    }
    Ok(())
}
fn named(
    file: Option<&str>,
    images: &[(String, usize)],
    b: &mut Budget,
) -> Result<Option<usize>, TextureError> {
    if let Some(file) = file.filter(|s| !s.is_empty()) {
        let key = base(file, b)?;
        Ok(find(images, &key, b)?.map(|i| images[i].1))
    } else {
        Ok(None)
    }
}
/// Complete source texture bake on decoded companions, preserving null/missing/
/// applied distinctions, map precedence, fallback selection and ordered notes.
pub fn bake_obj_texture(
    refs: &ObjTextureRefs,
    given: Option<&ObjCompanions>,
    limits: TextureLimits,
) -> Result<Option<BakedTexture>, TextureError> {
    total_bytes(refs, given, limits)?;
    let mut b = Budget::new(limits);
    let no_images = given.is_none_or(|g| g.images.is_empty());
    if refs.uvs.is_empty() || (refs.libraries.is_empty() && no_images) {
        return Ok(None);
    }
    let empty = ObjCompanions::default();
    let given = given.unwrap_or(&empty);
    let mut libraries = Vec::new();
    for m in &given.materials {
        let key = base(&m.name, &mut b)?;
        set(
            &mut libraries,
            &key,
            m.text.as_str(),
            limits.max_files,
            &mut b,
        )?;
    }
    let mut images = Vec::new();
    for (i, img) in given.images.iter().enumerate() {
        let key = base(&img.name, &mut b)?;
        set(&mut images, &key, i, limits.max_files, &mut b)?;
    }
    let mut missing = Vec::new();
    let mut maps = Vec::new();
    for lib in &refs.libraries {
        let key = base(lib, &mut b)?;
        let Some(i) = find(&libraries, &key, &mut b)? else {
            insert_name(&mut missing, tail(lib), &mut b)?;
            continue;
        };
        for (name, file) in parse_mtl_inner(libraries[i].1, &mut b)? {
            if find(&maps, &name, &mut b)?.is_none() {
                set(&mut maps, &name, file, limits.max_materials, &mut b)?;
            }
        }
    }
    for (name, text) in &libraries {
        let mut listed = false;
        for lib in &refs.libraries {
            if base(lib, &mut b)? == *name {
                listed = true;
                break;
            }
        }
        if listed {
            continue;
        }
        for (mat, file) in parse_mtl_inner(text, &mut b)? {
            if find(&maps, &mat, &mut b)?.is_none() {
                set(&mut maps, &mat, file, limits.max_materials, &mut b)?;
            }
        }
    }
    let only = if given.images.len() == 1 {
        Some(0)
    } else {
        None
    };
    let mut by_material = Vec::new();
    for m in &refs.material_names {
        let file = find(&maps, m, &mut b)?.map(|i| maps[i].1.as_str());
        let img = named(file, &images, &mut b)?.or(only);
        if img.is_none()
            && let Some(f) = file.filter(|s| !s.is_empty())
        {
            insert_name(&mut missing, tail(f), &mut b)?;
        }
        push(&mut by_material, img)?;
    }
    let loose = if only.is_some() {
        only
    } else if maps.len() == 1 {
        named(Some(&maps[0].1), &images, &mut b)?
    } else {
        None
    };
    if !by_material.iter().any(Option::is_some) && loose.is_none() {
        if missing.is_empty() {
            for (_, file) in &maps {
                insert_name(&mut missing, tail(file), &mut b)?;
            }
        }
        return Ok(if missing.is_empty() {
            None
        } else {
            Some(BakedTexture {
                colors: None,
                note: TextureNote::Missing(missing),
            })
        });
    }
    let size = refs
        .vertex_uv
        .len()
        .checked_mul(3)
        .ok_or(TextureError::Limit)?;
    let mut colors = Vec::new();
    colors
        .try_reserve_exact(size)
        .map_err(|_| TextureError::Allocation)?;
    colors.resize(size, 180);
    let mut used = Vec::new();
    for (v, &t) in refs.vertex_uv.iter().enumerate() {
        b.spend(1)?;
        if t < 0 {
            continue;
        }
        let m = refs.vertex_material.get(v).copied().unwrap_or(-1);
        let img = if m >= 0 {
            by_material.get(m as usize).copied().flatten()
        } else {
            loose
        };
        let Some(img) = img.map(|i| &given.images[i]) else {
            continue;
        };
        insert_name(&mut used, &img.name, &mut b)?;
        let at = (t as usize).checked_mul(2);
        let u = at
            .and_then(|i| refs.uvs.get(i))
            .copied()
            .unwrap_or(f64::NAN);
        let v1 = at
            .and_then(|i| i.checked_add(1))
            .and_then(|i| refs.uvs.get(i))
            .copied()
            .unwrap_or(f64::NAN);
        sample_texture(img, u, v1, &mut colors, v * 3)?;
    }
    Ok(Some(BakedTexture {
        colors: Some(colors),
        note: TextureNote::Applied(used),
    }))
}
