// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen cylinder.ts: axis from normals, circle across the axis, damped
//! Gauss-Newton refinement of all five degrees of freedom, clipped refitting and
//! the LMedS robust fit. `splitTwoCylinders` (region growing's one-or-two test)
//! is not ported here.
use crate::{
    circle2d::fit_circle_2d,
    clip::{ClipError, clipped_refit},
    curvature_gate::{curved_beyond_plane, rms_over},
    linear::{SolveError, solve_linear},
    numeric::{hypot, norm3},
    plane::{eigen3, normalize_or_z},
    ransac::{
        ConsensusShape, Mulberry32, RansacError, RansacLimits, RansacOptions, ransac_consensus,
    },
};

/// Source cylinder: a point on the axis, the unit axis direction and the radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    /// Axis point X.
    pub px: f64,
    /// Axis point Y.
    pub py: f64,
    /// Axis point Z.
    pub pz: f64,
    /// Axis direction X.
    pub ax: f64,
    /// Axis direction Y.
    pub ay: f64,
    /// Axis direction Z.
    pub az: f64,
    /// Radius.
    pub r: f64,
}

struct Radial {
    e: f64,
    wx: f64,
    wy: f64,
    wz: f64,
    t: f64,
}

#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn radial(c: &Cylinder, x: f64, y: f64, z: f64) -> Radial {
    let qx = x - c.px;
    let qy = y - c.py;
    let qz = z - c.pz;
    let t = qx * c.ax + qy * c.ay + qz * c.az;
    let wx = qx - t * c.ax;
    let wy = qy - t * c.ay;
    let wz = qz - t * c.az;
    let rho = (wx * wx + wy * wy + wz * wz).sqrt();
    if !(rho > 1e-12) {
        return Radial {
            e: f64::NAN,
            wx: 0.,
            wy: 0.,
            wz: 0.,
            t,
        };
    }
    Radial {
        e: rho - c.r,
        wx: wx / rho,
        wy: wy / rho,
        wz: wz / rho,
        t,
    }
}

/// Signed distance from the cylinder surface (positive outside); NaN on the axis.
pub fn cylinder_residual(c: &Cylinder, x: f64, y: f64, z: f64) -> f64 {
    radial(c, x, y, z).e
}

fn validate(len: usize, idx: &[u32]) -> Result<(), ClipError> {
    if !len.is_multiple_of(3) || idx.iter().any(|&i| i as usize >= len / 3) {
        return Err(ClipError::InvalidInput);
    }
    Ok(())
}

fn rms(p: &[f32], idx: &[u32], c: &Cylinder) -> Result<f64, ClipError> {
    rms_over(p, idx, |x, y, z| radial(c, x, y, z).e).map_err(|_| ClipError::InvalidInput)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// linalg.ts::orthoBasis: two unit vectors completing `n` into a right-handed frame.
pub(crate) fn basis(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    ortho_basis(n)
}
fn ortho_basis(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let ax = n[0].abs();
    let ay = n[1].abs();
    let az = n[2].abs();
    let helper = if ax <= ay && ax <= az {
        [1., 0., 0.]
    } else if ay <= az {
        [0., 1., 0.]
    } else {
        [0., 0., 1.]
    };
    let u = normalize_or_z(cross(n, helper));
    let v = normalize_or_z(cross(n, u));
    (u, v)
}

fn js_max(values: &[f64]) -> f64 {
    let mut m = f64::NEG_INFINITY;
    for &v in values {
        if v.is_nan() {
            return f64::NAN;
        }
        if v > m {
            m = v;
        }
    }
    m
}

/// Axis direction from surface normals: the direction the normal cloud scatters
/// least along. `None` for fewer than three normals or an almost flat patch.
/// `normals` must be packed XYZ with every index in range.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn axis_from_normals(normals: &[f32], idx: &[u32]) -> Result<Option<[f64; 3]>, ClipError> {
    validate(normals.len(), idx)?;
    let n = idx.len();
    if n < 3 {
        return Ok(None);
    }
    let (mut cxx, mut cxy, mut cxz, mut cyy, mut cyz, mut czz) = (0., 0., 0., 0., 0., 0.);
    for &i in idx {
        let j = i as usize * 3;
        let x = f64::from(normals[j]);
        let y = f64::from(normals[j + 1]);
        let z = f64::from(normals[j + 2]);
        cxx += x * x;
        cxy += x * y;
        cxz += x * z;
        cyy += y * y;
        cyz += y * z;
        czz += z * z;
    }
    let (values, vectors) = eigen3([cxx, cxy, cxz, cxy, cyy, cyz, cxz, cyz, czz]);
    if !(values[1] > 1e-6 * n as f64) {
        return Ok(None);
    }
    Ok(Some(vectors[0]))
}

/// The best circle in the plane across a fixed axis, as a cylinder.
pub fn fit_cylinder_on_axis(
    p: &[f32],
    idx: &[u32],
    axis: [f64; 3],
) -> Result<Option<Cylinder>, ClipError> {
    validate(p.len(), idx)?;
    let n = idx.len();
    if n < 5 {
        return Ok(None);
    }
    let d = normalize_or_z(axis);
    let (u, v) = ortho_basis(d);
    let (mut mx, mut my, mut mz) = (0., 0., 0.);
    for &i in idx {
        let j = i as usize * 3;
        mx += f64::from(p[j]);
        my += f64::from(p[j + 1]);
        mz += f64::from(p[j + 2]);
    }
    let nf = n as f64;
    let (mx, my, mz) = (mx / nf, my / nf, mz / nf);
    let mut pu = Vec::new();
    let mut pv = Vec::new();
    pu.try_reserve_exact(n).map_err(|_| ClipError::Allocation)?;
    pv.try_reserve_exact(n).map_err(|_| ClipError::Allocation)?;
    for &i in idx {
        let j = i as usize * 3;
        let qx = f64::from(p[j]) - mx;
        let qy = f64::from(p[j + 1]) - my;
        let qz = f64::from(p[j + 2]) - mz;
        pu.push(qx * u[0] + qy * u[1] + qz * u[2]);
        pv.push(qx * v[0] + qy * v[1] + qz * v[2]);
    }
    let Some(c) = fit_circle_2d(&pu, &pv)? else {
        return Ok(None);
    };
    Ok(Some(Cylinder {
        px: mx + c.cu * u[0] + c.cv * v[0],
        py: my + c.cu * u[1] + c.cv * v[1],
        pz: mz + c.cu * u[2] + c.cv * v[2],
        ax: d[0],
        ay: d[1],
        az: d[2],
        r: c.r,
    }))
}

/// Damped Gauss-Newton refinement of all five degrees of freedom (the source
/// default `max_iter` is 60).
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn refine_cylinder_geometric(
    p: &[f32],
    idx: &[u32],
    init: Cylinder,
    max_iter: usize,
) -> Result<Cylinder, ClipError> {
    validate(p.len(), idx)?;
    let n = idx.len();
    if n < 6 {
        return Ok(init);
    }
    let mut c = init;
    let mut cost = rms(p, idx, &c)?;
    if !cost.is_finite() {
        return Ok(init);
    }
    let mut lambda = 1e-6;
    for _ in 0..max_iter {
        let (u, v) = ortho_basis([c.ax, c.ay, c.az]);
        let mut jtj = [0f64; 25];
        let mut jtf = [0f64; 5];
        let mut row = [0f64; 5];
        let mut m = 0usize;
        for &i in idx {
            let j = i as usize * 3;
            let r = radial(
                &c,
                f64::from(p[j]),
                f64::from(p[j + 1]),
                f64::from(p[j + 2]),
            );
            if !r.e.is_finite() {
                continue;
            }
            let su = r.wx * u[0] + r.wy * u[1] + r.wz * u[2];
            let sv = r.wx * v[0] + r.wy * v[1] + r.wz * v[2];
            row[0] = -su;
            row[1] = -sv;
            row[2] = -r.t * su;
            row[3] = -r.t * sv;
            row[4] = -1.;
            for a in 0..5 {
                jtf[a] += row[a] * r.e;
                for b in a..5 {
                    jtj[a * 5 + b] += row[a] * row[b];
                }
            }
            m += 1;
        }
        if m < 6 {
            break;
        }
        for a in 0..5 {
            for b in 0..a {
                jtj[a * 5 + b] = jtj[b * 5 + a];
            }
        }
        let mut applied = false;
        let mut converged = false;
        for _ in 0..6 {
            let mut mat = jtj;
            for a in 0..5 {
                mat[a * 5 + a] *= 1. + lambda;
            }
            let rhs = jtf.map(|x| -x);
            let step = solve_linear(5, &mut mat, &rhs, 5).map_err(|e| match e {
                SolveError::Allocation => ClipError::Allocation,
                SolveError::ShapeOrLimit => ClipError::InvalidInput,
            })?;
            let Some(step) = step else {
                lambda *= 10.;
                continue;
            };
            let axis = normalize_or_z([
                c.ax + step[2] * u[0] + step[3] * v[0],
                c.ay + step[2] * u[1] + step[3] * v[1],
                c.az + step[2] * u[2] + step[3] * v[2],
            ]);
            let cand = Cylinder {
                px: c.px + step[0] * u[0] + step[1] * v[0],
                py: c.py + step[0] * u[1] + step[1] * v[1],
                pz: c.pz + step[0] * u[2] + step[1] * v[2],
                ax: axis[0],
                ay: axis[1],
                az: axis[2],
                r: c.r + step[4],
            };
            if !(cand.r > 0.) || !cand.px.is_finite() {
                lambda *= 10.;
                continue;
            }
            let cand_cost = rms(p, idx, &cand)?;
            if cand_cost.is_finite() && cand_cost <= cost {
                converged = cost - cand_cost < 1e-12 * js_max(&[cost, cand.r]);
                c = cand;
                cost = cand_cost;
                lambda = js_max(&[lambda * 0.3, 1e-9]);
                applied = true;
                break;
            }
            lambda *= 10.;
        }
        if !applied || converged {
            break;
        }
    }
    Ok(c)
}

/// fitCylinderClipped result.
#[derive(Debug)]
pub struct ClippedCylinderFit {
    /// Last successful clipped cylinder.
    pub cylinder: Cylinder,
    /// RMS radial residual over the used points.
    pub sigma: f64,
    /// Peak-to-peak radial residual (cylindricity).
    pub span: f64,
    /// Most negative radial residual.
    pub min: f64,
    /// Most positive radial residual.
    pub max: f64,
    /// Ordered used point indices.
    pub used: Vec<u32>,
}

/// Gaussian best fit with the source's used-point clipping; each round refits the
/// circle on the previous round's axis, then refines geometrically.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn fit_cylinder_clipped(
    p: &[f32],
    idx: &[u32],
    init: Cylinder,
    k: f64,
    max_points: usize,
) -> Result<Option<ClippedCylinderFit>, ClipError> {
    let mut start = init;
    let r = clipped_refit(
        p,
        idx,
        k,
        max_points,
        |used| {
            let seeded =
                fit_cylinder_on_axis(p, used, [start.ax, start.ay, start.az])?.unwrap_or(start);
            let c = refine_cylinder_geometric(p, used, seeded, 60)?;
            if !(c.r > 0.) || !c.r.is_finite() {
                return Ok(None);
            }
            start = c;
            Ok(Some(c))
        },
        |c, x, y, z| radial(c, x, y, z).e,
    )?;
    Ok(r.map(|r| ClippedCylinderFit {
        cylinder: r.model,
        sigma: r.sigma,
        span: r.span,
        min: r.min,
        max: r.max,
        used: r.used,
    }))
}

/// Cylinder through two points with known normals.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn cylinder_from_2(p: &[f32], normals: &[f32], i0: u32, i1: u32) -> Option<Cylinder> {
    let a = i0 as usize * 3;
    let b = i1 as usize * 3;
    let f = |s: &[f32], k: usize| f64::from(s[k]);
    let n0 = [f(normals, a), f(normals, a + 1), f(normals, a + 2)];
    let n1 = [f(normals, b), f(normals, b + 1), f(normals, b + 2)];
    let axis_raw = cross(n0, n1);
    let len = norm3(axis_raw);
    if !(len > 0.08) {
        return None;
    }
    let d = normalize_or_z(axis_raw);
    let (u, v) = ortho_basis(d);
    let dot =
        |s: &[f32], k: usize, w: [f64; 3]| f(s, k) * w[0] + f(s, k + 1) * w[1] + f(s, k + 2) * w[2];
    let p0u = dot(p, a, u);
    let p0v = dot(p, a, v);
    let p1u = dot(p, b, u);
    let p1v = dot(p, b, v);
    let n0u = n0[0] * u[0] + n0[1] * u[1] + n0[2] * u[2];
    let n0v = n0[0] * v[0] + n0[1] * v[1] + n0[2] * v[2];
    let n1u = n1[0] * u[0] + n1[1] * u[1] + n1[2] * u[2];
    let n1v = n1[0] * v[0] + n1[1] * v[1] + n1[2] * v[2];
    let det = n0u * -n1v - -n1u * n0v;
    if det.abs() < 1e-9 {
        return None;
    }
    let du = p1u - p0u;
    let dv = p1v - p0v;
    let s = (du * -n1v - -n1u * dv) / det;
    let cu = p0u + s * n0u;
    let cv = p0v + s * n0v;
    let r = hypot(&[cu - p0u, cv - p0v]);
    if !(r > 0.) || !r.is_finite() {
        return None;
    }
    let along_u = cu - p0u;
    let along_v = cv - p0v;
    Some(Cylinder {
        px: f(p, a) + along_u * u[0] + along_v * v[0],
        py: f(p, a + 1) + along_u * u[1] + along_v * v[1],
        pz: f(p, a + 2) + along_u * u[2] + along_v * v[2],
        ax: d[0],
        ay: d[1],
        az: d[2],
        r,
    })
}

struct CylinderShape<'a> {
    p: &'a [f32],
    normals: &'a [f32],
    patch: &'a [u32],
    diag: f64,
}

impl CylinderShape<'_> {
    fn plausible(&self, c: &Cylinder) -> bool {
        c.r > self.diag * 0.01 && c.r <= self.diag * 60.
    }
}

impl ConsensusShape for CylinderShape<'_> {
    type Model = Cylinder;
    fn prepare(&mut self, diag: f64, _: &mut Mulberry32) -> Result<Option<Cylinder>, RansacError> {
        self.diag = diag;
        let host = |_| RansacError::ShapeFailure;
        let Some(axis) = axis_from_normals(self.normals, self.patch).map_err(host)? else {
            return Ok(None);
        };
        let from = fit_cylinder_on_axis(self.p, self.patch, axis).map_err(host)?;
        Ok(from.filter(|c| self.plausible(c)))
    }
    fn generate(&mut self, rand: &mut Mulberry32) -> Result<Option<Cylinder>, RansacError> {
        let n = self.patch.len() as f64;
        let i0 = self.patch[(rand.next_f64() * n) as usize];
        let i1 = self.patch[(rand.next_f64() * n) as usize];
        if i0 == i1 {
            return Ok(None);
        }
        Ok(cylinder_from_2(self.p, self.normals, i0, i1).filter(|c| self.plausible(c)))
    }
    fn residual(&self, m: &Cylinder, x: f64, y: f64, z: f64) -> f64 {
        radial(m, x, y, z).e
    }
}

/// ransacCylinder result.
#[derive(Debug)]
pub struct RansacCylinderResult {
    /// Clipped refined cylinder.
    pub cylinder: Cylinder,
    /// Ordered refined used points.
    pub inliers: Vec<u32>,
    /// Maximum of the refined RMS, the robust estimate and 1e-9.
    pub sigma: f64,
}

/// Robust cylinder on a local patch: normal-covariance axis plus two-point
/// candidates scored by median residual, refined on the consensus, refused when
/// a plane fits about as well. `normals` is packed like `positions`.
pub fn ransac_cylinder(
    p: &[f32],
    normals: &[f32],
    patch: &[u32],
    opts: RansacOptions,
    limits: RansacLimits,
) -> Result<Option<RansacCylinderResult>, RansacError> {
    if normals.len() != p.len() {
        return Err(RansacError::InvalidInput);
    }
    let mut shape = CylinderShape {
        p,
        normals,
        patch,
        diag: 0.,
    };
    let Some(core) = ransac_consensus(p, patch, opts, limits, &mut shape)? else {
        return Ok(None);
    };
    let host = |_| RansacError::ShapeFailure;
    let Some(refined) =
        fit_cylinder_clipped(p, &core.inliers, core.model, 3., limits.max_points).map_err(host)?
    else {
        return Ok(None);
    };
    let curved = rms(p, &refined.used, &refined.cylinder).map_err(host)?;
    if !curved_beyond_plane(p, &refined.used, curved).map_err(|_| RansacError::ShapeFailure)? {
        return Ok(None);
    }
    Ok(Some(RansacCylinderResult {
        cylinder: refined.cylinder,
        inliers: refined.used,
        sigma: js_max(&[refined.sigma, core.sigma_est, 1e-9]),
    }))
}
