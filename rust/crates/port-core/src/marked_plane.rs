// SPDX-License-Identifier: AGPL-3.0-only
//! Complete frozen fitPlaneFromSeed.ts::fitPlaneOnSelection continuation.
use crate::plane::Plane;

/// Full plane element geometry/statistics over the original marked region.
#[derive(Debug)]
pub struct MarkedPlane {
    /// Projected center of the original selection's rectangle.
    pub center: [f64; 3],
    /// Plane normal facing the selected surface normals.
    pub normal: [f64; 3],
    /// First axis of the source minimum-area sampled rectangle.
    pub basis_u: [f64; 3],
    /// Second rectangle axis.
    pub basis_v: [f64; 3],
    /// Half-width along basis_u, using every original selected point.
    pub extent_u: f64,
    /// Half-width along basis_v.
    pub extent_v: f64,
    /// Clipped used-point residual RMS.
    pub sigma: f64,
    /// Final clipped used-point count (not the original region length).
    pub used_points: usize,
    /// Original marked-point count, duplicates retained.
    pub region_size: usize,
    /// Clipped peak-to-peak residual.
    pub form_error: f64,
    /// Minimum residual against the reported normal.
    pub residual_min: f64,
    /// Maximum residual against the reported normal.
    pub residual_max: f64,
    /// Original ordered marking; clipping never replaces this region.
    pub region: Vec<u32>,
}
/// Exact source refusals or explicit bounded host failure.
#[derive(Debug, PartialEq, Eq)]
pub enum MarkedPlaneError {
    /// Source FitError message, unchanged.
    Source(String),
    /// Packed buffers, selection indices or caller point limit invalid.
    InvalidInput,
    /// Work/result storage reservation failed.
    Allocation,
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = crate::numeric::norm3(v);
    if len.partial_cmp(&1e-20) != Some(std::cmp::Ordering::Greater) {
        return [0., 0., 1.];
    }
    v.map(|x| x / len)
}
fn basis(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let [ax, ay, az] = n.map(f64::abs);
    let helper = if ax <= ay && ax <= az {
        [1., 0., 0.]
    } else if ay <= az {
        [0., 1., 0.]
    } else {
        [0., 0., 1.]
    };
    let u = normalize(cross(n, helper));
    let v = normalize(cross(n, u));
    (u, v)
}
struct Extents {
    center: [f64; 3],
    u: [f64; 3],
    v: [f64; 3],
    eu: f64,
    ev: f64,
}
fn patch_extents(
    positions: &[f32],
    region: &[u32],
    p: Plane,
) -> Result<Option<Extents>, MarkedPlaneError> {
    let n = region.len();
    let mut mean = [0.; 3];
    for &i in region {
        let j = i as usize * 3;
        for k in 0..3 {
            mean[k] += f64::from(positions[j + k]);
        }
    }
    for x in &mut mean {
        *x /= n as f64;
    }
    let [mx, my, mz] = mean;
    let (u0, v0) = basis(p.normal);
    let step = n.div_ceil(4096).max(1);
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(n.div_ceil(step))
        .map_err(|_| MarkedPlaneError::Allocation)?;
    for i in (0..n).step_by(step) {
        let j = region[i] as usize * 3;
        let x = f64::from(positions[j]) - mx;
        let y = f64::from(positions[j + 1]) - my;
        let z = f64::from(positions[j + 2]) - mz;
        samples.push([
            x * u0[0] + y * u0[1] + z * u0[2],
            x * v0[0] + y * v0[1] + z * v0[2],
        ]);
    }
    if samples.len() < 3 {
        return Ok(None);
    }
    let mut best_area = f64::INFINITY;
    let mut best_angle = 0.;
    for t in 0..45 {
        let angle = (t as f64 / 45.) * (std::f64::consts::PI / 2.);
        let c = angle.cos();
        let s = angle.sin();
        let (mut lo1, mut hi1, mut lo2, mut hi2) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        for &[sa, sb] in &samples {
            let q1 = sa * c + sb * s;
            let q2 = -sa * s + sb * c;
            if q1 < lo1 {
                lo1 = q1;
            }
            if q1 > hi1 {
                hi1 = q1;
            }
            if q2 < lo2 {
                lo2 = q2;
            }
            if q2 > hi2 {
                hi2 = q2;
            }
        }
        let area = (hi1 - lo1) * (hi2 - lo2);
        if area < best_area {
            best_area = area;
            best_angle = angle;
        }
    }
    let ca = best_angle.cos();
    let sn = best_angle.sin();
    let u = [
        u0[0] * ca + v0[0] * sn,
        u0[1] * ca + v0[1] * sn,
        u0[2] * ca + v0[2] * sn,
    ];
    let v = [
        -u0[0] * sn + v0[0] * ca,
        -u0[1] * sn + v0[1] * ca,
        -u0[2] * sn + v0[2] * ca,
    ];
    let (mut min_u, mut max_u, mut min_v, mut max_v) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for &i in region {
        let j = i as usize * 3;
        let x = f64::from(positions[j]) - mx;
        let y = f64::from(positions[j + 1]) - my;
        let z = f64::from(positions[j + 2]) - mz;
        let pu = x * u[0] + y * u[1] + z * u[2];
        let pv = x * v[0] + y * v[1] + z * v[2];
        if pu < min_u {
            min_u = pu;
        }
        if pu > max_u {
            max_u = pu;
        }
        if pv < min_v {
            min_v = pv;
        }
        if pv > max_v {
            max_v = pv;
        }
    }
    if !min_u.is_finite() || !min_v.is_finite() {
        return Ok(None);
    }
    let cu = (min_u + max_u) / 2.;
    let cv = (min_v + max_v) / 2.;
    let cx = mx + cu * u[0] + cv * v[0];
    let cy = my + cu * u[1] + cv * v[1];
    let cz = mz + cu * u[2] + cv * v[2];
    let [nx, ny, nz] = p.normal;
    let off = nx * cx + ny * cy + nz * cz - p.d;
    Ok(Some(Extents {
        center: [cx - off * nx, cy - off * ny, cz - off * nz],
        u,
        v,
        eu: (max_u - min_u) / 2.,
        ev: (max_v - min_v) / 2.,
    }))
}

/// Fit on exactly the marking, with the source minimum of twenty selected entries.
/// Normals and extents use the original selection even when clipping rejects points.
/// No seeded region growth or source roughness gate is added to the marked path.
pub fn fit_plane_on_selection(
    positions: &[f32],
    normals: &[f32],
    selection: &[u32],
    sigma: f64,
    max_points: usize,
) -> Result<MarkedPlane, MarkedPlaneError> {
    if selection.len() < 20 {
        return Err(MarkedPlaneError::Source(format!(
            "Paint more of the surface — a fit needs at least 20 marked points, and this selection has {}.",
            selection.len()
        )));
    }
    if !positions.len().is_multiple_of(3)
        || normals.len() != positions.len()
        || selection.len() > max_points
        || selection.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(MarkedPlaneError::InvalidInput);
    }
    let fin=crate::clip::fit_plane_clipped(positions,selection,sigma,max_points).map_err(|e|match e{crate::clip::ClipError::Allocation=>MarkedPlaneError::Allocation,crate::clip::ClipError::InvalidInput=>MarkedPlaneError::InvalidInput})?.ok_or_else(||MarkedPlaneError::Source("Couldn't fit a plane to the marked surface — the points lie along a line rather than spanning an area.".into()))?;
    let mut vote = 0.;
    let [nx, ny, nz] = fin.plane.normal;
    for &i in selection {
        let j = i as usize * 3;
        vote += nx * f64::from(normals[j])
            + ny * f64::from(normals[j + 1])
            + nz * f64::from(normals[j + 2]);
    }
    let plane = if vote >= 0. {
        fin.plane
    } else {
        Plane {
            normal: fin.plane.normal.map(|x| -x),
            d: -fin.plane.d,
        }
    };
    let ext = patch_extents(positions, selection, plane)?.ok_or_else(|| {
        MarkedPlaneError::Source(
            "Couldn't measure the extent of the marked surface — try marking a wider patch.".into(),
        )
    })?;
    let same = fin.plane.normal[0] * plane.normal[0]
        + fin.plane.normal[1] * plane.normal[1]
        + fin.plane.normal[2] * plane.normal[2]
        > 0.;
    let (residual_min, residual_max) = if same {
        (fin.min, fin.max)
    } else {
        (-fin.max, -fin.min)
    };
    let mut region = Vec::new();
    region
        .try_reserve_exact(selection.len())
        .map_err(|_| MarkedPlaneError::Allocation)?;
    region.extend_from_slice(selection);
    Ok(MarkedPlane {
        center: ext.center,
        normal: plane.normal,
        basis_u: ext.u,
        basis_v: ext.v,
        extent_u: ext.eu,
        extent_v: ext.ev,
        sigma: fin.sigma,
        used_points: fin.used.len(),
        region_size: selection.len(),
        form_error: fin.span,
        residual_min,
        residual_max,
        region,
    })
}
