// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen circle2d.ts::fitCircle2d: algebraic (Coope) start about the centroid,
//! then the 2D orthogonal-distance fixed-point iteration.
use crate::{
    clip::ClipError,
    linear::{SolveError, solve_linear},
    numeric::hypot,
};

/// Circle centre and radius in the given plane coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Circle2d {
    /// Centre U.
    pub cu: f64,
    /// Centre V.
    pub cv: f64,
    /// Radius.
    pub r: f64,
}

fn reserve(n: usize) -> Result<Vec<f64>, ClipError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| ClipError::Allocation)?;
    Ok(v)
}

/// Best-fit circle through 2D points; `None` when they do not determine one.
/// `pu0` and `pv0` must have the same length (the source reads `pv0` by `pu0`'s).
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Source NaN-sensitive guards.
pub fn fit_circle_2d(pu0: &[f64], pv0: &[f64]) -> Result<Option<Circle2d>, ClipError> {
    let n = pu0.len();
    if pv0.len() != n {
        return Err(ClipError::InvalidInput);
    }
    if n < 3 {
        return Ok(None);
    }
    let (mut mu, mut mv) = (0., 0.);
    for i in 0..n {
        mu += pu0[i];
        mv += pv0[i];
    }
    let nf = n as f64;
    mu /= nf;
    mv /= nf;
    let mut pu = reserve(n)?;
    let mut pv = reserve(n)?;
    for i in 0..n {
        pu.push(pu0[i] - mu);
        pv.push(pv0[i] - mv);
    }
    let (mut suu, mut suv, mut svv, mut su, mut sv, mut sub, mut svb, mut sb) =
        (0., 0., 0., 0., 0., 0., 0., 0.);
    for i in 0..n {
        let b = pu[i] * pu[i] + pv[i] * pv[i];
        suu += pu[i] * pu[i];
        suv += pu[i] * pv[i];
        svv += pv[i] * pv[i];
        su += pu[i];
        sv += pv[i];
        sub += pu[i] * b;
        svb += pv[i] * b;
        sb += b;
    }
    let mut a = [
        4. * suu,
        4. * suv,
        2. * su,
        4. * suv,
        4. * svv,
        2. * sv,
        2. * su,
        2. * sv,
        nf,
    ];
    let sol = solve_linear(3, &mut a, &[2. * sub, 2. * svb, sb], 3).map_err(|e| match e {
        SolveError::Allocation => ClipError::Allocation,
        SolveError::ShapeOrLimit => ClipError::InvalidInput,
    })?;
    let Some(sol) = sol else { return Ok(None) };
    let mut cu = sol[0];
    let mut cv = sol[1];
    let r2 = sol[2] + cu * cu + cv * cv;
    if !(r2 > 0.) || !r2.is_finite() {
        return Ok(None);
    }
    let mut r = r2.sqrt();
    for _ in 0..100 {
        let (mut sd, mut sxu, mut sxv, mut m) = (0., 0., 0., 0usize);
        for i in 0..n {
            let du = pu[i] - cu;
            let dv = pv[i] - cv;
            let dist = hypot(&[du, dv]);
            if dist < 1e-12 {
                continue;
            }
            sd += dist;
            sxu += du / dist;
            sxv += dv / dist;
            m += 1;
        }
        if m == 0 {
            break;
        }
        let mf = m as f64;
        let r_new = sd / mf;
        let nu = su / nf - r_new * (sxu / mf);
        let nv = sv / nf - r_new * (sxv / mf);
        let step = hypot(&[nu - cu, nv - cv]);
        cu = nu;
        cv = nv;
        r = r_new;
        // Math.max(1, r) is NaN for a NaN radius; f64::max would give 1.
        let scale = if r.is_nan() { f64::NAN } else { r.max(1.) };
        if step < 1e-10 * scale {
            break;
        }
    }
    if !cu.is_finite() || !cv.is_finite() || !r.is_finite() || !(r > 0.) {
        return Ok(None);
    }
    Ok(Some(Circle2d {
        cu: cu + mu,
        cv: cv + mv,
        r,
    }))
}
