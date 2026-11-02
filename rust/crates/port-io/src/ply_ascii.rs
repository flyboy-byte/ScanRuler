// SPDX-License-Identifier: AGPL-3.0-only
//! Complete ASCII PLY body and shared header from the pinned parser. Use
//! `crate::ply::read_ply` for all encodings. Mesh graph/session validation is separate.
use crate::stl_dispatch::{decode_utf8, js_space, parse_float_prefix};

/// Exact source comment declaring that winding must be preserved.
pub const KEEP_WINDING: &str = "ScanRuler: keep winding";

/// Source refusals and separately identified host/missing-port boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlyError {
    /// Invalid raw signature or absent end marker within the source header limit.
    InvalidFile,
    /// No vertex element in the header.
    NoVertex,
    /// No face element, or its first declaration has zero faces.
    NoFaces,
    /// Vertex properties lack at least one required coordinate name.
    MissingCoordinates,
    /// Encoding not recognized by the source.
    UnsupportedFormat(String),
    /// Source typed-array allocation would receive a negative length.
    InvalidArrayLength(String),
    /// ASCII-only entrypoint reached a binary encoding; use `ply::read_ply`.
    BinaryNotPorted,
    /// Binary scalar type not supported by the original.
    UnsupportedType(String),
    /// Original cannot skip unknown elements containing lists.
    CannotSkipElement(String),
    /// Original DataView scalar read is outside the file.
    Truncated,
    /// Body contained no vertex allocation.
    NoVertices,
    /// Caller byte, vertex, triangle or work budget exceeded.
    ResourceLimit,
    /// Fallible storage reservation failed.
    Allocation,
}

impl std::fmt::Display for PlyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFile => f.write_str("Not a valid PLY file."),
            Self::NoVertex => f.write_str("PLY file has no vertex element."),
            Self::NoFaces => {
                f.write_str("This PLY contains no faces — point clouds are not supported yet.")
            }
            Self::MissingCoordinates => f.write_str("PLY vertex element is missing x/y/z."),
            Self::UnsupportedFormat(s) => write!(f, "Unsupported PLY format \"{s}\"."),
            Self::InvalidArrayLength(s) => write!(f, "Invalid typed array length: {s}"),
            Self::BinaryNotPorted => {
                f.write_str("Binary PLY requires the full read_ply dispatcher.")
            }
            Self::UnsupportedType(s) => write!(f, "Unsupported PLY type \"{s}\"."),
            Self::CannotSkipElement(s) => {
                write!(f, "Cannot skip PLY element \"{s}\" with list properties.")
            }
            Self::Truncated => f.write_str("Offset is outside the bounds of the DataView"),
            Self::NoVertices => f.write_str("PLY file has no vertices."),
            Self::ResourceLimit => f.write_str("PLY input exceeds the caller's resource budget."),
            Self::Allocation => f.write_str("Could not allocate PLY storage."),
        }
    }
}

/// Explicit host budgets, distinct from source numerical/file semantics.
#[derive(Debug, Clone, Copy)]
pub struct PlyLimits {
    /// Maximum input byte length.
    pub max_bytes: usize,
    /// Maximum declared vertices in one vertex element.
    pub max_vertices: usize,
    /// Maximum output triangle count.
    pub max_triangles: usize,
    /// Property visits plus generated triangles. Bounds headers that repeat work
    /// without consuming tokens (including zero-property elements).
    pub max_work: usize,
}

/// Owned indexed output; ignored source normals/colors remain absent.
#[derive(Debug, Clone, PartialEq)]
pub struct PlyMesh {
    /// Float32 XYZ coordinates in source property/element order.
    pub positions: Vec<f32>,
    /// Polygon fans with source Uint32 conversion; no index validation here.
    pub indices: Vec<u32>,
    /// False represents the original's absent optional field, not an explicit
    /// `keepWinding: false` in the public JSON schema.
    pub keep_winding: bool,
}

pub(crate) struct Property {
    pub(crate) name: String,
    pub(crate) list: bool,
    pub(crate) scalar_type: String,
    pub(crate) count_type: String,
    pub(crate) item_type: String,
}
pub(crate) struct Element {
    pub(crate) name: String,
    pub(crate) count: f64,
    pub(crate) props: Vec<Property>,
}
pub(crate) struct Header {
    pub(crate) body_start: usize,
    pub(crate) format: String,
    pub(crate) keep_winding: bool,
    pub(crate) elements: Vec<Element>,
    pub(crate) left: usize,
}

pub(crate) fn own(s: &str) -> Result<String, PlyError> {
    let mut out = String::new();
    out.try_reserve_exact(s.len())
        .map_err(|_| PlyError::Allocation)?;
    out.push_str(s);
    Ok(out)
}
pub(crate) fn append<T>(v: &mut Vec<T>, value: T) -> Result<(), PlyError> {
    v.try_reserve(1).map_err(|_| PlyError::Allocation)?;
    v.push(value);
    Ok(())
}
fn decode(bytes: &[u8]) -> Result<String, PlyError> {
    decode_utf8(bytes).map_err(|_| PlyError::Allocation)
}
fn words(s: &str) -> Result<Vec<&str>, PlyError> {
    let mut out = Vec::new();
    for t in s.split(js_space).filter(|s| !s.is_empty()) {
        append(&mut out, t)?;
    }
    Ok(out)
}
// parseInt(..., 10), including a consumed decimal prefix and negative zero.
fn integer(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if end == start {
        f64::NAN
    } else {
        s[..end].parse().unwrap_or(f64::NAN)
    }
}
fn token<'a>(tokens: &[&'a str], cursor: f64) -> &'a str {
    if cursor.is_finite() && cursor >= 0.0 && cursor.fract() == 0.0 {
        tokens.get(cursor as usize).copied().unwrap_or("undefined")
    } else {
        "undefined"
    }
}
pub(crate) fn uint32(v: f64) -> u32 {
    if v.is_finite() {
        v.trunc().rem_euclid(4_294_967_296.0) as u32
    } else {
        0
    }
}
pub(crate) fn work(left: &mut usize) -> Result<(), PlyError> {
    *left = left.checked_sub(1).ok_or(PlyError::ResourceLimit)?;
    Ok(())
}

/// Parse the complete original ASCII body and its shared header.
///
/// Numeric prefixes, nonfinite/missing coordinates, invalid indices and repeated
/// declarations preserve parser output; downstream graph validation is separate.
/// Binary is refused by this ASCII-only entrypoint; use `ply::read_ply` instead.
/// No partial mesh escapes an error.
pub fn read_ply_ascii(
    bytes: &[u8],
    limits: PlyLimits,
    on_progress: impl FnMut(&str),
) -> Result<PlyMesh, PlyError> {
    let header = read_header(bytes, limits)?;
    if header.format != "ascii" {
        return Err(PlyError::BinaryNotPorted);
    }
    ascii_body(bytes, header, limits, on_progress)
}

pub(crate) fn read_header(bytes: &[u8], limits: PlyLimits) -> Result<Header, PlyError> {
    if bytes.len() > limits.max_bytes {
        return Err(PlyError::ResourceLimit);
    }
    let header_limit = bytes.len().min(64 * 1024);
    let end = bytes[..header_limit]
        .windows(10)
        .position(|s| s == b"end_header");
    if !bytes.starts_with(b"ply") || end.is_none() {
        return Err(PlyError::InvalidFile);
    }
    let end = end.ok_or(PlyError::InvalidFile)?;
    let body_start = bytes[end..header_limit]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(header_limit + 1, |n| end + n + 1);
    let header = decode(&bytes[..end])?;
    let mut format = own("")?;
    let mut keep_winding = false;
    let mut elements: Vec<Element> = Vec::new();
    let mut left = limits.max_work;
    for line in header.split('\n') {
        work(&mut left)?;
        let parts = words(line.trim_matches(js_space))?;
        let part = |i| parts.get(i).copied().unwrap_or("undefined");
        match part(0) {
            "format" => format = own(part(1))?,
            "comment" => {
                // Compare the whitespace-normalized words without allocating a join.
                if parts
                    .get(1..)
                    .is_some_and(|p| p.iter().copied().eq(KEEP_WINDING.split(' ')))
                {
                    keep_winding = true;
                }
            }
            "element" => append(
                &mut elements,
                Element {
                    name: own(part(1))?,
                    count: integer(part(2)),
                    props: Vec::new(),
                },
            )?,
            "property" => {
                if let Some(el) = elements.last_mut() {
                    let list = part(1) == "list";
                    append(
                        &mut el.props,
                        Property {
                            name: own(part(if list { 4 } else { 2 }))?,
                            list,
                            scalar_type: own(part(1))?,
                            count_type: own(part(2))?,
                            item_type: own(part(3))?,
                        },
                    )?;
                }
            }
            _ => {}
        }
    }
    if !elements.iter().any(|e| e.name == "vertex") {
        return Err(PlyError::NoVertex);
    }
    if elements
        .iter()
        .find(|e| e.name == "face")
        .is_none_or(|e| e.count == 0.0)
    {
        return Err(PlyError::NoFaces);
    }
    if !matches!(
        format.as_str(),
        "ascii" | "binary_little_endian" | "binary_big_endian"
    ) {
        return Err(PlyError::UnsupportedFormat(format));
    }
    Ok(Header {
        body_start,
        format,
        keep_winding,
        elements,
        left,
    })
}

pub(crate) fn ascii_body(
    bytes: &[u8],
    header: Header,
    limits: PlyLimits,
    mut on_progress: impl FnMut(&str),
) -> Result<PlyMesh, PlyError> {
    let Header {
        body_start,
        keep_winding,
        elements,
        mut left,
        ..
    } = header;
    on_progress("Reading PLY…");
    let body = decode(&bytes[body_start.min(bytes.len())..])?;
    let tokens = words(&body)?;
    let mut ti = 0.0;
    let mut positions = None;
    let mut indices = Vec::new();
    for el in elements {
        work(&mut left)?;
        if el.name == "vertex" {
            let xi = el
                .props
                .iter()
                .position(|p| p.name == "x")
                .ok_or(PlyError::MissingCoordinates)?;
            let yi = el
                .props
                .iter()
                .position(|p| p.name == "y")
                .ok_or(PlyError::MissingCoordinates)?;
            let zi = el
                .props
                .iter()
                .position(|p| p.name == "z")
                .ok_or(PlyError::MissingCoordinates)?;
            if el.count < 0.0 {
                return Err(PlyError::InvalidArrayLength(format!("{}", el.count * 3.0)));
            }
            if el.count > limits.max_vertices as f64 {
                return Err(PlyError::ResourceLimit);
            }
            let n = el.count as usize; // JS ToIndex(NaN) is zero.
            let len = n.checked_mul(3).ok_or(PlyError::ResourceLimit)?;
            let mut out = Vec::new();
            out.try_reserve_exact(len)
                .map_err(|_| PlyError::Allocation)?;
            out.resize(len, 0.0);
            for i in 0..n {
                work(&mut left)?;
                for (p, prop) in el.props.iter().enumerate() {
                    work(&mut left)?;
                    if prop.list {
                        let n = integer(token(&tokens, ti));
                        ti += 1.0;
                        ti += n;
                    } else {
                        let v = parse_float_prefix(token(&tokens, ti));
                        ti += 1.0;
                        if p == xi {
                            out[i * 3] = v as f32;
                        } else if p == yi {
                            out[i * 3 + 1] = v as f32;
                        } else if p == zi {
                            out[i * 3 + 2] = v as f32;
                        }
                    }
                }
            }
            positions = Some(out);
        } else {
            if el.count > limits.max_work as f64 {
                return Err(PlyError::ResourceLimit);
            }
            let count = el.count as usize; // Negative/NaN loops have zero iterations.
            for _ in 0..count {
                work(&mut left)?;
                for prop in &el.props {
                    work(&mut left)?;
                    if prop.list {
                        let n = integer(token(&tokens, ti));
                        ti += 1.0;
                        if el.name == "face"
                            && matches!(prop.name.as_str(), "vertex_indices" | "vertex_index")
                        {
                            let first = integer(token(&tokens, ti));
                            let mut k = 2.0;
                            while k < n {
                                work(&mut left)?;
                                if indices.len() / 3 >= limits.max_triangles {
                                    return Err(PlyError::ResourceLimit);
                                }
                                let previous = integer(token(&tokens, ti + k - 1.0));
                                let current = integer(token(&tokens, ti + k));
                                for v in [first, previous, current] {
                                    append(&mut indices, uint32(v))?;
                                }
                                k += 1.0;
                            }
                        }
                        ti += n;
                    } else {
                        ti += 1.0;
                    }
                }
            }
        }
    }
    Ok(PlyMesh {
        positions: positions.ok_or(PlyError::NoVertices)?,
        indices,
        keep_winding,
    })
}
