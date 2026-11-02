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
