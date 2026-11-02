// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen linalg.ts::rmsOver and plane.ts::curvedBeyondPlane.
use crate::plane::{InvalidPlaneInput, fit_plane_tls};

fn validate(positions: &[f32], idx: &[u32]) -> Result<(), InvalidPlaneInput> {
    if !positions.len().is_multiple_of(3) || idx.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(InvalidPlaneInput);
    }
    Ok(())
}

/// Root mean square over finite residuals, in selection order (duplicates retained).
/// No finite answers yields infinity; overflow of finite squared answers also
/// yields infinity. Invalid packed coordinates/indices fail before any callback.
pub fn rms_over(
    positions: &[f32],
    idx: &[u32],
    mut residual: impl FnMut(f64, f64, f64) -> f64,
) -> Result<f64, InvalidPlaneInput> {
    validate(positions, idx)?;
    let mut sum_sq = 0.;
    let mut m = 0usize;
    for &i in idx {
        let j = i as usize * 3;
        let e = residual(
            f64::from(positions[j]),
            f64::from(positions[j + 1]),
            f64::from(positions[j + 2]),
        );
        if !e.is_finite() {
            continue;
        }
        sum_sq += e * e;
        m += 1;
    }
    Ok(if m == 0 {
        f64::INFINITY
    } else {
        (sum_sq / m as f64).sqrt()
    })
}

/// Whether the curved fit improves over the best plane by the source ratio 1.1.
/// A refused plane returns true, retaining the source continuation. NaN, negative
/// and infinite curved RMS values use the original IEEE comparison semantics.
pub fn curved_beyond_plane(
    positions: &[f32],
    idx: &[u32],
    curved_rms: f64,
) -> Result<bool, InvalidPlaneInput> {
    let Some(plane) = fit_plane_tls(positions, idx)? else {
        return Ok(true);
    };
    Ok(rms_over(positions, idx, |x, y, z| plane.residual(x, y, z))? >= 1.1 * curved_rms)
}
