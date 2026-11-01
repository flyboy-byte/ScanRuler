// SPDX-License-Identifier: AGPL-3.0-only
//! Area-weighted normals, from pinned normals.ts::computeVertexNormals (2cd89c6).

/// Invalid mesh contracts or bounded storage failures.
#[derive(Debug, PartialEq, Eq)]
pub enum NormalError {
    /// Coordinates/indices must contain complete triples and indices must be in range.
    InvalidMesh,
    /// Caller vertex budget exceeded.
    VertexLimit,
    /// Output reservation failed.
    Allocation,
}

/// Compute area-weighted vertex normals, retaining source triangle/rounding order.
/// Each addition narrows back to f32 as the original Float32Array assignment does.
/// Isolated/collapsed vertices receive the original +Z fallback. This does not
/// reorient triangle winding; orientNormalsOutward is a separate unported body.
pub fn compute_vertex_normals(
    positions: &[f32],
    indices: &[u32],
    max_vertices: usize,
) -> Result<Vec<f32>, NormalError> {
    if !positions.len().is_multiple_of(3)
        || !indices.len().is_multiple_of(3)
        || indices.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(NormalError::InvalidMesh);
    }
    if positions.len() / 3 > max_vertices {
        return Err(NormalError::VertexLimit);
    }
    let mut normals = Vec::new();
    normals
        .try_reserve_exact(positions.len())
        .map_err(|_| NormalError::Allocation)?;
    normals.resize(positions.len(), 0.0f32);
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        let a = a as usize * 3;
        let b = b as usize * 3;
        let c = c as usize * 3;
        let abx = f64::from(positions[b]) - f64::from(positions[a]);
        let aby = f64::from(positions[b + 1]) - f64::from(positions[a + 1]);
        let abz = f64::from(positions[b + 2]) - f64::from(positions[a + 2]);
        let acx = f64::from(positions[c]) - f64::from(positions[a]);
        let acy = f64::from(positions[c + 1]) - f64::from(positions[a + 1]);
        let acz = f64::from(positions[c + 2]) - f64::from(positions[a + 2]);
        let n = [
            aby * acz - abz * acy,
            abz * acx - abx * acz,
            abx * acy - aby * acx,
        ];
        for v in [a, b, c] {
            for k in 0..3 {
                normals[v + k] = (f64::from(normals[v + k]) + n[k]) as f32;
            }
        }
    }
    for xyz in normals.as_chunks_mut::<3>().0 {
        let [x, y, z] = xyz.map(f64::from);
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-20 {
            for v in xyz {
                *v = (f64::from(*v) / len) as f32;
            }
        } else {
            xyz[2] = 1.;
        }
    }
    Ok(normals)
}
