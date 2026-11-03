// SPDX-License-Identifier: AGPL-3.0-only
//! Complete fixed-selection sphere element output/refusal continuation.
use crate::{
    clip::ClipError,
    js_fixed::fixed3,
    numeric::norm3,
    selection::require_selection,
    sphere::{self, ClippedFit},
};
use std::sync::Arc;
/// Owned settings consumed by the source selected-sphere body.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Source clipping preset; no native default/normalization is substituted.
    pub sigma: f64,
    /// Diameter (not radius). Invalid/nonpositive/nonfinite values are ignored
    /// by the exact original heldRadius guard, not treated as host refusals.
    pub known_diameter: Option<f64>,
}
/// Source held-radius companion summary, present only when a known size held.
#[derive(Debug, Clone, Copy)]
pub struct Held {
    /// Diameter given by a free fit on final used points.
    pub free_diameter: f64,
    /// Distance between free and held centres.
    pub shift: f64,
}
/// Complete SphereFitOutput fields; kind is intrinsically sphere for this type.
#[derive(Debug)]
pub struct MarkedSphere {
    /// Fitted centre, without extra source-absent guards.
    pub center: [f64; 3],
    /// Positive finite accepted radius.
    pub radius: f64,
    /// Exact optional held summary.
    pub held: Option<Held>,
    /// Clipped used-point RMS, in millimetres.
    pub sigma: f64,
    /// Final clipped count, not the original marking size.
    pub used_points: usize,
    /// Original marking count, duplicates retained.
    pub region_size: usize,
    /// Clipped peak-to-peak radial residual.
    pub form_error: f64,
    /// Signed minimum radial residual.
    pub residual_min: f64,
    /// Signed maximum radial residual.
    pub residual_max: f64,
    /// Exact original region owner; clipping never replaces this marking.
    pub region: Arc<[u32]>,
}
/// Exact source FitError versus bounded typed-buffer/work failures.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Source FitError.message; fresh source error construction belongs here.
    Source(String),
    /// Packed positions/selection domain or point budget invalid.
    InvalidInput,
    /// Fallible shared fit/result work reservation failed.
    Allocation,
}
fn held_radius(settings: Settings) -> Option<f64> {
    settings
        .known_diameter
        .filter(|d| d.is_finite() && *d > 0.)
        .map(|d| d / 2.)
}
fn held_limit(diameter: f64) -> f64 {
    (0.1 * diameter).max(1.)
}
fn held_too_far(fin: &ClippedFit) -> bool {
    fin.free
        .is_some_and(|free| (2. * free.r - 2. * fin.sphere.r).abs() > held_limit(2. * fin.sphere.r))
}
fn held_refusal(fin: &ClippedFit) -> String {
    let known = 2. * fin.sphere.r;
    let free = fin.free.map_or(f64::NAN, |s| 2. * s.r);
    format!(
        "The surface here fits a sphere of Ø {} mm, too far from the known Ø {} mm to hold it — check the diameter, or that this is the ball meant.",
        fixed3(free),
        fixed3(known)
    )
}
fn sphere_output(fin: ClippedFit, region: Arc<[u32]>) -> MarkedSphere {
    let s = fin.sphere;
    let held = fin.free.map(|free| Held {
        free_diameter: 2. * free.r,
        shift: norm3([free.cx - s.cx, free.cy - s.cy, free.cz - s.cz]),
    });
    MarkedSphere {
        center: [s.cx, s.cy, s.cz],
        radius: s.r,
        held,
        sigma: fin.sigma,
        used_points: fin.used.len(),
        region_size: region.len(),
        form_error: fin.span,
        residual_min: fin.min,
        residual_max: fin.max,
        region,
    }
}
/// Full fitSphereOnSelection at the positions/owned-settings boundary; no seeded
/// fit, growth, bbox/normal heuristics or early-success stub is introduced.
pub fn fit_sphere_on_selection(
    positions: &[f32],
    selection: Arc<[u32]>,
    settings: Settings,
    max_points: usize,
) -> Result<MarkedSphere, Error> {
    require_selection(selection.len(), "sphere").map_err(Error::Source)?;
    let fitted = sphere::fit_sphere_clipped(
        positions,
        &selection,
        settings.sigma,
        held_radius(settings),
        max_points,
    )
    .map_err(|e| match e {
        ClipError::InvalidInput => Error::InvalidInput,
        ClipError::Allocation => Error::Allocation,
    })?;
    let Some(fin) = fitted.filter(|fin| fin.sphere.r.is_finite() && fin.sphere.r > 0.) else {
        return Err(Error::Source("Couldn't fit a sphere to the marked surface — it curves too little to place a centre. Mark more of the ball.".into()));
    };
    if held_too_far(&fin) {
        return Err(Error::Source(held_refusal(&fin)));
    }
    Ok(sphere_output(fin, selection))
}
