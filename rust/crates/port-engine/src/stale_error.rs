// SPDX-License-Identifier: AGPL-3.0-only
//! Complete stale-page envelope construction over owned host message conversion.
use crate::{
    error::{CommandError, CommandErrorCode, EnvelopeError},
    page_stale::{MarkError, PageStale},
};

/// Original public stale-page explanation (lower-case first word).
pub const STALE_PAGE: &str = "this tab is older than the app it came from (rebuilt since) — save the project (project_save), then ask the person to reload; the work is lost on reload otherwise";

/// Source clock/coercion failures and distinct host storage failures.
#[derive(Debug, PartialEq, Eq)]
pub enum StaleError<E> {
    /// The page could not be marked; message conversion has not run.
    Mark(MarkError<E>),
    /// Message lookup or String conversion failed after marking the page.
    Message(E),
    /// Envelope construction failed after marking; it does not roll back state.
    Envelope(EnvelopeError),
}

/// Mark stale first, then read/convert the caught value's message and construct
/// the exact source error. Host callbacks are synchronous and non-reentrant.
/// `said` supplies Error.message or the result of String(value), preserving UTF-16.
pub fn stale_error<E>(
    page: &mut PageStale,
    timestamp_units: usize,
    message_units: usize,
    clock: impl FnOnce() -> Result<Vec<u16>, E>,
    said: impl FnOnce() -> Result<Vec<u16>, E>,
) -> Result<CommandError, StaleError<E>> {
    page.mark(timestamp_units, clock)
        .map_err(StaleError::Mark)?;
    let said = said().map_err(StaleError::Message)?;
    // The original uppercases exactly its first ASCII 't', retaining the tail.
    let prefix = STALE_PAGE.encode_utf16().count();
    let size = prefix
        .checked_add(4) // ". (" and ")"
        .and_then(|n| n.checked_add(said.len()))
        .ok_or(StaleError::Envelope(EnvelopeError::LengthOverflow))?;
    if size > message_units {
        return Err(StaleError::Envelope(EnvelopeError::MessageLimit));
    }
    let mut message = Vec::new();
    message
        .try_reserve_exact(size)
        .map_err(|_| StaleError::Envelope(EnvelopeError::Allocation))?;
    message.push(u16::from(b'T'));
    message.extend(STALE_PAGE[1..].encode_utf16());
    message.extend(". (".encode_utf16());
    message.extend(said);
    message.push(u16::from(b')'));
    CommandError::from_utf16(CommandErrorCode::StalePage, &message, message_units)
        .map_err(StaleError::Envelope)
}
