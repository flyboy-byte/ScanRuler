// SPDX-License-Identifier: AGPL-3.0-only
//! Complete frozen centroid.ts area/centroid bodies, reusing bounds and tetraVolume6.
use crate::{bounds::extent_of_indexed, numeric::norm3, orientation::tetra_volume6};
/// Packed coordinates or triangle indices violate the Rust mesh domain.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidCentroidMesh;
/// Original centroid meaning: surface center and conditional enclosed-volume center.
#[derive(Debug)]
pub struct MeshCentroid {
    /// Enclosed-volume centroid, absent for the original open/degenerate refusals.
    pub volume: Option<[f64; 3]>,
    /// Surface-area-weighted centroid or original zero-area fallback.
    pub area: [f64; 3],
    /// Original two-origin signed-volume heuristic, not a topology proof.
    pub closed: bool,
    /// Unsigned volume estimate; meaningful as enclosed volume only when closed.
    pub volume_mm3: f64,
}
fn valid(p: &[f32], idx: &[u32]) -> Result<(), InvalidCentroidMesh> {
    if !p.len().is_multiple_of(3)
        || !idx.len().is_multiple_of(3)
        || idx.iter().any(|&i| i as usize >= p.len() / 3)
    {
        return Err(InvalidCentroidMesh);
    }
    Ok(())
}
fn point(p: &[f32], i: u32) -> [f64; 3] {
    let j = i as usize * 3;
    [f64::from(p[j]), f64::from(p[j + 1]), f64::from(p[j + 2])]
}
fn cross(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let ux = b[0] - a[0];
    let uy = b[1] - a[1];
    let uz = b[2] - a[2];
    let vx = c[0] - a[0];
    let vy = c[1] - a[1];
    let vz = c[2] - a[2];
    [uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx]
}
/// Sum source triangle areas in input order, including repeated faces.
pub fn triangle_area_sum(p: &[f32], idx: &[u32]) -> Result<f64, InvalidCentroidMesh> {
    valid(p, idx)?;
    let mut area = 0.;
    for &[a, b, c] in idx.as_chunks::<3>().0 {
        area += norm3(cross(point(p, a), point(p, b), point(p, c))) / 2.;
    }
    Ok(area)
}
/// Complete surface/volume centroid computation with the original two-origin
/// closure decision and zero-area fallback. No repairs or topology rewriting.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Source NaN-sensitive fallback.
pub fn mesh_centroid(p: &[f32], idx: &[u32]) -> Result<MeshCentroid, InvalidCentroidMesh> {
    valid(p, idx)?;
    let extent = extent_of_indexed(p, idx).map_err(|_| InvalidCentroidMesh)?;
    let min = extent.min;
    let max = extent.max;
    let (mut ax, mut ay, mut az, mut area_sum) = (0., 0., 0., 0.);
    for &[a, b, c] in idx.as_chunks::<3>().0 {
        let a = point(p, a);
        let b = point(p, b);
        let c = point(p, c);
        let area = norm3(cross(a, b, c)) / 2.;
        ax += ((a[0] + b[0] + c[0]) / 3.) * area;
        ay += ((a[1] + b[1] + c[1]) / 3.) * area;
        az += ((a[2] + b[2] + c[2]) / 3.) * area;
        area_sum += area;
    }
    if !(area_sum > 0.) {
        let mid = if min[0].is_finite() {
            [
                (min[0] + max[0]) / 2.,
                (min[1] + max[1]) / 2.,
                (min[2] + max[2]) / 2.,
            ]
        } else {
            [0.; 3]
        };
        return Ok(MeshCentroid {
            volume: None,
            area: mid,
            closed: false,
            volume_mm3: 0.,
        });
    }
    let area = [ax / area_sum, ay / area_sum, az / area_sum];
    let diag = norm3([max[0] - min[0], max[1] - min[1], max[2] - min[2]]);
    let o1 = area;
    let o2 = [area[0] + diag, area[1] + 0.7 * diag, area[2] + 0.4 * diag];
    let (mut v1, mut v2, mut mx, mut my, mut mz) = (0., 0., 0., 0., 0.);
    for &[a, b, c] in idx.as_chunks::<3>().0 {
        let a = point(p, a);
        let b = point(p, b);
        let c = point(p, c);
        v1 += tetra_volume6(a, b, c, o1) / 6.;
        v2 += tetra_volume6(a, b, c, o2) / 6.;
        let v = tetra_volume6(a, b, c, o1) / 6.;
        mx += v * (a[0] + b[0] + c[0] - 3. * o1[0]) / 4.;
        my += v * (a[1] + b[1] + c[1] - 3. * o1[1]) / 4.;
        mz += v * (a[2] + b[2] + c[2] - 3. * o1[2]) / 4.;
    }
    let closed = v1.abs() > 0. && (v1 - v2).abs() <= 0.01 * v1.abs();
    let volume = if closed {
        Some([o1[0] + mx / v1, o1[1] + my / v1, o1[2] + mz / v1])
    } else {
        None
    };
    Ok(MeshCentroid {
        volume,
        area,
        closed,
        volume_mm3: v1.abs(),
    })
}
