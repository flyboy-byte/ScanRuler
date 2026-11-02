// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen normals.ts::orientNormalsOutward and its minimal extent/volume dependencies.

/// Mesh contract violation detected before mutation.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidOrientationMesh;

/// Six times signed tetrahedron volume; coordinates are independent copies of
/// the three source corners and supplied origin. Operation order matches tetraVolume6.
pub fn tetra_volume6(a: [f64; 3], b: [f64; 3], c: [f64; 3], o: [f64; 3]) -> f64 {
    let ax = a[0] - o[0];
    let ay = a[1] - o[1];
    let az = a[2] - o[2];
    let bx = b[0] - o[0];
    let by = b[1] - o[1];
    let bz = b[2] - o[2];
    let cx = c[0] - o[0];
    let cy = c[1] - o[1];
    let cz = c[2] - o[2];
    ax * (by * cz - bz * cy) + ay * (bz * cx - bx * cz) + az * (bx * cy - by * cx)
}

/// Orient normals and winding together using the source's signed-volume threshold.
/// Returns whether both arrays were reversed. Does not infer closure or repair a
/// mesh: open scans retain the source's heuristic limitations. Invalid Rust mesh
/// shapes/indices return an error without partially changing either array.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Preserve the source NaN-sensitive refusal.
pub fn orient_normals_outward(
    positions: &[f32],
    indices: &mut [u32],
    normals: &mut [f32],
) -> Result<bool, InvalidOrientationMesh> {
    if !positions.len().is_multiple_of(3)
        || !indices.len().is_multiple_of(3)
        || normals.len() != positions.len()
        || indices.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(InvalidOrientationMesh);
    }
    let count = positions.len() / 3;
    if count == 0 {
        return Ok(false);
    }
    let mut centre = [0.; 3];
    for xyz in positions.as_chunks::<3>().0 {
        for k in 0..3 {
            let x = f64::from(xyz[k]);
            centre[k] += x;
        }
    }
    let crate::bounds::Extent { min, max } =
        crate::bounds::extent_of(positions, count).map_err(|_| InvalidOrientationMesh)?;
    for c in &mut centre {
        *c /= count as f64;
    }
    let mut volume6 = 0.;
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        let point = |i: u32| {
            let j = i as usize * 3;
            [
                f64::from(positions[j]),
                f64::from(positions[j + 1]),
                f64::from(positions[j + 2]),
            ]
        };
        volume6 += tetra_volume6(point(a), point(b), point(c), centre);
    }
    let bbox = (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2]);
    // Preserve the source's comparisons, including its NaN branch behavior.
    if !(bbox > 0.) || volume6 >= -6e-3 * bbox {
        return Ok(false);
    }
    for n in normals {
        *n = -*n;
    }
    for tri in indices.as_chunks_mut::<3>().0 {
        tri.swap(1, 2);
    }
    Ok(true)
}
