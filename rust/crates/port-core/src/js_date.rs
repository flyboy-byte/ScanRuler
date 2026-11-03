// SPDX-License-Identifier: AGPL-3.0-only
//! Own bounded Gregorian UTC conversion for Date millisecond inputs.
//! No platform calendar, foreign runtime or calendar dependency is used.

/// Invalid source time or distinct bounded-host allocation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    /// Non-finite input or magnitude beyond the Date TimeClip boundary.
    InvalidTime,
    /// Fallible reservation for the at-most-27-unit timestamp failed.
    Allocation,
}
impl DateError {
    /// Original Node22 Date.toISOString exception fields, if a source refusal.
    /// Host allocation failure has no substituted source error.
    pub const fn source_error(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::InvalidTime => Some(("RangeError", "Invalid time value")),
            Self::Allocation => None,
        }
    }
}
fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
fn digits(out: &mut Vec<u16>, mut value: u32, width: usize) {
    let mut buffer = [48; 6];
    for i in (0..width).rev() {
        buffer[i] += (value % 10) as u16;
        value /= 10;
    }
    out.extend_from_slice(&buffer[..width]);
}
/// Date TimeClip followed by exact UTC ISO text, including signed extended years.
/// Fractional milliseconds truncate toward zero; negative-day division floors.
/// After clipping, conversion needs at most400 year and12 month iterations.
pub fn utc_iso_millis(ms: f64) -> Result<Vec<u16>, DateError> {
    const LIMIT: f64 = 8_640_000_000_000_000.0;
    if !ms.is_finite() || ms.abs() > LIMIT {
        return Err(DateError::InvalidTime);
    }
    let ms = ms.trunc() as i64;
    let days = ms.div_euclid(86_400_000);
    let time = ms.rem_euclid(86_400_000);
    // Days from astronomical year0 to1970: ordinary days plus leap days.
    const EPOCH_DAYS: i64 = 1970 * 365 + (1970 + 3) / 4 - (1970 + 99) / 100 + (1970 + 399) / 400;
    let days = days + EPOCH_DAYS;
    let era = days.div_euclid(146_097);
    let mut left = days.rem_euclid(146_097);
    let mut year = era * 400;
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if left < length {
            break;
        }
        left -= length;
        year += 1;
    }
    let mut month = 1;
    for length in [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ] {
        if left < length {
            break;
        }
        left -= length;
        month += 1;
    }
    let mut out = Vec::new();
    out.try_reserve_exact(27)
        .map_err(|_| DateError::Allocation)?;
    if (0..=9999).contains(&year) {
        digits(&mut out, year as u32, 4);
    } else {
        out.push(u16::from(if year < 0 { b'-' } else { b'+' }));
        digits(&mut out, year.unsigned_abs() as u32, 6);
    }
    out.push(u16::from(b'-'));
    digits(&mut out, month, 2);
    out.push(u16::from(b'-'));
    digits(&mut out, (left + 1) as u32, 2);
    out.push(u16::from(b'T'));
    digits(&mut out, (time / 3_600_000) as u32, 2);
    out.push(u16::from(b':'));
    digits(&mut out, (time / 60_000 % 60) as u32, 2);
    out.push(u16::from(b':'));
    digits(&mut out, (time / 1000 % 60) as u32, 2);
    out.push(u16::from(b'.'));
    digits(&mut out, (time % 1000) as u32, 3);
    out.push(u16::from(b'Z'));
    Ok(out)
}
