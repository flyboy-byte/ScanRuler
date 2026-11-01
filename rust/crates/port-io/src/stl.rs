// SPDX-License-Identifier: AGPL-3.0-only
//! Binary triangle decoding from the pinned STL parser's `parseBinary` helper.
//!
//! This is not the `parseSTL` format dispatcher: callers supply the triangle count.
//! Header interpretation, ASCII parsing and mesh welding belong to separate units.
//! Source: `src/core/parsers/stl.ts` at `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

/// Failure before any triangle decoding or progress callbacks occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryStlError {
    /// Requested coordinate bytes are incomplete, or sizes overflow.
    Truncated,
    /// The caller's triangle budget would be exceeded.
    TriangleLimit,
    /// Storage for all output coordinates could not be reserved.
    Allocation,
}

/// Read a triangle soup, skipping each facet normal and two attribute bytes.
///
/// `triangle_count` has already been selected by the format dispatcher. As in the
/// original helper, extra bytes are ignored. Coordinates are not welded, scaled,
/// filtered or checked for finiteness. Output order is triangle, corner, then XYZ.
///
/// The explicit caller budget and fallible allocation bound resource use. A short
/// input returns an error instead of the original DataView range exception. No
/// partially parsed mesh escapes on failure.
pub fn read_binary_triangles(
    bytes: &[u8],
    triangle_count: usize,
    max_triangles: usize,
    mut on_progress: impl FnMut(&str),
) -> Result<Vec<f32>, BinaryStlError> {
    if triangle_count > max_triangles {
        return Err(BinaryStlError::TriangleLimit);
    }
    let end = triangle_count
        .checked_mul(50)
        .and_then(|n| n.checked_add(84))
        .ok_or(BinaryStlError::Truncated)?;
    // The original never reads the final attribute bytes, nor any header bytes
    // when the explicit count is zero. Preserve those successful paths.
    if triangle_count > 0 && bytes.len() < end - 2 {
        return Err(BinaryStlError::Truncated);
    }
    let count = triangle_count
        .checked_mul(9)
        .ok_or(BinaryStlError::Truncated)?;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(count)
        .map_err(|_| BinaryStlError::Allocation)?;
    for t in 0..triangle_count {
        let offset = 96 + t * 50;
        for coordinate in bytes[offset..offset + 36].as_chunks::<4>().0 {
            let mut bits =
                u32::from_le_bytes([coordinate[0], coordinate[1], coordinate[2], coordinate[3]]);
            // DataView.getFloat32 widens to a JS Number before Float32Array stores
            // it again. Quiet signaling NaNs as that conversion does.
            if bits & 0x7f80_0000 == 0x7f80_0000 && bits & 0x007f_ffff != 0 {
                bits |= 0x0040_0000;
            }
            positions.push(f32::from_bits(bits));
        }
        if t & 0x3ffff == 0 && t > 0 {
            let percent = ((t as f64 / triangle_count as f64) * 100.0 + 0.5).floor();
            on_progress(&format!("Reading STL… {percent:.0}%"));
        }
    }
    Ok(positions)
}
