// SPDX-License-Identifier: AGPL-3.0-only
//! Complete frozen bounds.ts extent and bounds operations (2cd89c6).

/// Packed coordinate/count/selection contract violation.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidBoundsInput;
/// Axis extents, including inside-out infinities for an empty selection.
#[derive(Debug, Clone, Copy)]
pub struct Extent {
    /// Minimum coordinates.
    pub min: [f64; 3],
    /// Maximum coordinates.
    pub max: [f64; 3],
}
impl Extent {
    fn empty() -> Self {
        Self {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
        }
    }
    fn include(&mut self, xyz: &[f32]) {
        for (k, &x) in xyz.iter().enumerate() {
            let x = f64::from(x);
            if x < self.min[k] {
                self.min[k] = x;
            }
            if x > self.max[k] {
                self.max[k] = x;
            }
        }
    }
}
/// Least/greatest coordinates of the first count points. The original default
/// count is positions.len()/3; Rust makes that count explicit.
pub fn extent_of(positions: &[f32], count: usize) -> Result<Extent, InvalidBoundsInput> {
    if !positions.len().is_multiple_of(3) || count > positions.len() / 3 {
        return Err(InvalidBoundsInput);
    }
    let mut e = Extent::empty();
    for xyz in positions[..count * 3].as_chunks::<3>().0 {
        e.include(xyz);
    }
    Ok(e)
}
/// Extent of exactly the named points, retaining repetition and source comparison
/// semantics (not min/max intrinsics that could change signed-zero behavior).
pub fn extent_of_indexed(positions: &[f32], idx: &[u32]) -> Result<Extent, InvalidBoundsInput> {
    if !positions.len().is_multiple_of(3) || idx.iter().any(|&i| i as usize >= positions.len() / 3)
    {
        return Err(InvalidBoundsInput);
    }
    let mut e = Extent::empty();
    for &i in idx {
        let j = i as usize * 3;
        e.include(&positions[j..j + 3]);
    }
    Ok(e)
}
/// Axis-aligned mesh box and original viewport center/radius meaning.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    /// Minimum coordinates (zero for no points).
    pub min: [f64; 3],
    /// Maximum coordinates (zero for no points).
    pub max: [f64; 3],
    /// Midpoint of extrema.
    pub center: [f64; 3],
    /// Half diagonal, floored at 1e-4 as in the source.
    pub radius: f64,
}
/// Full bounds of the first count points, preserving the distinct empty result.
pub fn bounds_of(positions: &[f32], count: usize) -> Result<Bounds, InvalidBoundsInput> {
    let Extent { min, max } = extent_of(positions, count)?;
    if count == 0 {
        return Ok(Bounds {
            min: [0.; 3],
            max: [0.; 3],
            center: [0.; 3],
            radius: 1e-4,
        });
    }
    let d = crate::numeric::norm3([max[0] - min[0], max[1] - min[1], max[2] - min[2]]);
    Ok(Bounds {
        min,
        max,
        center: [
            (min[0] + max[0]) / 2.,
            (min[1] + max[1]) / 2.,
            (min[2] + max[2]) / 2.,
        ],
        radius: if d.is_nan() {
            f64::NAN
        } else {
            (d / 2.).max(1e-4)
        },
    })
}
