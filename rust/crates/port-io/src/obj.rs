// SPDX-License-Identifier: AGPL-3.0-only
//! Complete pinned parseOBJ body, including decoded companion texture continuation.
use crate::obj_texture::{
    ObjCompanions, ObjTextureRefs, TextureError, TextureLimits, TextureNote, bake_obj_texture,
    mtllib_names,
};
use crate::stl_dispatch::{decode_utf8, js_space};
use std::collections::HashMap;

/// Explicit parser budgets; texture lookup/baking has its own bounded phase.
#[derive(Debug, Clone, Copy)]
pub struct ObjLimits {
    /// Input file bytes.
    pub max_bytes: usize,
    /// Vertex declarations.
    pub max_vertices: usize,
    /// Texture coordinate pairs.
    pub max_uvs: usize,
    /// Output fan indices.
    pub max_indices: usize,
    /// Corners in one temporary face.
    pub max_face_vertices: usize,
    /// Unique encountered material names.
    pub max_materials: usize,
    /// Declared library names including duplicates.
    pub max_libraries: usize,
    /// Parser byte/lookup visits.
    pub max_work: usize,
    /// Independent decoded-companion/texture-phase caps.
    pub texture: TextureLimits,
}
/// Original source refusals versus separate safe host resource failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjError {
    /// Invalid finite XYZ record, numbered as in the source text.
    InvalidVertex(usize),
    /// Invalid decimal/safe/relative face index.
    InvalidFaceIndex(usize),
    /// No vertex declarations survived parsing.
    NoVertices,
    /// No complete triangle fan.
    NoFaces,
    /// Explicit caller resource budget exceeded.
    Limit,
    /// Storage reservation failed.
    Allocation,
    /// Explicit host texture-phase refusal, not a fabricated source missing note.
    Texture(TextureError),
}
impl std::fmt::Display for ObjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVertex(n) => write!(f, "Invalid OBJ vertex on line {n}."),
            Self::InvalidFaceIndex(n) => write!(f, "Invalid OBJ face index on line {n}."),
            Self::NoVertices => f.write_str("OBJ file has no vertices."),
            Self::NoFaces => {
                f.write_str("This OBJ contains no faces — point clouds are not supported yet.")
            }
            Self::Limit => f.write_str("OBJ input exceeds the caller's resource budget."),
            Self::Allocation => f.write_str("Could not allocate OBJ storage."),
            Self::Texture(e) => write!(f, "OBJ texture host refusal: {e:?}"),
        }
    }
}
/// Owned source indexed result; normals/groups/winding are not fabricated.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjMesh {
    /// XYZ narrowed from source Numbers only after parsing.
    pub positions: Vec<f32>,
    /// Fan indices, without downstream graph validation or repair.
    pub indices: Vec<u32>,
    /// Absent or the source per-vertex RGB array.
    pub colors: Option<Vec<u8>>,
    /// Absent, missing or applied with exact ordered original names.
    pub texture: Option<TextureNote>,
}
fn push<T>(v: &mut Vec<T>, x: T) -> Result<(), ObjError> {
    v.try_reserve(1).map_err(|_| ObjError::Allocation)?;
    v.push(x);
    Ok(())
}
fn own(s: &str) -> Result<String, ObjError> {
    let mut out = String::new();
    out.try_reserve_exact(s.len())
        .map_err(|_| ObjError::Allocation)?;
    out.push_str(s);
    Ok(out)
}
fn spend(left: &mut usize, n: usize) -> Result<(), ObjError> {
    *left = left.checked_sub(n).ok_or(ObjError::Limit)?;
    Ok(())
}
fn words(s: &str) -> Result<Vec<&str>, ObjError> {
    let mut out = Vec::new();
    for t in s.split(js_space).filter(|t| !t.is_empty()) {
        push(&mut out, t)?;
    }
    Ok(out)
}
fn safe_integer(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0
}
fn decimal_index(s: &str) -> bool {
    let rest = s.strip_prefix('-').unwrap_or(s);
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}
// Complete String Number grammar used by OBJ, not STL's numeric-prefix parsing.
pub(crate) fn number(s: &str) -> f64 {
    let s = s.trim_matches(js_space);
    if s.is_empty() {
        return 0.0;
    }
    if let Some(radix) = match s.get(..2) {
        Some("0x" | "0X") => Some(4),
        Some("0o" | "0O") => Some(3),
        Some("0b" | "0B") => Some(1),
        _ => None,
    } {
        let mut significand = 0u64;
        let mut bits = 0usize;
        let mut guard = false;
        let mut sticky = false;
        if s.len() == 2 {
            return f64::NAN;
        }
        for c in s[2..].bytes() {
            let digit = match c {
                b'0'..=b'9' => u32::from(c - b'0'),
                b'a'..=b'f' => u32::from(c - b'a' + 10),
                b'A'..=b'F' => u32::from(c - b'A' + 10),
                _ => return f64::NAN,
            };
            if digit >= 1 << radix {
                return f64::NAN;
            }
            for shift in (0..radix).rev() {
                let one = digit & (1 << shift) != 0;
                if bits == 0 && !one {
                    continue;
                }
                bits += 1;
                if bits <= 53 {
                    significand = (significand << 1) | u64::from(one);
                } else if bits == 54 {
                    guard = one;
                } else {
                    sticky |= one;
                }
            }
        }
        if bits <= 53 {
            return significand as f64;
        }
        if bits > 1024 {
            return f64::INFINITY;
        }
        if guard && (sticky || significand & 1 != 0) {
            significand += 1;
        }
        return significand as f64 * f64::from_bits(((1023 + bits - 53) as u64) << 52);
    }
    if s == "Infinity" || s == "+Infinity" {
        return f64::INFINITY;
    }
    if s == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    let bytes = s.as_bytes();
    let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
        digits += 1;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return f64::NAN;
        }
    }
    if i != bytes.len() {
        return f64::NAN;
    }
    s.parse().unwrap_or(f64::NAN)
}

/// Entire original geometry/UV/material body and successful bake continuation.
/// Invalid indices/float32 overflow accepted by the source parser are retained;
/// downstream graph validation remains separate. No partial mesh escapes errors.
pub fn read_obj(
    bytes: &[u8],
    limits: ObjLimits,
    mut on_progress: impl FnMut(&str),
    companions: Option<&ObjCompanions>,
) -> Result<ObjMesh, ObjError> {
    if bytes.len() > limits.max_bytes {
        return Err(ObjError::Limit);
    }
    on_progress("Reading OBJ…");
    let text = decode_utf8(bytes).map_err(|_| ObjError::Allocation)?;
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();
    let mut corners: HashMap<u32, (i32, i32)> = HashMap::new();
    let mut material_names = Vec::new();
    let mut materials: HashMap<String, i32> = HashMap::new();
    let mut libraries = Vec::new();
    let mut material = -1;
    let mut left = limits.max_work;
    let line_count = text.split('\n').count();
    for (li, raw) in text.split('\n').enumerate() {
        spend(&mut left, raw.len().saturating_add(1))?;
        let line = raw.split('#').next().unwrap_or("").trim_matches(js_space);
        if line.encode_utf16().take(3).count() < 3 {
            continue;
        }
        let c = line.as_bytes();
        let sep = matches!(c.get(1), Some(b' ' | b'\t'));
        if c.first() == Some(&b'v') && sep {
            let parts = words(line)?;
            if parts.len() < 4 {
                return Err(ObjError::InvalidVertex(li + 1));
            }
            let xyz = [number(parts[1]), number(parts[2]), number(parts[3])];
            if !xyz.iter().all(|x| x.is_finite()) {
                return Err(ObjError::InvalidVertex(li + 1));
            }
            if coords.len() / 3 >= limits.max_vertices || coords.len() / 3 >= u32::MAX as usize {
                return Err(ObjError::Limit);
            }
            coords.try_reserve(3).map_err(|_| ObjError::Allocation)?;
            coords.extend(xyz);
        } else if c.starts_with(b"vt") && matches!(c.get(2), Some(b' ' | b'\t')) {
            let parts = words(line)?;
            let u = number(parts.get(1).copied().unwrap_or("undefined"));
            let v = parts.get(2).map_or(0.0, |s| number(s));
            if uvs.len() / 2 >= limits.max_uvs || uvs.len() / 2 >= i32::MAX as usize {
                return Err(ObjError::Limit);
            }
            uvs.try_reserve(2).map_err(|_| ObjError::Allocation)?;
            uvs.push(if u.is_finite() { u } else { 0.0 });
            uvs.push(if v.is_finite() { v } else { 0.0 });
        } else if c.first() == Some(&b'f') && sep {
            let parts = words(line)?;
            let vcount = (coords.len() / 3) as f64;
            let tcount = (uvs.len() / 2) as f64;
            let mut face = Vec::new();
            for part in parts.iter().skip(1) {
                let mut fields = part.split('/');
                let token = fields.next().unwrap_or("");
                let idx = number(token);
                if !decimal_index(token)
                    || !safe_integer(idx)
                    || idx == 0.0
                    || idx > 4_294_967_295.0
                    || idx < -vcount
                {
                    return Err(ObjError::InvalidFaceIndex(li + 1));
                }
                let vi = if idx < 0.0 { vcount + idx } else { idx - 1.0 } as u32;
                if face.len() >= limits.max_face_vertices {
                    return Err(ObjError::Limit);
                }
                push(&mut face, vi)?;
                if let Some(t) = fields.next().filter(|s| !s.is_empty())
                    && !corners.contains_key(&vi)
                {
                    let t = number(t);
                    let ti = if t < 0.0 { tcount + t } else { t - 1.0 };
                    if safe_integer(t) && ti >= 0.0 && ti < tcount {
                        corners.try_reserve(1).map_err(|_| ObjError::Allocation)?;
                        corners.insert(vi, (ti as i32, material));
                    }
                }
            }
            for k in 2..face.len() {
                if indices
                    .len()
                    .checked_add(3)
                    .is_none_or(|n| n > limits.max_indices)
                {
                    return Err(ObjError::Limit);
                }
                indices.try_reserve(3).map_err(|_| ObjError::Allocation)?;
                indices.extend([face[0], face[k - 1], face[k]]);
            }
        } else if let Some(name) = line.strip_prefix("usemtl") {
            let name = name.trim_matches(js_space);
            if name.len() > limits.texture.max_name_bytes {
                return Err(ObjError::Limit);
            }
            if let Some(&m) = materials.get(name) {
                material = m;
            } else {
                if material_names.len() >= limits.max_materials
                    || material_names.len() >= i32::MAX as usize
                {
                    return Err(ObjError::Limit);
                }
                material = material_names.len() as i32;
                materials.try_reserve(1).map_err(|_| ObjError::Allocation)?;
                materials.insert(own(name)?, material);
                push(&mut material_names, own(name)?)?;
            }
        } else if let Some(rest) = line.strip_prefix("mtllib") {
            for name in mtllib_names(rest, limits.texture).map_err(ObjError::Texture)? {
                if libraries.len() >= limits.max_libraries {
                    return Err(ObjError::Limit);
                }
                push(&mut libraries, name)?;
            }
        }
        if li & 0xfffff == 0 && li > 0 {
            let p = li as f64 / line_count as f64 * 100.0;
            let lower = p.floor();
            let rounded = if p - lower < 0.5 { lower } else { lower + 1.0 };
            on_progress(&format!("Reading OBJ… {rounded:.0}%"));
        }
    }
    if coords.is_empty() {
        return Err(ObjError::NoVertices);
    }
    if indices.is_empty() {
        return Err(ObjError::NoFaces);
    }
    let n = coords.len() / 3;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(coords.len())
        .map_err(|_| ObjError::Allocation)?;
    positions.extend(coords.into_iter().map(|x| x as f32));
    let mut mesh = ObjMesh {
        positions,
        indices,
        colors: None,
        texture: None,
    };
    if !uvs.is_empty() {
        let mut vertex_uv = Vec::new();
        vertex_uv
            .try_reserve_exact(n)
            .map_err(|_| ObjError::Allocation)?;
        vertex_uv.resize(n, -1);
        let mut vertex_material = Vec::new();
        vertex_material
            .try_reserve_exact(n)
            .map_err(|_| ObjError::Allocation)?;
        vertex_material.resize(n, -1);
        for v in 0..n {
            if let Some(&(t, m)) = corners.get(&(v as u32)) {
                vertex_uv[v] = t;
                vertex_material[v] = m;
            }
        }
        if companions.is_some_and(|c| !c.images.is_empty()) {
            on_progress("Colouring the scan from its texture…");
        }
        let refs = ObjTextureRefs {
            uvs,
            vertex_uv,
            vertex_material,
            material_names,
            libraries,
        };
        if let Some(baked) =
            bake_obj_texture(&refs, companions, limits.texture).map_err(ObjError::Texture)?
        {
            mesh.colors = baked.colors;
            mesh.texture = Some(baked.note);
        }
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_number_rules_do_not_become_numeric_prefix_rules() {
        assert_eq!(number("1.e2"), 100.0);
        assert_eq!(number("0x20000000000001"), 9_007_199_254_740_992.0);
        assert_eq!(number("0x20000000000003"), 9_007_199_254_740_996.0);
        assert_eq!(number("0b101"), 5.0);
        assert_eq!(number("0o17"), 15.0);
        assert!(number("1tail").is_nan());
        assert!(number("-0x1").is_nan());
        assert!(number("0b2").is_nan());
        assert!(number("1e+").is_nan());
        assert_eq!(number("-0").to_bits(), (-0.0f64).to_bits());
    }
}
