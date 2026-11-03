// SPDX-License-Identifier: AGPL-3.0-only
//! Exact guarded/interrupt Agent-label expression over owned text/provider data.
use port_core::js_string::{self, StringError};
/// Original custom-label provider, bound to the exact command and input owners.
pub type Label<'a, C, I, E> = dyn FnMut(&C, &I) -> Result<Option<Vec<u16>>, E> + 'a;
/// Native resource bounds; they do not introduce a source fallback/normalizer.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Maximum default-title UTF-16 units read by lowercase.
    pub title_units: usize,
    /// Maximum complete Agent-label units, including prefix.
    pub output_units: usize,
}
/// Callback/getter faults retain their original value and identity.
#[derive(Debug, PartialEq, Eq)]
pub enum LabelError<E> {
    /// Exact custom-label or default-title provider exception.
    Callback(E),
    /// Bounded default lowercase failure.
    Lowercase(StringError),
    /// Complete label exceeds the host output limit.
    OutputLimit,
    /// Fallible complete-label reservation failed.
    Allocation,
}
/// Invoke the custom provider at most once. Some(empty) is a real label; only
/// absent/nullish uses the lazily read, fully lowercased title. Prefix and label
/// preserve all UTF-16 units. Fault normalization/history execution is the caller's.
pub fn history_label<C, I, E>(
    command: &C,
    input: &I,
    title: impl FnOnce(&C) -> Result<Vec<u16>, E>,
    label: Option<&mut Label<'_, C, I, E>>,
    limits: Limits,
) -> Result<Vec<u16>, LabelError<E>> {
    let supplied = match label {
        Some(label) => label(command, input).map_err(LabelError::Callback)?,
        None => None,
    };
    let value = match supplied {
        Some(value) => value,
        None => {
            let title = title(command).map_err(LabelError::Callback)?;
            js_string::lowercase(&title, limits.title_units, limits.output_units)
                .map_err(LabelError::Lowercase)?
        }
    };
    let size = value.len().checked_add(7).ok_or(LabelError::OutputLimit)?;
    if size > limits.output_units {
        return Err(LabelError::OutputLimit);
    }
    let mut out = Vec::new();
    out.try_reserve_exact(size)
        .map_err(|_| LabelError::Allocation)?;
    out.extend("Agent: ".encode_utf16());
    out.extend(value);
    Ok(out)
}
