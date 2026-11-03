// SPDX-License-Identifier: AGPL-3.0-only
//! Exact owned Number.toFixed(3), used by measurement refusal messages.
use crate::js_number::number_string;
/// Fixed three-decimal primitive Number text, retaining source tie, sign,
/// subnormal, special-value and 1e21 threshold behavior. Output is at most26 ASCII
/// bytes. This is not a generic fraction-count or boxed-JS-object interface.
pub fn fixed3(value: f64) -> String {
    if !value.is_finite() || value.abs() >= 1e21 {
        return number_string(value);
    }
    // x<0 deliberately excludes negative zero; a nonzero negative x that rounds
    // to zero still keeps its minus sign, exactly as the source intrinsic.
    let sign = if value < 0. { "-" } else { "" };
    let bits = value.abs().to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let (mantissa, power) = if exponent == 0 {
        (bits & ((1u64 << 52) - 1), -1074 + 3)
    } else {
        (
            (bits & ((1u64 << 52) - 1)) | (1u64 << 52),
            exponent - 1023 - 52 + 3,
        )
    };
    // Multiplying exactly by1000 is mantissa*125 times2^(exponent+3).
    // Below1e21 this integer needs at most80 bits; no floating scaling or
    // decimal tie-to-even formatter can move the source midpoint decision.
    let numerator = u128::from(mantissa) * 125;
    let rounded = if power >= 0 {
        numerator << (power as u32)
    } else {
        let shift = (-power) as u32;
        if shift >= 128 {
            0
        } else {
            let whole = numerator >> shift;
            let remainder = numerator & ((1u128 << shift) - 1);
            whole + u128::from(remainder >= (1u128 << (shift - 1)))
        }
    };
    let digits = rounded.to_string();
    if digits.len() <= 3 {
        format!("{sign}0.{}{digits}", "0".repeat(3 - digits.len()))
    } else {
        let at = digits.len() - 3;
        format!("{sign}{}.{}", &digits[..at], &digits[at..])
    }
}
