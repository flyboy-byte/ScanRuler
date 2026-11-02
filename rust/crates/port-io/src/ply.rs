// SPDX-License-Identifier: AGPL-3.0-only
//! Full original PLY dispatcher. Output is owned; graph validation is separate.
pub use crate::ply_ascii::{KEEP_WINDING, PlyError, PlyLimits, PlyMesh};

/// Read ASCII, binary little-endian or binary big-endian PLY with the original
/// header/guard order, scalar semantics, face fans and optional winding marker.
pub fn read_ply(
    bytes: &[u8],
    limits: PlyLimits,
    on_progress: impl FnMut(&str),
) -> Result<PlyMesh, PlyError> {
    let header = crate::ply_ascii::read_header(bytes, limits)?;
    if header.format == "ascii" {
        crate::ply_ascii::ascii_body(bytes, header, limits, on_progress)
    } else {
        crate::ply_binary::binary_body(bytes, header, limits, on_progress)
    }
}
