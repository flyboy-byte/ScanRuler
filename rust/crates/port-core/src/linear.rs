// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen linalg.ts::choleskySolve and solveLinear (2cd89c6).

/// Original Cholesky pivot threshold.
pub const DEFAULT_CHOLESKY_TINY: f64 = 1e-12;
/// Host errors distinct from a source numerical refusal (`Ok(None)`).
#[derive(Debug, PartialEq, Eq)]
pub enum SolveError {
    /// Inconsistent matrix/vector shape, oversized dimension or size overflow.
    ShapeOrLimit,
    /// Work storage could not be reserved.
    Allocation,
}
fn shape(n: usize, a: &[f64], b: &[f64], limit: usize) -> Result<usize, SolveError> {
    let size = n.checked_mul(n).ok_or(SolveError::ShapeOrLimit)?;
    if n > limit || a.len() != size || b.len() != n {
        return Err(SolveError::ShapeOrLimit);
    }
    Ok(size)
}
fn zeros(n: usize) -> Result<Vec<f64>, SolveError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| SolveError::Allocation)?;
    v.resize(n, 0.);
    Ok(v)
}
/// Cholesky solve preserving the lower-triangle read order and source pivot guard.
/// Does not mutate either input. `tiny` defaults to DEFAULT_CHOLESKY_TINY in the
/// original; Rust makes it explicit. Non-positive pivots return `Ok(None)`.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Original NaN-sensitive guard.
pub fn cholesky_solve(
    n: usize,
    a: &[f64],
    b: &[f64],
    tiny: f64,
    max_dimension: usize,
) -> Result<Option<Vec<f64>>, SolveError> {
    let size = shape(n, a, b, max_dimension)?;
    let mut l = zeros(size)?;
    for j in 0..n {
        let rj = j * n;
        let mut d = a[rj + j];
        for k in 0..j {
            d -= l[rj + k] * l[rj + k];
        }
        if !(d > tiny) {
            return Ok(None);
        }
        let ljj = d.sqrt();
        l[rj + j] = ljj;
        for i in j + 1..n {
            let ri = i * n;
            let mut s = a[ri + j];
            for k in 0..j {
                s -= l[ri + k] * l[rj + k];
            }
            l[ri + j] = s / ljj;
        }
    }
    let mut y = zeros(n)?;
    for i in 0..n {
        let ri = i * n;
        let mut s = b[i];
        for k in 0..i {
            s -= l[ri + k] * y[k];
        }
        y[i] = s / l[ri + i];
    }
    let mut x = zeros(n)?;
    for i in (0..n).rev() {
        let mut s = y[i];
        for k in i + 1..n {
            s -= l[k * n + i] * x[k];
        }
        x[i] = s / l[i * n + i];
    }
    Ok(Some(x))
}
/// Gaussian elimination with source partial-pivot tie order and in-place matrix
/// mutations, including mutations already made before a numerical refusal.
/// The right hand side is copied and never changed.
pub fn solve_linear(
    n: usize,
    a: &mut [f64],
    b: &[f64],
    max_dimension: usize,
) -> Result<Option<Vec<f64>>, SolveError> {
    shape(n, a, b, max_dimension)?;
    let mut max_abs = 0.0f64;
    for &v in a.iter() {
        max_abs = if v.is_nan() {
            f64::NAN
        } else if max_abs.is_nan() {
            max_abs
        } else {
            max_abs.max(v.abs())
        };
    }
    let tiny = if max_abs.is_nan() {
        max_abs
    } else {
        max_abs.max(1.)
    } * 1e-13;
    let mut x = zeros(n)?;
    x.copy_from_slice(b);
    for col in 0..n {
        let mut pivot_row = col;
        let mut best = a[col * n + col].abs();
        for r in col + 1..n {
            let v = a[r * n + col].abs();
            if v > best {
                best = v;
                pivot_row = r;
            }
        }
        if best < tiny {
            return Ok(None);
        }
        if pivot_row != col {
            for c in col..n {
                a.swap(col * n + c, pivot_row * n + c);
            }
            x.swap(col, pivot_row);
        }
        let piv = a[col * n + col];
        for r in col + 1..n {
            let f = a[r * n + col] / piv;
            if f == 0. {
                continue;
            }
            for c in col..n {
                a[r * n + c] -= f * a[col * n + c];
            }
            x[r] -= f * x[col];
        }
    }
    for row in (0..n).rev() {
        let mut s = x[row];
        for c in row + 1..n {
            s -= a[row * n + c] * x[c];
        }
        x[row] = s / a[row * n + row];
    }
    Ok(Some(x))
}
