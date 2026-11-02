// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen clip.ts::clippedRefit, plus plane.ts::fitPlaneClipped.
use crate::plane::{Plane, fit_plane_tls};
/// Host validation/storage failure, distinct from a numerical fit refusal.
#[derive(Debug, PartialEq, Eq)]
pub enum ClipError {
    /// Invalid XYZ/selection or point budget exceeded.
    InvalidInput,
    /// Work storage reservation failed.
    Allocation,
}
/// Last successful fitted model and original used-point statistics.
#[derive(Debug)]
pub struct ClippedRefit<M> {
    /// Fitted model from the last successful round.
    pub model: M,
    /// RMS of finite residuals.
    pub sigma: f64,
    /// Peak-to-peak residual over finite observations.
    pub span: f64,
    /// Minimum finite residual.
    pub min: f64,
    /// Maximum finite residual.
    pub max: f64,
    /// Ordered selection used in that round, including any source-retained nonfinite points.
    pub used: Vec<u32>,
}
fn reserve<T>(n: usize) -> Result<Vec<T>, ClipError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| ClipError::Allocation)?;
    Ok(v)
}
/// Shared twelve-round clipping loop, preserving the last good result when a
/// subsequent model is degenerate or no finite residual remains. k<=0 uses all
/// points. Callbacks must not derive candidate outputs from reference results.
pub fn clipped_refit<M>(
    positions: &[f32],
    idx: &[u32],
    k: f64,
    max_points: usize,
    mut fit: impl FnMut(&[u32]) -> Result<Option<M>, ClipError>,
    mut residual: impl FnMut(&M, f64, f64, f64) -> f64,
) -> Result<Option<ClippedRefit<M>>, ClipError> {
    if !positions.len().is_multiple_of(3)
        || idx.len() > max_points
        || idx.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(ClipError::InvalidInput);
    }
    let mut used = reserve(idx.len())?;
    used.extend_from_slice(idx);
    let mut result = None;
    for _ in 0..12 {
        let Some(model) = fit(&used)? else {
            return Ok(result);
        };
        let (mut sum_sq, mut m, mut lo, mut hi) = (0., 0usize, f64::INFINITY, f64::NEG_INFINITY);
        let mut res = reserve(used.len())?;
        for &i in &used {
            let j = i as usize * 3;
            let e = residual(
                &model,
                f64::from(positions[j]),
                f64::from(positions[j + 1]),
                f64::from(positions[j + 2]),
            );
            res.push(e);
            if !e.is_finite() {
                continue;
            }
            sum_sq += e * e;
            if e < lo {
                lo = e;
            }
            if e > hi {
                hi = e;
            }
            m += 1;
        }
        if m == 0 {
            return Ok(result);
        }
        let sigma = (sum_sq / m as f64).sqrt();
        let current = ClippedRefit {
            model,
            sigma,
            span: hi - lo,
            min: lo,
            max: hi,
            used,
        };
        if k <= 0. || sigma < 1e-9 {
            return Ok(Some(current));
        }
        let thr = k * sigma;
        let keep = res.iter().filter(|e| e.abs() <= thr).count();
        if keep == current.used.len() || keep < 10 {
            return Ok(Some(current));
        }
        let mut next = reserve(keep)?;
        for (i, &e) in res.iter().enumerate() {
            if e.abs() <= thr {
                next.push(current.used[i]);
            }
        }
        used = next;
        result = Some(current);
    }
    Ok(result)
}
/// Original plane-facing clipping result (not an alias that renames `plane` to `model`).
#[derive(Debug)]
pub struct ClippedPlaneFit {
    /// Last successful TLS plane.
    pub plane: Plane,
    /// RMS over finite used-point residuals.
    pub sigma: f64,
    /// Finite peak-to-peak residual.
    pub span: f64,
    /// Finite minimum residual.
    pub min: f64,
    /// Finite maximum residual.
    pub max: f64,
    /// Ordered source point indices used by the reported fit.
    pub used: Vec<u32>,
}
/// Complete fitPlaneClipped continuation over the generic loop; no RANSAC start.
pub fn fit_plane_clipped(
    positions: &[f32],
    idx: &[u32],
    k: f64,
    max_points: usize,
) -> Result<Option<ClippedPlaneFit>, ClipError> {
    let r = clipped_refit(
        positions,
        idx,
        k,
        max_points,
        |used| fit_plane_tls(positions, used).map_err(|_| ClipError::InvalidInput),
        |plane, x, y, z| plane.residual(x, y, z),
    )?;
    Ok(r.map(|r| ClippedPlaneFit {
        plane: r.model,
        sigma: r.sigma,
        span: r.span,
        min: r.min,
        max: r.max,
        used: r.used,
    }))
}
