// SPDX-License-Identifier: AGPL-3.0-only
//! Full source binary body, including its two-pass face and malformed-offset semantics.
use crate::ply_ascii::{Header, PlyError, PlyLimits, PlyMesh, append, own, uint32, work};

fn type_size(t: &str) -> f64 {
    match t {
        "char" | "int8" | "uchar" | "uint8" => 1.0,
        "short" | "int16" | "ushort" | "uint16" => 2.0,
        "int" | "int32" | "uint" | "uint32" | "float" | "float32" => 4.0,
        "double" | "float64" => 8.0,
        _ => f64::NAN, // JS TYPE_SIZE[unknown] is undefined, not a premature refusal.
    }
}

fn read_scalar(bytes: &[u8], off: f64, t: &str, little: bool) -> Result<f64, PlyError> {
    let size = type_size(t);
    if size.is_nan() {
        return Err(PlyError::UnsupportedType(own(t)?));
    }
    // DataView ToIndex: NaN -> 0, fractions truncate, negative integers refuse.
    let offset = if off.is_nan() { 0.0 } else { off.trunc() };
    if !offset.is_finite() || offset < 0.0 || offset > bytes.len() as f64 {
        return Err(PlyError::Truncated);
    }
    let offset = offset as usize;
    let end = offset
        .checked_add(size as usize)
        .ok_or(PlyError::Truncated)?;
    let raw = bytes.get(offset..end).ok_or(PlyError::Truncated)?;
    let mut bits = 0u64;
    if little {
        for (i, b) in raw.iter().enumerate() {
            bits |= u64::from(*b) << (8 * i);
        }
    } else {
        for b in raw {
            bits = (bits << 8) | u64::from(*b);
        }
    }
    Ok(match t {
        "char" | "int8" => f64::from(bits as i8),
        "uchar" | "uint8" => f64::from(bits as u8),
        "short" | "int16" => f64::from(bits as i16),
        "ushort" | "uint16" => f64::from(bits as u16),
        "int" | "int32" => f64::from(bits as i32),
        "uint" | "uint32" => f64::from(bits as u32),
        "float" | "float32" => {
            let mut b = bits as u32;
            // The source widens DataView float32 to Number before storing it.
            if b & 0x7f80_0000 == 0x7f80_0000 && b & 0x007f_ffff != 0 {
                b |= 0x0040_0000;
            }
            f64::from(f32::from_bits(b))
        }
        "double" | "float64" => f64::from_bits(bits),
        _ => return Err(PlyError::UnsupportedType(own(t)?)),
    })
}

fn zeros<T: Default + Clone>(n: usize) -> Result<Vec<T>, PlyError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| PlyError::Allocation)?;
    v.resize(n, T::default());
    Ok(v)
}
fn loops(n: f64, max: usize) -> Result<usize, PlyError> {
    if n > max as f64 {
        return Err(PlyError::ResourceLimit);
    }
    Ok(n as usize)
}
fn vertex_count(n: f64, limits: PlyLimits) -> Result<usize, PlyError> {
    if n < 0.0 {
        return Err(PlyError::InvalidArrayLength(format!("{}", n * 3.0)));
    }
    loops(n, limits.max_vertices)
}

pub(crate) fn binary_body(
    bytes: &[u8],
    header: Header,
    limits: PlyLimits,
    mut on_progress: impl FnMut(&str),
) -> Result<PlyMesh, PlyError> {
    let Header {
        body_start,
        format,
        keep_winding,
        elements,
        mut left,
    } = header;
    let little = format == "binary_little_endian";
    let mut off = body_start as f64;
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
            let count = vertex_count(el.count, limits)?;
            let mut out = zeros::<f32>(count.checked_mul(3).ok_or(PlyError::ResourceLimit)?)?;
            if !el.props.iter().any(|p| p.list) {
                let mut stride = 0.0;
                let mut offsets = Vec::new();
                for p in &el.props {
                    work(&mut left)?;
                    append(&mut offsets, stride)?;
                    stride += type_size(&p.scalar_type);
                }
                for i in 0..count {
                    work(&mut left)?;
                    let base = off + i as f64 * stride;
                    out[i * 3] =
                        read_scalar(bytes, base + offsets[xi], &el.props[xi].scalar_type, little)?
                            as f32;
                    out[i * 3 + 1] =
                        read_scalar(bytes, base + offsets[yi], &el.props[yi].scalar_type, little)?
                            as f32;
                    out[i * 3 + 2] =
                        read_scalar(bytes, base + offsets[zi], &el.props[zi].scalar_type, little)?
                            as f32;
                    if i & 0xfffff == 0 && i > 0 {
                        let percent = (i as f64 / el.count * 100.0 + 0.5).floor();
                        on_progress(&format!("Reading PLY vertices… {percent:.0}%"));
                    }
                }
                off += el.count * stride;
            } else {
                for i in 0..count {
                    work(&mut left)?;
                    for (p, prop) in el.props.iter().enumerate() {
                        work(&mut left)?;
                        if prop.list {
                            let n = read_scalar(bytes, off, &prop.count_type, little)?;
                            off += type_size(&prop.count_type) + n * type_size(&prop.item_type);
                        } else {
                            let v = read_scalar(bytes, off, &prop.scalar_type, little)?;
                            if p == xi {
                                out[i * 3] = v as f32;
                            } else if p == yi {
                                out[i * 3 + 1] = v as f32;
                            } else if p == zi {
                                out[i * 3 + 2] = v as f32;
                            }
                            off += type_size(&prop.scalar_type);
                        }
                    }
                }
            }
            positions = Some(out);
        } else if el.name == "face" {
            let count = loops(el.count, limits.max_work)?;
            let start = off;
            let mut tri_total = 0.0;
            for _ in 0..count {
                work(&mut left)?;
                for prop in &el.props {
                    work(&mut left)?;
                    if prop.list {
                        let n = read_scalar(bytes, off, &prop.count_type, little)?;
                        off += type_size(&prop.count_type) + n * type_size(&prop.item_type);
                        if matches!(prop.name.as_str(), "vertex_indices" | "vertex_index")
                            && n >= 3.0
                        {
                            tri_total += n - 2.0;
                        }
                    } else {
                        off += type_size(&prop.scalar_type);
                    }
                }
            }
            if !tri_total.is_finite() || tri_total > limits.max_triangles as f64 {
                return Err(PlyError::ResourceLimit);
            }
            let scalars = tri_total * 3.0;
            if scalars >= usize::MAX as f64 {
                return Err(PlyError::ResourceLimit);
            }
            indices = zeros::<u32>(scalars as usize)?;
            off = start;
            let mut w = 0usize;
            for i in 0..count {
                work(&mut left)?;
                for prop in &el.props {
                    work(&mut left)?;
                    if prop.list {
                        let n = read_scalar(bytes, off, &prop.count_type, little)?;
                        off += type_size(&prop.count_type);
                        let item_size = type_size(&prop.item_type);
                        if matches!(prop.name.as_str(), "vertex_indices" | "vertex_index") {
                            // Source reads both even if n < 2; do not short-circuit.
                            let first = read_scalar(bytes, off, &prop.item_type, little)?;
                            let mut previous =
                                read_scalar(bytes, off + item_size, &prop.item_type, little)?;
                            let mut k = 2.0;
                            while k < n {
                                work(&mut left)?;
                                let current = read_scalar(
                                    bytes,
                                    off + k * item_size,
                                    &prop.item_type,
                                    little,
                                )?;
                                for v in [first, previous, current] {
                                    // Fractional face counts can size a partial last
                                    // triangle; typed-array out-of-range writes vanish.
                                    if let Some(slot) = indices.get_mut(w) {
                                        *slot = uint32(v);
                                    }
                                    w = w.checked_add(1).ok_or(PlyError::ResourceLimit)?;
                                }
                                previous = current;
                                k += 1.0;
                            }
                        }
                        off += n * item_size;
                    } else {
                        off += type_size(&prop.scalar_type);
                    }
                }
                if i & 0xfffff == 0 && i > 0 {
                    let percent = (i as f64 / el.count * 100.0 + 0.5).floor();
                    on_progress(&format!("Reading PLY faces… {percent:.0}%"));
                }
            }
        } else {
            let mut stride = 0.0;
            for p in &el.props {
                work(&mut left)?;
                if p.list {
                    return Err(PlyError::CannotSkipElement(own(&el.name)?));
                }
                stride += type_size(&p.scalar_type);
            }
            off += el.count * stride;
        }
    }
    Ok(PlyMesh {
        positions: positions.ok_or(PlyError::NoVertices)?,
        indices,
        keep_winding,
    })
}
