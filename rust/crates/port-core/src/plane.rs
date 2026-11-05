// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen plane.ts::fitPlaneTLS/planeResidual and minimal linalg dependencies.

/// Analytic plane with signed distance n dot p - d.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal, retaining the source eigenvector direction.
    pub normal: [f64; 3],
    /// Offset along the normal.
    pub d: f64,
}
impl Plane {
    /// Source signed residual, with unchanged addition order.
    pub fn residual(&self, x: f64, y: f64, z: f64) -> f64 {
        self.normal[0] * x + self.normal[1] * y + self.normal[2] * z - self.d
    }
}
/// Invalid packed coordinates/selection, before computation begins.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPlaneInput;

// Pure-Rust scaled compensated norm at the source Math.hypot boundary.
pub(crate) fn normalize_or_z(v: [f64; 3]) -> [f64; 3] {
    let len = crate::numeric::norm3(v);
    if len.partial_cmp(&1e-20) != Some(std::cmp::Ordering::Greater) {
        return [0., 0., 1.];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}
pub(crate) fn eigen3(mut a: [f64; 9]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut v = [1., 0., 0., 0., 1., 0., 0., 0., 1.];
    for _ in 0..32 {
        let off = a[1].abs() + a[2].abs() + a[5].abs();
        let diag = a[0].abs() + a[4].abs() + a[8].abs();
        if off <= 1e-14 * diag || off == 0. {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            let apq = a[p * 3 + q];
            if apq == 0. {
                continue;
            }
            let theta = (a[q * 3 + q] - a[p * 3 + p]) / (2. * apq);
            let sign = if theta >= 0. { 1. } else { -1. };
            let t = sign / (theta.abs() + (theta * theta + 1.).sqrt());
            let c = 1. / (t * t + 1.).sqrt();
            let s = t * c;
            for k in 0..3 {
                let akp = a[k * 3 + p];
                let akq = a[k * 3 + q];
                a[k * 3 + p] = c * akp - s * akq;
                a[k * 3 + q] = s * akp + c * akq;
            }
            for k in 0..3 {
                let apk = a[p * 3 + k];
                let aqk = a[q * 3 + k];
                a[p * 3 + k] = c * apk - s * aqk;
                a[q * 3 + k] = s * apk + c * aqk;
            }
            for k in 0..3 {
                let vkp = v[k * 3 + p];
                let vkq = v[k * 3 + q];
                v[k * 3 + p] = c * vkp - s * vkq;
                v[k * 3 + q] = s * vkp + c * vkq;
            }
        }
    }
    let mut order = [0usize, 1, 2];
    order.sort_by(|&i, &j| {
        (a[i * 3 + i] - a[j * 3 + j])
            .partial_cmp(&0.)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (
        order.map(|i| a[i * 3 + i]),
        order.map(|i| normalize_or_z([v[i], v[3 + i], v[6 + i]])),
    )
}
/// Fit a total-least-squares plane to the selected f32 points using f64 compute.
/// Complete fixed-selection body only: no robust starts, clipping or region growth.
/// Returns None for the source insufficient/collinear/invalid-normal refusals.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Exact source NaN-sensitive guards.
pub fn fit_plane_tls(positions: &[f32], idx: &[u32]) -> Result<Option<Plane>, InvalidPlaneInput> {
    if !positions.len().is_multiple_of(3) || idx.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(InvalidPlaneInput);
    }
    let n = idx.len();
    if n < 3 {
        return Ok(None);
    }
    let mut mean = [0.; 3];
    for &i in idx {
        let j = i as usize * 3;
        for k in 0..3 {
            mean[k] += f64::from(positions[j + k]);
        }
    }
    for m in &mut mean {
        *m /= n as f64;
    }
    let [mx, my, mz] = mean;
    let (mut cxx, mut cxy, mut cxz, mut cyy, mut cyz, mut czz) = (0., 0., 0., 0., 0., 0.);
    for &i in idx {
        let j = i as usize * 3;
        let x = f64::from(positions[j]) - mx;
        let y = f64::from(positions[j + 1]) - my;
        let z = f64::from(positions[j + 2]) - mz;
        cxx += x * x;
        cxy += x * y;
        cxz += x * z;
        cyy += y * y;
        cyz += y * z;
        czz += z * z;
    }
    let (values, vectors) = eigen3([cxx, cxy, cxz, cxy, cyy, cyz, cxz, cyz, czz]);
    if !(values[1] > 1e-12 * values[2]) || !(values[2] > 0.) {
        return Ok(None);
    }
    let [nx, ny, nz] = vectors[0];
    if !nx.is_finite() {
        return Ok(None);
    }
    Ok(Some(Plane {
        normal: [nx, ny, nz],
        d: nx * mx + ny * my + nz * mz,
    }))
}
