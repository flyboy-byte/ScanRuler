// SPDX-License-Identifier: AGPL-3.0-only
//! Internal pure-Rust platform numeric primitives.

/// Scaled compensated three-component norm at the source Math.hypot boundary.
pub(crate) fn norm3(v: [f64; 3]) -> f64 {
    let abs = v.map(f64::abs);
    let max = abs[0].max(abs[1]).max(abs[2]);
    if max == 0. {
        0.
    } else if max.is_infinite() {
        f64::INFINITY
    } else if abs.iter().any(|x| x.is_nan()) {
        f64::NAN
    } else {
        let mut sum = 0.;
        let mut compensation = 0.;
        for x in abs {
            let scaled = x / max;
            let summand = scaled * scaled - compensation;
            let preliminary = sum + summand;
            compensation = (preliminary - sum) - summand;
            sum = preliminary;
        }
        sum.sqrt() * max
    }
}

/// Scaled compensated norm of any number of components at the source Math.hypot
/// boundary (the two-argument calls in the circle and cylinder kernels).
pub(crate) fn hypot(v: &[f64]) -> f64 {
    let mut max = 0f64;
    let mut nan = false;
    for &x in v {
        let a = x.abs();
        if a.is_infinite() {
            return f64::INFINITY;
        }
        if a.is_nan() {
            nan = true;
        } else if a > max {
            max = a;
        }
    }
    if nan {
        return f64::NAN;
    }
    if max == 0. {
        return 0.;
    }
    let mut sum = 0.;
    let mut compensation = 0.;
    for &x in v {
        let scaled = x.abs() / max;
        let summand = scaled * scaled - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}
