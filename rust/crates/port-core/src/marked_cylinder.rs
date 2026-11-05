// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen fitCylinderFromSeed.ts::fitCylinderOnSelection (the hand-painted
//! cylinder), with cylinderResult, fitInsideWindow and extents.ts::axialExtents.
use crate::{
    clip::ClipError,
    cylinder::{
        ClippedCylinderFit, Cylinder, axis_from_normals, fit_cylinder_clipped,
        fit_cylinder_on_axis, ransac_cylinder,
    },
    numeric::js_atan2,
    ransac::{RansacError, RansacLimits, RansacOptions},
    selection::{MIN_SELECTED, require_selection},
};
use std::sync::Arc;

/// The span a fit is confined to: millimetres past each end of the measured
/// surface (negative pulls the end in).
#[derive(Debug, Clone, Copy)]
pub struct AxialWindow {
    /// Past the low end.
    pub start: f64,
    /// Past the high end.
    pub end: f64,
}

/// The CylinderFit fields fitCylinderOnSelection reports.
#[derive(Debug, Clone)]
pub struct MarkedCylinder {
    /// Axis point at the middle of the surface's axial span.
    pub center: [f64; 3],
    /// Unit axis direction.
    pub axis: [f64; 3],
    /// Radius.
    pub radius: f64,
    /// Axial length of the surface.
    pub length: f64,
    /// Degrees of arc round the axis the used surface covers.
    pub coverage: f64,
    /// Clipped RMS radial residual.
    pub sigma: f64,
    /// Final clipped point count.
    pub used_points: usize,
    /// Points the fit ran on before clipping.
    pub region_size: usize,
    /// Peak-to-peak radial residual.
    pub form_error: f64,
    /// Most negative radial residual.
    pub residual_min: f64,
    /// Most positive radial residual.
    pub residual_max: f64,
    /// The points the fit ran on.
    pub region: Arc<[u32]>,
}

/// Source FitError versus host failures.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Source FitError message.
    Source(String),
    /// Packed buffers, selection domain or point budget invalid.
    InvalidInput,
    /// Work storage could not be reserved.
    Allocation,
}

impl From<ClipError> for Error {
    fn from(e: ClipError) -> Self {
        match e {
            ClipError::InvalidInput => Self::InvalidInput,
            ClipError::Allocation => Self::Allocation,
        }
    }
}

const COVERAGE_SAMPLES: usize = 20_000;

/// extents.ts::axialExtents: degrees covered round the axis (360 minus the widest
/// gap) and the axial reach of the region.
pub fn axial_extents(p: &[f32], region: &[u32], c: &Cylinder) -> (f64, f64, f64) {
    let (u, v) = crate::cylinder::basis([c.ax, c.ay, c.az]);
    let mut min_t = f64::INFINITY;
    let mut max_t = f64::NEG_INFINITY;
    let step = region.len().div_ceil(COVERAGE_SAMPLES).max(1);
    let mut angles = Vec::new();
    for (i, &k) in region.iter().enumerate() {
        let j = k as usize * 3;
        let qx = f64::from(p[j]) - c.px;
        let qy = f64::from(p[j + 1]) - c.py;
        let qz = f64::from(p[j + 2]) - c.pz;
        let t = qx * c.ax + qy * c.ay + qz * c.az;
        if t < min_t {
            min_t = t;
        }
        if t > max_t {
            max_t = t;
        }
        if i % step == 0 {
            let wu = qx * u[0] + qy * u[1] + qz * u[2];
            let wv = qx * v[0] + qy * v[1] + qz * v[2];
            angles.push(js_atan2(wv, wu));
        }
    }
    if angles.len() < 2 {
        return (0., min_t, max_t);
    }
    // (a, b) => a - b; a NaN angle only arises from non-finite positions.
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut widest = angles[0] + 2. * std::f64::consts::PI - angles[angles.len() - 1];
    for i in 1..angles.len() {
        let gap = angles[i] - angles[i - 1];
        if gap > widest {
            widest = gap;
        }
    }
    let coverage = 360. - (widest * 180.) / std::f64::consts::PI;
    let coverage = if coverage.is_nan() {
        f64::NAN
    } else {
        coverage.max(0.)
    };
    (coverage, min_t, max_t)
}

#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn cylinder_result(
    p: &[f32],
    fin: &ClippedCylinderFit,
    used: &Arc<[u32]>,
    extent: Option<&[u32]>,
) -> Option<MarkedCylinder> {
    let c = fin.cylinder;
    let (coverage, lo, hi) = axial_extents(p, used, &c);
    let (min_t, max_t) = match extent {
        None => (lo, hi),
        Some(e) => {
            let (_, a, b) = axial_extents(p, e, &c);
            (a, b)
        }
    };
    let length = max_t - min_t;
    if !(length > 0.) {
        return None;
    }
    let mid = (min_t + max_t) / 2.;
    Some(MarkedCylinder {
        center: [c.px + mid * c.ax, c.py + mid * c.ay, c.pz + mid * c.az],
        axis: [c.ax, c.ay, c.az],
        radius: c.r,
        length,
        coverage,
        sigma: fin.sigma,
        used_points: fin.used.len(),
        region_size: used.len(),
        form_error: fin.span,
        residual_min: fin.min,
        residual_max: fin.max,
        region: used.clone(),
    })
}

fn usable(fin: Option<ClippedCylinderFit>) -> Option<ClippedCylinderFit> {
    fin.filter(|f| f.cylinder.r.is_finite() && f.cylinder.r > 0.)
}

#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn fit_inside_window(
    p: &[f32],
    full: MarkedCylinder,
    window: AxialWindow,
    sigma: f64,
    max_points: usize,
) -> Result<MarkedCylinder, Error> {
    if !(window.start < 0.) && !(window.end < 0.) {
        return Ok(full);
    }
    let region = &full.region;
    let [ax, ay, az] = full.axis;
    let [cx, cy, cz] = full.center;
    let half = full.length / 2.;
    let lo = -half - window.start;
    let hi = half + window.end;
    let mut kept = Vec::new();
    for &k in region.iter() {
        let j = k as usize * 3;
        let t = (f64::from(p[j]) - cx) * ax
            + (f64::from(p[j + 1]) - cy) * ay
            + (f64::from(p[j + 2]) - cz) * az;
        if t >= lo && t <= hi {
            kept.push(k);
        }
    }
    if kept.len() < MIN_SELECTED {
        return Err(Error::Source(format!(
            "Only {} of the {} points lie inside the span — a fit needs at least {MIN_SELECTED}. Pull the ends back out, or switch the option off.",
            kept.len(),
            region.len()
        )));
    }
    let used: Arc<[u32]> = kept.into();
    let init = Cylinder {
        px: cx,
        py: cy,
        pz: cz,
        ax,
        ay,
        az,
        r: full.radius,
    };
    let Some(fin) = usable(fit_cylinder_clipped(p, &used, init, sigma, max_points)?) else {
        return Err(Error::Source(
            "Couldn't fit a cylinder to the surface inside the span.".into(),
        ));
    };
    cylinder_result(p, &fin, &used, Some(region)).ok_or_else(|| {
        Error::Source("Couldn't measure the length of the surface inside the span.".into())
    })
}

/// fitCylinderOnSelection: normal-covariance axis (else the robust estimate),
/// clipped best fit, reported over the marked surface, optionally refitted
/// inside an axial window. `normals` is packed like `positions`.
pub fn fit_cylinder_on_selection(
    positions: &[f32],
    normals: &[f32],
    selection: Arc<[u32]>,
    sigma: f64,
    window: Option<AxialWindow>,
    max_points: usize,
) -> Result<MarkedCylinder, Error> {
    require_selection(selection.len(), "cylindrical surface").map_err(Error::Source)?;
    if normals.len() != positions.len() || selection.len() > max_points {
        return Err(Error::InvalidInput);
    }
    let mut init = match axis_from_normals(normals, &selection)? {
        Some(axis) => fit_cylinder_on_axis(positions, &selection, axis)?,
        None => None,
    };
    if init.is_none() {
        init = ransac_cylinder(
            positions,
            normals,
            &selection,
            RansacOptions {
                seed: 0x5eed,
                iterations: RansacOptions::default().iterations,
            },
            RansacLimits {
                max_points,
                max_iterations: RansacOptions::default().iterations,
            },
        )
        .map_err(|e| match e {
            RansacError::Allocation => Error::Allocation,
            _ => Error::InvalidInput,
        })?
        .map(|r| r.cylinder);
    }
    let Some(init) = init else {
        return Err(Error::Source("Couldn't find an axis in the marked surface — it curves too little to be told from a plane. Mark more of the way around the cylinder.".into()));
    };
    let Some(fin) = usable(fit_cylinder_clipped(
        positions, &selection, init, sigma, max_points,
    )?) else {
        return Err(Error::Source(
            "Couldn't fit a cylinder to the marked surface.".into(),
        ));
    };
    let result = cylinder_result(positions, &fin, &selection, None).ok_or_else(|| {
        Error::Source("Couldn't measure the length of the marked surface.".into())
    })?;
    match window {
        Some(w) => fit_inside_window(positions, result, w, sigma, max_points),
        None => Ok(result),
    }
}
