// SPDX-License-Identifier: AGPL-3.0-only
//! `isStaleImport` over owned, string-message host inputs.
//! The original non-Unicode `/i` expression folds ASCII letters only when
//! matching its ASCII alternatives. Page state and error normalization are separate.

/// The relevant runtime categories, classified by the host without coercion.
#[derive(Clone, Copy, Debug)]
pub enum ImportFault<'a> {
    /// An actual CommandError; excluded before inspecting its message.
    CommandError,
    /// An actual Error with an owned string message.
    Error(&'a [u16]),
    /// A primitive string.
    Text(&'a [u16]),
    /// Anything else (including objects with a message property).
    Other,
}

/// Distinct host admission failure, not a command error or a negative match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextLimit;

/// Match the four complete original alternatives without changing page state.
/// `max_units` bounds borrowed text scanning; excluded categories never scan text.
pub fn is_stale_import(fault: ImportFault<'_>, max_units: usize) -> Result<bool, TextLimit> {
    let text = match fault {
        ImportFault::CommandError | ImportFault::Other => return Ok(false),
        ImportFault::Error(text) | ImportFault::Text(text) => text,
    };
    if text.len() > max_units {
        return Err(TextLimit);
    }
    const PATTERNS: [&[u8]; 4] = [
        b"failed to fetch dynamically imported module",
        b"error loading dynamically imported module",
        b"importing a module script failed",
        b"unable to preload css",
    ];
    Ok(PATTERNS.iter().any(|pattern| {
        text.windows(pattern.len()).any(|window| {
            window.iter().zip(*pattern).all(|(&unit, &expected)| {
                let folded = if (65..=90).contains(&unit) {
                    unit + 32
                } else {
                    unit
                };
                folded == u16::from(expected)
            })
        })
    }))
}
