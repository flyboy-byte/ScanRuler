// SPDX-License-Identifier: AGPL-3.0-only
//! Complete pinned sphere.ts fixed-selection free/held-radius fitting bodies.
use crate::{
    clip::{self, ClipError},
    linear::{SolveError, solve_linear},
    numeric::norm3,
};
/// Source sphere coordinates and radius, without changed sign/units conventions.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    /// Centre X in model millimetres.
    pub cx: f64,
    /// Centre Y in model millimetres.
    pub cy: f64,
    /// Centre Z in model millimetres.
    pub cz: f64,
    /// Fitted or caller-held radius, not a diameter.
    pub r: f64,
}
/// Full fitSphereClipped result, including the optional free-radius companion.
#[derive(Debug)]
pub struct ClippedFit {
    /// Last successful clipped free or held-radius sphere.
    pub sphere: Sphere,
    /// RMS radial residual on the reported used points.
    pub sigma: f64,
    /// Peak-to-peak radial residual, source sphericity meaning.
    pub span: f64,
    /// Minimum signed radial residual.
    pub min: f64,
    /// Maximum signed radial residual.
    pub max: f64,
    /// Ordered used indices, retaining duplicates and source clipping semantics.
    pub used: Vec<u32>,
    /// Independently refitted free sphere on final used points when radius held.
    pub free: Option<Sphere>,
}
fn solve(n: usize, a: &mut [f64], b: &[f64]) -> Result<Option<Vec<f64>>, ClipError> {
    solve_linear(n, a, b, 4).map_err(|e| match e {
        SolveError::Allocation => ClipError::Allocation,
        SolveError::ShapeOrLimit => ClipError::InvalidInput,
    })
}
fn point(p: &[f32], i: u32) -> [f64; 3] {
    let j = i as usize * 3;
    [f64::from(p[j]), f64::from(p[j + 1]), f64::from(p[j + 2])]
}
fn centroid(p: &[f32], idx: &[u32]) -> [f64; 3] {
    let (mut x, mut y, mut z) = (0., 0., 0.);
    for &i in idx {
        let q = point(p, i);
        x += q[0];
        y += q[1];
        z += q[2]
    }
    let n = idx.len() as f64;
    [x / n, y / n, z / n]
}
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Source's NaN-sensitive r2 guard.
fn algebraic(p: &[f32], idx: &[u32]) -> Result<Option<Sphere>, ClipError> {
    let n = idx.len();
    if n < 4 {
        return Ok(None);
    }
    let [mx, my, mz] = centroid(p, idx);
    let (mut sxx, mut sxy, mut sxz, mut syy, mut syz, mut szz) = (0., 0., 0., 0., 0., 0.);
    let (mut sx, mut sy, mut sz) = (0., 0., 0.);
    let (mut sxb, mut syb, mut szb, mut sb) = (0., 0., 0., 0.);
    for &i in idx {
        let q = point(p, i);
        let qx = q[0] - mx;
        let qy = q[1] - my;
        let qz = q[2] - mz;
        let b = qx * qx + qy * qy + qz * qz;
        sxx += qx * qx;
        sxy += qx * qy;
        sxz += qx * qz;
        syy += qy * qy;
        syz += qy * qz;
        szz += qz * qz;
        sx += qx;
        sy += qy;
        sz += qz;
        sxb += qx * b;
        syb += qy * b;
        szb += qz * b;
        sb += b;
    }
    let mut a = [
        4. * sxx,
        4. * sxy,
        4. * sxz,
        2. * sx,
        4. * sxy,
        4. * syy,
        4. * syz,
        2. * sy,
        4. * sxz,
        4. * syz,
        4. * szz,
        2. * sz,
        2. * sx,
        2. * sy,
        2. * sz,
        n as f64,
    ];
    let rhs = [2. * sxb, 2. * syb, 2. * szb, sb];
    let Some(u) = solve(4, &mut a, &rhs)? else {
        return Ok(None);
    };
    let r2 = u[3] + u[0] * u[0] + u[1] * u[1] + u[2] * u[2];
    if !(r2 > 0.) || !r2.is_finite() {
        return Ok(None);
    }
    Ok(Some(Sphere {
        cx: mx + u[0],
        cy: my + u[1],
        cz: mz + u[2],
        r: r2.sqrt(),
    }))
}
fn max_one(r: f64) -> f64 {
    if r.is_nan() || r > 1. { r } else { 1. }
}
fn geometric(p: &[f32], idx: &[u32], init: Sphere, max_iter: usize) -> Sphere {
    if idx.is_empty() {
        return init;
    }
    let [mx, my, mz] = centroid(p, idx);
    let (mut cx, mut cy, mut cz, mut r) = (init.cx, init.cy, init.cz, init.r);
    for _ in 0..max_iter {
        let (mut sd, mut sux, mut suy, mut suz, mut m) = (0., 0., 0., 0., 0usize);
        for &i in idx {
            let q = point(p, i);
            let dx = q[0] - cx;
            let dy = q[1] - cy;
            let dz = q[2] - cz;
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d < 1e-12 {
                continue;
            }
            sd += d;
            sux += dx / d;
            suy += dy / d;
            suz += dz / d;
            m += 1;
        }
        if m == 0 {
            break;
        }
        let m = m as f64;
        let r_new = sd / m;
        let ncx = mx - r_new * (sux / m);
        let ncy = my - r_new * (suy / m);
        let ncz = mz - r_new * (suz / m);
        let dx = ncx - cx;
        let dy = ncy - cy;
        let dz = ncz - cz;
        let movement = (dx * dx + dy * dy + dz * dz).sqrt();
        cx = ncx;
        cy = ncy;
        cz = ncz;
        r = r_new;
        if movement < 1e-10 * max_one(r) {
            break;
        }
    }
    if !cx.is_finite() || !r.is_finite() {
        init
    } else {
        Sphere { cx, cy, cz, r }
    }
}
fn centre(
    p: &[f32],
    idx: &[u32],
    r: f64,
    start: Sphere,
    max_iter: usize,
) -> Result<Option<Sphere>, ClipError> {
    let (mut cx, mut cy, mut cz) = (start.cx, start.cy, start.cz);
    for _ in 0..max_iter {
        let (mut axx, mut axy, mut axz, mut ayy, mut ayz, mut azz) = (0., 0., 0., 0., 0., 0.);
        let (mut bx, mut by, mut bz) = (0., 0., 0.);
        for &i in idx {
            let q = point(p, i);
            let dx = q[0] - cx;
            let dy = q[1] - cy;
            let dz = q[2] - cz;
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d < 1e-12 {
                continue;
            }
            let ux = dx / d;
            let uy = dy / d;
            let uz = dz / d;
            let e = d - r;
            axx += ux * ux;
            axy += ux * uy;
            axz += ux * uz;
            ayy += uy * uy;
            ayz += uy * uz;
            azz += uz * uz;
            bx += ux * e;
            by += uy * e;
            bz += uz * e;
        }
        let Some(step) = solve(
            3,
            &mut [axx, axy, axz, axy, ayy, ayz, axz, ayz, azz],
            &[bx, by, bz],
        )?
        else {
            return Ok(None);
        };
        cx += step[0];
        cy += step[1];
        cz += step[2];
        if norm3([step[0], step[1], step[2]]) < 1e-10 * max_one(r) {
            break;
        }
    }
    Ok(if cx.is_finite() && cy.is_finite() && cz.is_finite() {
        Some(Sphere { cx, cy, cz, r })
    } else {
        None
    })
}
fn free(p: &[f32], idx: &[u32]) -> Result<Option<Sphere>, ClipError> {
    Ok(algebraic(p, idx)?.map(|s| geometric(p, idx, s, 150)))
}
/// Full source fixed-selection sphere fit. k<=0 disables clipping; held preserves
/// all source numerical behavior, including zero/negative/nonfinite input radii.
/// Invalid Rust buffer domains/point limits are separate host errors; numerical
/// degeneracy remains Ok(None). No normal voting, region growing or element shell.
pub fn fit_sphere_clipped(
    p: &[f32],
    idx: &[u32],
    k: f64,
    held: Option<f64>,
    max_points: usize,
) -> Result<Option<ClippedFit>, ClipError> {
    let result = clip::clipped_refit(
        p,
        idx,
        k,
        max_points,
        |used| {
            let fitted = free(p, used)?;
            match (fitted, held) {
                (Some(free), Some(r)) => centre(p, used, r, free, 50),
                (free, _) => Ok(free),
            }
        },
        |s, x, y, z| {
            let dx = x - s.cx;
            let dy = y - s.cy;
            let dz = z - s.cz;
            (dx * dx + dy * dy + dz * dz).sqrt() - s.r
        },
    )?;
    let Some(result) = result else {
        return Ok(None);
    };
    let companion = if held.is_some() {
        free(p, &result.used)?
    } else {
        None
    };
    Ok(Some(ClippedFit {
        sphere: result.model,
        sigma: result.sigma,
        span: result.span,
        min: result.min,
        max: result.max,
        used: result.used,
        free: companion,
    }))
}
