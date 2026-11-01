// SPDX-License-Identifier: AGPL-3.0-only
//! Exact-bit welding from frozen `src/core/geometry/weld.ts` (2cd89c6).

/// Bounded input/allocation failure; no partial mesh escapes.
#[derive(Debug, PartialEq, Eq)]
pub enum WeldError {
    /// Coordinates or indices do not form complete triples.
    IncompleteTriple,
    /// Caller budget or original safe table capacity exceeded.
    VertexLimit,
    /// Storage reservation failed.
    Allocation,
}

/// Owned indexed mesh, preserving first-seen vertex order.
#[derive(Debug)]
pub struct WeldedMesh {
    /// Packed XYZ coordinates.
    pub positions: Vec<f32>,
    /// Vertex identities in input corner order.
    pub indices: Vec<u32>,
}

fn hash3(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(0x9e3779b1);
    h ^= b;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h ^= c;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^ (h >> 16)
}

fn storage<T>(capacity: usize) -> Result<Vec<T>, WeldError> {
    let mut out = Vec::new();
    out.try_reserve_exact(capacity)
        .map_err(|_| WeldError::Allocation)?;
    Ok(out)
}

/// Weld by original float bit identities, with original hash/probe and progress order.
/// Signed zeros are distinct at the frozen baseline. Callers must supply complete
/// XYZ triples; non-finite coordinates are retained. Explicit limits prevent the
/// original signed table-capacity shift from overflowing on oversized inputs.
pub fn weld_triangle_soup(
    soup: &[f32],
    max_vertices: usize,
    mut progress: impl FnMut(&str),
) -> Result<WeldedMesh, WeldError> {
    if !soup.len().is_multiple_of(3) {
        return Err(WeldError::IncompleteTriple);
    }
    let count = soup.len() / 3;
    if count > max_vertices || count > (1 << 29) {
        return Err(WeldError::VertexLimit);
    }
    let mut cap = 16;
    while cap < count * 2 {
        cap *= 2;
    }
    let mut table = storage(cap)?;
    table.resize(cap, usize::MAX);
    let mut positions: Vec<f32> = storage(soup.len())?;
    let mut indices = storage(count)?;
    for (i, xyz) in soup.as_chunks::<3>().0.iter().enumerate() {
        let bits = xyz.map(f32::to_bits);
        let mut h = hash3(bits[0], bits[1], bits[2]) as usize & (cap - 1);
        loop {
            let slot = table[h];
            if slot == usize::MAX {
                let out_count = positions.len() / 3;
                table[h] = out_count;
                for mut b in bits {
                    // JS Number assignment quiets signaling binary32 NaNs.
                    if b & 0x7f800000 == 0x7f800000 && b & 0x007fffff != 0 {
                        b |= 0x00400000;
                    }
                    positions.push(f32::from_bits(b));
                }
                indices.push(out_count as u32);
                break;
            }
            if positions[slot * 3..slot * 3 + 3]
                .iter()
                .map(|v| v.to_bits())
                .eq(bits)
            {
                indices.push(slot as u32);
                break;
            }
            h = (h + 1) & (cap - 1);
        }
        if i & 0x1fffff == 0 && i > 0 {
            let percent = ((i as f64 / count as f64) * 100.0 + 0.5).floor();
            progress(&format!("Merging vertices… {percent:.0}%"));
        }
    }
    Ok(WeldedMesh { positions, indices })
}

/// Remove triangles with repeated vertex indices, preserving all other indices.
/// This does not perform a geometric area test.
pub fn filter_degenerate_triangles(indices: &[u32]) -> Result<Vec<u32>, WeldError> {
    if !indices.len().is_multiple_of(3) {
        return Err(WeldError::IncompleteTriple);
    }
    let mut out = storage(indices.len())?;
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        if a == b || b == c || a == c {
            continue;
        }
        out.extend_from_slice(&[a, b, c]);
    }
    Ok(out)
}
