// SPDX-License-Identifier: AGPL-3.0-only
//! Lossless UTF-16 lowercase using the toolchain's full contextual Unicode rules.
/// Unicode database used by this implementation; checked against the oracle.
pub const UNICODE_VERSION: (u8, u8, u8) = std::char::UNICODE_VERSION;
/// Bounded-host failure, distinct from an original callback exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringError {
    /// Toolchain Unicode data changed; a new profile must be reviewed first.
    UnicodeVersion,
    /// Input exceeds the explicit UTF-16 work bound.
    InputLimit,
    /// Complete output would exceed the explicit UTF-16 storage bound.
    OutputLimit,
    /// Fallible span/output reservation failed.
    Allocation,
}
fn flush(span: &mut String, out: &mut Vec<u16>, max: usize) -> Result<(), StringError> {
    // Standard Unicode lowercase retains final-sigma context within the valid
    // run. Its temporary is bounded by the checked input; it is not process-wide
    // OOM isolation. Unpaired units are non-cased/non-ignorable run boundaries.
    let lower = span.to_lowercase();
    let count = lower.encode_utf16().count();
    if out.len().checked_add(count).is_none_or(|n| n > max) {
        return Err(StringError::OutputLimit);
    }
    out.try_reserve(count)
        .map_err(|_| StringError::Allocation)?;
    out.extend(lower.encode_utf16());
    span.clear();
    Ok(())
}
/// Full default Unicode lowercase with every unpaired UTF-16 unit preserved.
/// Invalid units break case context; they are never replaced with U+FFFD. Input
/// and complete output are bounded; no partially transformed value escapes.
pub fn lowercase(
    input: &[u16],
    max_input: usize,
    max_output: usize,
) -> Result<Vec<u16>, StringError> {
    if UNICODE_VERSION != (17, 0, 0) {
        return Err(StringError::UnicodeVersion);
    }
    if input.len() > max_input {
        return Err(StringError::InputLimit);
    }
    let bytes = input.len().checked_mul(3).ok_or(StringError::InputLimit)?;
    let mut span = String::new();
    span.try_reserve(bytes)
        .map_err(|_| StringError::Allocation)?;
    let mut out = Vec::new();
    for decoded in std::char::decode_utf16(input.iter().copied()) {
        match decoded {
            Ok(c) => span.push(c),
            Err(e) => {
                flush(&mut span, &mut out, max_output)?;
                if out.len() >= max_output {
                    return Err(StringError::OutputLimit);
                }
                out.try_reserve(1).map_err(|_| StringError::Allocation)?;
                out.push(e.unpaired_surrogate())
            }
        }
    }
    flush(&mut span, &mut out, max_output)?;
    Ok(out)
}
