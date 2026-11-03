// SPDX-License-Identifier: AGPL-3.0-only
//! First stale-page timestamp and reset from `stalePage.ts`.
//! UTC timestamp creation is supplied by a synchronous, non-reentrant host clock.

/// Clock or host admission failure; neither marks the page stale.
#[derive(Debug, PartialEq, Eq)]
pub enum MarkError<E> {
    /// The host clock failed before yielding its timestamp.
    Clock(E),
    /// The supplied owned timestamp exceeds the explicit host text budget.
    TextLimit,
}

/// Per-session stale-page state. A new instance represents a freshly loaded page.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PageStale {
    since: Option<Vec<u16>>,
}
impl PageStale {
    /// The first observed timestamp, or none while this page is not marked.
    pub fn since(&self) -> Option<&[u16]> {
        self.since.as_deref()
    }
    /// Sample the platform clock only when not already marked.
    /// The clock returns owned ISO UTC text; actual time conversion is its boundary.
    /// Clock errors and host limits leave the previous state untouched.
    pub fn mark<E>(
        &mut self,
        max_units: usize,
        clock: impl FnOnce() -> Result<Vec<u16>, E>,
    ) -> Result<(), MarkError<E>> {
        if self.since.is_none() {
            let timestamp = clock().map_err(MarkError::Clock)?;
            if timestamp.len() > max_units {
                return Err(MarkError::TextLimit);
            }
            self.since = Some(timestamp);
        }
        Ok(())
    }
    /// Return to the state of a freshly loaded page; the next mark samples again.
    pub fn reset(&mut self) {
        self.since = None;
    }
}

impl PageStale {
    /// Native Date-millisecond adapter. Conversion is lazy: an already marked
    /// page neither samples nor validates the new time, even when it is invalid.
    /// Reading the actual OS current time remains the runtime's responsibility.
    pub fn mark_millis(
        &mut self,
        milliseconds: f64,
        max_units: usize,
    ) -> Result<(), MarkError<port_core::js_date::DateError>> {
        self.mark(max_units, || {
            port_core::js_date::utc_iso_millis(milliseconds)
        })
    }
}
