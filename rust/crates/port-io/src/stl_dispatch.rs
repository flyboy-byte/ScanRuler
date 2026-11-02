// SPDX-License-Identifier: AGPL-3.0-only
//! Complete STL format dispatch and ASCII triangle extraction, ported from
//! `src/core/parsers/stl.ts` at `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

use crate::stl::{BinaryStlError, read_binary_triangles};

/// Caller-owned resource budgets, checked before exceeding them.
#[derive(Debug, Clone, Copy)]
pub struct StlLimits {
    /// Maximum input byte length, including headers and ignored trailing data.
    pub max_bytes: usize,
    /// Maximum number of output triangles.
    pub max_triangles: usize,
}

/// Original STL refusals and explicit Rust host resource errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlError {
    /// Neither a binary-sized buffer nor a `solid` text prefix.
    InvalidFile,
    /// No ASCII vertices or a vertex count not divisible by three.
    InvalidAscii,
    /// Input exceeds the caller's byte budget.
    InputLimit,
    /// Output exceeds the caller's triangle budget.
    TriangleLimit,
    /// Output or decoded-text storage could not be reserved.
    Allocation,
    /// Binary coordinates are incomplete or cannot be addressed.
    Truncated,
}

impl std::fmt::Display for StlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidFile => "Not a valid STL file.",
            Self::InvalidAscii => "Could not read triangles from ASCII STL.",
            Self::InputLimit => "STL input byte limit exceeded.",
            Self::TriangleLimit => "STL triangle limit exceeded.",
            Self::Allocation => "Could not allocate STL storage.",
            Self::Truncated => "STL coordinates are truncated.",
        })
    }
}

impl std::error::Error for StlError {}

impl From<BinaryStlError> for StlError {
    fn from(value: BinaryStlError) -> Self {
        match value {
            BinaryStlError::Truncated => Self::Truncated,
            BinaryStlError::TriangleLimit => Self::TriangleLimit,
            BinaryStlError::Allocation => Self::Allocation,
        }
    }
}

/// Read an STL into owned, unwelded XYZ triangle-soup coordinates.
///
/// Matches the original dispatch order: exact binary size first (even a `solid`
/// header), ASCII prefix second, then binary by complete record count, ignoring
/// a trailing partial record. ASCII matches unanchored lowercase `vertex` tokens,
/// ECMAScript whitespace and numeric prefixes, including non-finite values.
/// No live session is mutated. Resource-budget errors are explicit host boundaries.
pub fn read_stl(
    bytes: &[u8],
    limits: StlLimits,
    mut on_progress: impl FnMut(&str),
) -> Result<Vec<f32>, StlError> {
    if bytes.len() > limits.max_bytes {
        return Err(StlError::InputLimit);
    }
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        // Use the JS Number's range rather than overflowing wasm32 usize.
        if 84 + u64::from(count) * 50 == bytes.len() as u64 {
            return read_binary_triangles(bytes, count as usize, limits.max_triangles, on_progress)
                .map_err(Into::into);
        }
    }
    let head = decode_utf8(&bytes[..bytes.len().min(512)])?;
    if head.trim_start_matches(js_space).starts_with("solid") {
        return read_ascii(bytes, limits.max_triangles, &mut on_progress);
    }
    if bytes.len() >= 84 {
        return read_binary_triangles(
            bytes,
            (bytes.len() - 84) / 50,
            limits.max_triangles,
            on_progress,
        )
        .map_err(Into::into);
    }
    Err(StlError::InvalidFile)
}

// TextDecoder's default UTF-8 replacement mode and initial BOM handling. Reserve
// each append fallibly, avoiding a hidden infallible lossy-string allocation.
pub(crate) fn decode_utf8(mut bytes: &[u8]) -> Result<String, StlError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        bytes = &bytes[3..];
    }
    let mut text = String::new();
    while !bytes.is_empty() {
        match std::str::from_utf8(bytes) {
            Ok(valid) => {
                text.try_reserve(valid.len())
                    .map_err(|_| StlError::Allocation)?;
                text.push_str(valid);
                break;
            }
            Err(error) => {
                let valid_len = error.valid_up_to();
                // The prefix is guaranteed valid by Utf8Error.
                let valid =
                    std::str::from_utf8(&bytes[..valid_len]).map_err(|_| StlError::InvalidAscii)?;
                text.try_reserve(valid_len + 3)
                    .map_err(|_| StlError::Allocation)?;
                text.push_str(valid);
                text.push('\u{fffd}');
                bytes = &bytes[valid_len + error.error_len().unwrap_or(bytes.len() - valid_len)..];
            }
        }
    }
    Ok(text)
}

// ECMAScript \s is not Rust char::is_whitespace (notably U+0085 and U+FEFF).
pub(crate) fn js_space(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

fn vertex_match(text: &str, mut at: usize) -> Option<([&str; 3], usize)> {
    let mut tokens = [""; 3];
    for token in &mut tokens {
        let rest = &text[at..];
        let begin = rest.len() - rest.trim_start_matches(js_space).len();
        if begin == 0 {
            return None;
        }
        at += begin;
        let end = text[at..].find(js_space).map_or(text.len(), |n| at + n);
        if at == end {
            return None;
        }
        *token = &text[at..end];
        at = end;
    }
    Some((tokens, at))
}

fn read_ascii(
    bytes: &[u8],
    max_triangles: usize,
    on_progress: &mut impl FnMut(&str),
) -> Result<Vec<f32>, StlError> {
    on_progress("Reading STL…");
    let text = decode_utf8(bytes)?;
    let mut positions = Vec::new();
    let mut search = 0;
    while let Some(found) = text[search..].find("vertex") {
        let after_keyword = search + found + 6;
        if let Some((tokens, end)) = vertex_match(&text, after_keyword) {
            if positions.len().saturating_add(3) > max_triangles.saturating_mul(9) {
                return Err(StlError::TriangleLimit);
            }
            positions.try_reserve(3).map_err(|_| StlError::Allocation)?;
            for token in tokens {
                positions.push(parse_float_prefix(token) as f32);
            }
            search = end;
        } else {
            // A failed regex attempt must still search inside its attempted tokens.
            search = after_keyword;
        }
    }
    if positions.is_empty() || positions.len() % 9 != 0 {
        return Err(StlError::InvalidAscii);
    }
    Ok(positions)
}

// parseFloat's decimal/Infinity prefix grammar, not Rust's whole-string parse.
// Parse via f64 before narrowing: the original stores JS Numbers into Float32Array.
pub(crate) fn parse_float_prefix(token: &str) -> f64 {
    let bytes = token.as_bytes();
    let mut at = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if token[at..].starts_with("Infinity") {
        return if bytes.first() == Some(&b'-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let mut digits = 0;
    while bytes.get(at).is_some_and(u8::is_ascii_digit) {
        at += 1;
        digits += 1;
    }
    if bytes.get(at) == Some(&b'.') {
        at += 1;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return f64::NAN;
    }
    let mantissa_end = at;
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        let exponent_start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        if at == exponent_start {
            at = mantissa_end;
        }
    }
    token[..at].parse::<f64>().unwrap_or(f64::NAN)
}
