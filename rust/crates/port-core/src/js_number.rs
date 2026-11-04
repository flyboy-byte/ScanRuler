// SPDX-License-Identifier: AGPL-3.0-only
//! Shared owned ECMAScript Number text used by source-compatible error messages.
/// Render a number with source exponent thresholds, special values and signed zero.
pub fn number_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x == 0.0 {
        return "0".into(); // also -0
    }
    if x.is_infinite() {
        return if x < 0.0 { "-Infinity" } else { "Infinity" }.into();
    }
    let sign = if x < 0.0 { "-" } else { "" };
    // Shortest round-trip digits, as `d[.ddd]e<exp>`.
    let sci = format!("{:e}", x.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let mut digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mut decimal_power = exp.parse::<i32>().unwrap_or(0) + 1 - digits.len() as i32;
    // Rust's shortest formatter can choose the odd coefficient at an exact
    // decimal midpoint. The source chooses the even coefficient. Establish
    // the midpoint with integers; floating subtraction would lose the tie.
    if let Ok(coefficient) = digits.parse::<u128>()
        && coefficient % 2 == 1
    {
        let twice = coefficient * 2;
        let corrected = if decimal_midpoint(x.abs(), twice - 1, decimal_power) {
            coefficient - 1
        } else if decimal_midpoint(x.abs(), twice + 1, decimal_power) {
            coefficient + 1
        } else {
            coefficient
        };
        digits = corrected.to_string();
    }
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
        decimal_power += 1;
    }
    let k = digits.len() as i32;
    let n = decimal_power + k;
    let body = if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let e = n - 1;
        let es = if e < 0 {
            format!("-{}", -e)
        } else {
            format!("+{e}")
        };
        if k == 1 {
            format!("{digits}e{es}")
        } else {
            format!("{}.{}e{es}", &digits[..1], &digits[1..])
        }
    };
    format!("{sign}{body}")
}

// Test x == midpoint * 10^decimal_power / 2 exactly. Both midpoint and
// the normalized binary mantissa are odd. Large powers of five cannot
// divide/fit the at-most-17-digit coefficient / 53-bit mantissa, so no
// big integer, floating decimal scaling or external formatter is needed.
fn decimal_midpoint(x: f64, midpoint: u128, decimal_power: i32) -> bool {
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let (mantissa, power) = if exponent == 0 {
        (bits & ((1u64 << 52) - 1), -1074)
    } else {
        (
            (bits & ((1u64 << 52) - 1)) | (1u64 << 52),
            exponent - 1023 - 52,
        )
    };
    let zeros = mantissa.trailing_zeros();
    let odd = u128::from(mantissa >> zeros);
    let binary_power = power + zeros as i32;
    if decimal_power >= 0 {
        if decimal_power > 23 || binary_power != decimal_power - 1 {
            return false;
        }
        midpoint * 5u128.pow(decimal_power as u32) == odd
    } else {
        let places = (-decimal_power) as u32;
        if places > 27 || binary_power != -(places as i32) - 1 {
            return false;
        }
        let five = 5u128.pow(places);
        midpoint.is_multiple_of(five) && midpoint / five == odd
    }
}
