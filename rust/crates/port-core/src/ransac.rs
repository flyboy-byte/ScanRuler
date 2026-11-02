// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen ransac.ts sampling/consensus and plane.ts::ransacPlane.
use crate::{bounds::extent_of_indexed, clip::fit_plane_clipped, plane::Plane};
/// Deterministic source mulberry32 state, with explicit modulo-2^32 operations.
#[derive(Debug, Clone)]
pub struct Mulberry32 {
    state: u32,
}
impl Mulberry32 {
    /// Start the same sequence as the source unsigned seed.
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    /// Next source draw in [0,1), preserving all imul/shift/wrapping semantics.
    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b79f5);
        let a = self.state;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4294967296.
    }
}
/// Source options, with original defaults.
#[derive(Debug, Clone, Copy)]
pub struct RansacOptions {
    /// Number of generated candidates, including after a perfect score.
    pub iterations: usize,
    /// Deterministic source PRNG seed.
    pub seed: u32,
}
impl Default for RansacOptions {
    fn default() -> Self {
        Self {
            iterations: 256,
            seed: 0x5eed,
        }
    }
}
/// Explicit caller work budgets.
#[derive(Debug, Clone, Copy)]
pub struct RansacLimits {
    /// Maximum patch selection length.
    pub max_points: usize,
    /// Maximum candidate iterations.
    pub max_iterations: usize,
}
/// Rust host/storage errors distinct from source numerical refusal.
#[derive(Debug, PartialEq, Eq)]
pub enum RansacError {
    /// Invalid packed geometry/selection or signed source indexing domain.
    InvalidInput,
    /// Caller work budget exceeded.
    Limit,
    /// Scratch/output reservation failed.
    Allocation,
    /// A candidate/refinement could not satisfy its host contract.
    ShapeFailure,
}
/// Typed numerical model adapter for the shared consensus body. Residuals must
/// represent the shape's signed distance; model values are numerical objects.
pub trait ConsensusShape {
    /// Owned candidate model.
    type Model;
    /// Source shape setup and optional deterministic initial candidate, once.
    fn prepare(
        &mut self,
        diag: f64,
        rand: &mut Mulberry32,
    ) -> Result<Option<Self::Model>, RansacError>;
    /// Generate the next candidate, retaining original random draw order.
    fn generate(&mut self, rand: &mut Mulberry32) -> Result<Option<Self::Model>, RansacError>;
    /// Signed candidate residual (evaluated twice over the full patch as in source).
    fn residual(&self, model: &Self::Model, x: f64, y: f64, z: f64) -> f64;
}
/// Candidate consensus before shape-specific refinement.
#[derive(Debug)]
pub struct RansacConsensus<M> {
    /// First candidate attaining the best median score.
    pub model: M,
    /// Full ordered consensus point identities.
    pub inliers: Vec<u32>,
    /// Source 1.4826 times median absolute residual.
    pub sigma_est: f64,
}
fn storage<T>(n: usize) -> Result<Vec<T>, RansacError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| RansacError::Allocation)?;
    Ok(v)
}
fn median<S: ConsensusShape>(
    shape: &S,
    model: &S::Model,
    p: &[f32],
    subset: &[u32],
    scratch: &mut [f64],
) -> f64 {
    for (i, &v) in subset.iter().enumerate() {
        let j = v as usize * 3;
        scratch[i] = shape
            .residual(
                model,
                f64::from(p[j]),
                f64::from(p[j + 1]),
                f64::from(p[j + 2]),
            )
            .abs();
    }
    scratch.sort_unstable_by(f64::total_cmp);
    scratch[subset.len() >> 1]
}
/// Complete shared LMedS body: strided scoring, first-best ties, fixed candidate
/// work, source thresholds and two-pass consensus construction. No early success exit.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn ransac_consensus<S: ConsensusShape>(
    positions: &[f32],
    patch: &[u32],
    opts: RansacOptions,
    limits: RansacLimits,
    shape: &mut S,
) -> Result<Option<RansacConsensus<S::Model>>, RansacError> {
    let n = patch.len();
    if !positions.len().is_multiple_of(3)
        || n > i32::MAX as usize
        || patch.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(RansacError::InvalidInput);
    }
    if n > limits.max_points || opts.iterations > limits.max_iterations {
        return Err(RansacError::Limit);
    }
    if n < 30 {
        return Ok(None);
    }
    let mut rand = Mulberry32::new(opts.seed);
    let extent = extent_of_indexed(positions, patch).map_err(|_| RansacError::InvalidInput)?;
    let dx = extent.max[0] - extent.min[0];
    let dy = extent.max[1] - extent.min[1];
    let dz = extent.max[2] - extent.min[2];
    let diag = (dx * dx + dy * dy + dz * dz).sqrt();
    if !(diag > 0.) {
        return Ok(None);
    }
    let initial = shape.prepare(diag, &mut rand)?;
    let score_n = n.min(512);
    let stride = n as f64 / score_n as f64;
    let mut subset = storage(score_n)?;
    for i in 0..score_n {
        subset.push(patch[(i as f64 * stride).floor() as usize]);
    }
    let mut resid = storage(score_n)?;
    resid.resize(score_n, 0.);
    let mut best_median = f64::INFINITY;
    let mut best = None;
    if let Some(initial) = initial {
        let med = median(shape, &initial, positions, &subset, &mut resid);
        if med < best_median {
            best_median = med;
            best = Some(initial);
        }
    }
    for _ in 0..opts.iterations {
        let Some(model) = shape.generate(&mut rand)? else {
            continue;
        };
        let med = median(shape, &model, positions, &subset, &mut resid);
        if med < best_median {
            best_median = med;
            best = Some(model);
        }
    }
    let Some(model) = best else {
        return Ok(None);
    };
    let sigma_est = 1.4826 * best_median;
    let thr = (3. * sigma_est).max(diag * 1e-5);
    let residual = |i: u32| {
        let j = i as usize * 3;
        shape.residual(
            &model,
            f64::from(positions[j]),
            f64::from(positions[j + 1]),
            f64::from(positions[j + 2]),
        )
    };
    let count = patch.iter().filter(|&&i| residual(i).abs() <= thr).count();
    if count < 30 {
        return Ok(None);
    }
    let mut inliers = storage(count)?;
    inliers.resize(count, 0);
    let mut w = 0;
    for &i in patch {
        if residual(i).abs() <= thr {
            if w < count {
                inliers[w] = i;
            }
            w += 1;
        }
    }
    Ok(Some(RansacConsensus {
        model,
        inliers,
        sigma_est,
    }))
}
struct PlaneShape<'a> {
    positions: &'a [f32],
    patch: &'a [u32],
    min_area: f64,
}
impl ConsensusShape for PlaneShape<'_> {
    type Model = Plane;
    fn prepare(&mut self, diag: f64, _: &mut Mulberry32) -> Result<Option<Plane>, RansacError> {
        self.min_area = diag * diag * 1e-4;
        Ok(None)
    }
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn generate(&mut self, rand: &mut Mulberry32) -> Result<Option<Plane>, RansacError> {
        let n = self.patch.len();
        let i0 = self.patch[(rand.next_f64() * n as f64) as usize];
        let i1 = self.patch[(rand.next_f64() * n as f64) as usize];
        let i2 = self.patch[(rand.next_f64() * n as f64) as usize];
        if i0 == i1 || i0 == i2 || i1 == i2 {
            return Ok(None);
        }
        let a = i0 as usize * 3;
        let b = i1 as usize * 3;
        let c = i2 as usize * 3;
        let p = self.positions;
        let ux = f64::from(p[b]) - f64::from(p[a]);
        let uy = f64::from(p[b + 1]) - f64::from(p[a + 1]);
        let uz = f64::from(p[b + 2]) - f64::from(p[a + 2]);
        let vx = f64::from(p[c]) - f64::from(p[a]);
        let vy = f64::from(p[c + 1]) - f64::from(p[a + 1]);
        let vz = f64::from(p[c + 2]) - f64::from(p[a + 2]);
        let mut nx = uy * vz - uz * vy;
        let mut ny = uz * vx - ux * vz;
        let mut nz = ux * vy - uy * vx;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if !(len > self.min_area) {
            return Ok(None);
        }
        nx /= len;
        ny /= len;
        nz /= len;
        Ok(Some(Plane {
            normal: [nx, ny, nz],
            d: nx * f64::from(p[a]) + ny * f64::from(p[a + 1]) + nz * f64::from(p[a + 2]),
        }))
    }
    fn residual(&self, m: &Plane, x: f64, y: f64, z: f64) -> f64 {
        m.residual(x, y, z)
    }
}
/// Refined robust plane result, retaining the source's post-clipping inliers.
#[derive(Debug)]
pub struct RansacPlaneResult {
    /// Clipped consensus plane.
    pub plane: Plane,
    /// Ordered refined used-point IDs.
    pub inliers: Vec<u32>,
    /// Maximum of refined RMS, robust estimate and source 1e-9 floor.
    pub sigma: f64,
}
/// Complete robust plane wrapper, including three-sigma clipped refinement.
pub fn ransac_plane(
    positions: &[f32],
    patch: &[u32],
    opts: RansacOptions,
    limits: RansacLimits,
) -> Result<Option<RansacPlaneResult>, RansacError> {
    let mut shape = PlaneShape {
        positions,
        patch,
        min_area: 0.,
    };
    let Some(core) = ransac_consensus(positions, patch, opts, limits, &mut shape)? else {
        return Ok(None);
    };
    let Some(refined) = fit_plane_clipped(positions, &core.inliers, 3., limits.max_points)
        .map_err(|_| RansacError::ShapeFailure)?
    else {
        return Ok(None);
    };
    let sigma = if refined.sigma.is_nan() || core.sigma_est.is_nan() {
        f64::NAN
    } else {
        refined.sigma.max(core.sigma_est).max(1e-9)
    };
    Ok(Some(RansacPlaneResult {
        plane: refined.plane,
        inliers: refined.used,
        sigma,
    }))
}
