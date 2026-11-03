// SPDX-License-Identifier: AGPL-3.0-only
//! Minimal shared requireSelection continuation at the constant-noun callers.
/// Original minimum marking size; duplicates still count as marked points.
pub const MIN_SELECTED: usize = 20;
/// Source refusal text for the valid Unicode nouns supplied by native callers.
/// No index/geometry validation occurs ahead of this source marking-size guard.
pub(crate) fn require_selection(len: usize, noun: &str) -> Result<(), String> {
    if len < MIN_SELECTED {
        Err(format!(
            "Paint more of the {noun} — a fit needs at least {MIN_SELECTED} marked points, and this selection has {len}."
        ))
    } else {
        Ok(())
    }
}
