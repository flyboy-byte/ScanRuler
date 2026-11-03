// SPDX-License-Identifier: AGPL-3.0-only
//! `asCommandError` at owned message and fallible host String-conversion boundaries.
use crate::{
    error::{CommandError, CommandErrorCode, EnvelopeError},
    page_stale::{MarkError, PageStale},
    stale_error::{StaleError, stale_error},
    stale_import::{ImportFault, is_stale_import},
};
use std::sync::Arc;

/// Runtime category of the caught value, without arbitrary JS prototype access.
pub enum Caught<'a> {
    /// Actual typed error, returned with exactly the same allocation identity.
    Command(Arc<CommandError>),
    /// Actual Error with an owned string message.
    Error(&'a [u16]),
    /// Primitive string.
    Text(&'a [u16]),
    /// Other thrown value; the caller supplies its fallible String conversion.
    Other,
}
/// Distinct host admission/storage failure or original clock/message exception.
#[derive(Debug, PartialEq, Eq)]
pub enum NormalizeError<E> {
    /// Host text/allocation failure, not a source error-code substitution.
    Host(EnvelopeError),
    /// Failed platform clock sampling during stale marking.
    Clock(E),
    /// Failed String conversion of an ordinary thrown value.
    Message(E),
}
fn copy<E>(text: &[u16], max_units: usize) -> Result<Vec<u16>, NormalizeError<E>> {
    if text.len() > max_units {
        return Err(NormalizeError::Host(EnvelopeError::MessageLimit));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(text.len())
        .map_err(|_| NormalizeError::Host(EnvelopeError::Allocation))?;
    out.extend_from_slice(text);
    Ok(out)
}
/// Classify a caught fault exactly as the original registry does. Existing typed
/// errors bypass all limits and callbacks. Non-error/non-string values cannot
/// become stale through their String conversion. Host callbacks are synchronous.
pub fn normalize<E>(
    page: &mut PageStale,
    caught: Caught<'_>,
    timestamp_units: usize,
    message_units: usize,
    clock: impl FnOnce() -> Result<Vec<u16>, E>,
    stringify_other: impl FnOnce() -> Result<Vec<u16>, E>,
) -> Result<Arc<CommandError>, NormalizeError<E>> {
    let text = match caught {
        Caught::Command(error) => return Ok(error),
        Caught::Error(text) | Caught::Text(text) => text,
        Caught::Other => {
            let text = stringify_other().map_err(NormalizeError::Message)?;
            return CommandError::from_utf16(CommandErrorCode::Failed, &text, message_units)
                .map(Arc::new)
                .map_err(NormalizeError::Host);
        }
    };
    if is_stale_import(ImportFault::Text(text), message_units)
        .map_err(|_| NormalizeError::Host(EnvelopeError::MessageLimit))?
    {
        stale_error(
            page,
            timestamp_units,
            message_units,
            || clock().map_err(NormalizeError::Clock),
            || copy(text, message_units),
        )
        .map(Arc::new)
        .map_err(|error| match error {
            StaleError::Mark(MarkError::Clock(error)) | StaleError::Message(error) => error,
            StaleError::Mark(MarkError::TextLimit) => {
                NormalizeError::Host(EnvelopeError::MessageLimit)
            }
            StaleError::Envelope(error) => NormalizeError::Host(error),
        })
    } else {
        CommandError::from_utf16(CommandErrorCode::Failed, text, message_units)
            .map(Arc::new)
            .map_err(NormalizeError::Host)
    }
}
