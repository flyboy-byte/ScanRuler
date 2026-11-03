// SPDX-License-Identifier: AGPL-3.0-only
//! Complete historyBlocked and draft/dimension hold predicates from the pinned
//! historyStore, elementModel and flatModel. Borrowed work projections are read
//! from the owner's actual state, without copying geometry or editor data.
/// Only the draft fields these predicates observe. Presence means not undefined
/// for editId and not null for a reference; numeric values are irrelevant here.
#[derive(Debug, Clone, Copy, Default)]
pub struct DraftWork<'a> {
    /// editId is present, including zero, null or NaN in source data.
    pub edit_present: bool,
    /// Number of picked points.
    pub picks: usize,
    /// Number of marked vertices; absent selection counts as zero.
    pub selection: usize,
    /// Filled reference slots, in original order.
    pub references: &'a [bool],
}
/// Original draftHolds for a 3D editor; an empty kind in hand holds nothing.
pub fn draft_holds(draft: Option<DraftWork<'_>>) -> bool {
    draft.is_some_and(|d| {
        d.edit_present || d.picks > 0 || d.selection > 0 || d.references.iter().any(|&r| r)
    })
}
/// Original flatDraftHolds; marked selection is not a flat-draft hold field.
pub fn flat_draft_holds(draft: Option<DraftWork<'_>>) -> bool {
    draft.is_some_and(|d| d.edit_present || d.picks > 0 || d.references.iter().any(|&r| r))
}
/// Original dimHolds; only an edit or a filled reference holds a dimension box.
pub fn dim_holds(draft: Option<DraftWork<'_>>) -> bool {
    draft.is_some_and(|d| d.edit_present || d.references.iter().any(|&r| r))
}
/// Immutable snapshot of the fields read at historyBlocked entry. The source
/// reads store snapshots before invoking hooks; callback replacement of a store
/// must not silently replace this already captured view.
#[derive(Debug, Default)]
pub struct Guards<'a> {
    /// History restore in progress.
    pub restoring: bool,
    /// Number of open asynchronous history wrappers.
    pub paused: usize,
    /// Measure worker busy.
    pub busy: bool,
    /// Flat image decode busy.
    pub image_busy: bool,
    /// Nominal read busy.
    pub nominal_busy: bool,
    /// Alignment status is exactly running.
    pub align_running: bool,
    /// Deviation map status is exactly running.
    pub map_running: bool,
    /// Thickness status is exactly running.
    pub thickness_running: bool,
    /// Measure draft.
    pub draft: Option<DraftWork<'a>>,
    /// Measure dimension draft.
    pub dimension: Option<DraftWork<'a>>,
    /// Alignment draft is truthy.
    pub align_draft: bool,
    /// Section draft is truthy.
    pub section_draft: bool,
    /// Flat draft.
    pub flat_draft: Option<DraftWork<'a>>,
    /// Flat dimension draft.
    pub flat_dimension: Option<DraftWork<'a>>,
    /// Flat tool kind differs from none, including datum and calibration tools.
    pub flat_tool: bool,
}
/// Guard evaluation faults, retaining callback-owned errors losslessly.
#[derive(Debug, PartialEq, Eq)]
pub enum GuardError<E> {
    /// Host bound on registered blocked callbacks was exceeded.
    Limit,
    /// Exact caller-owned error from the configured or participant callback.
    Callback(E),
}
/// Original short-circuit evaluation over stable typed snapshots/participants.
/// Missing optional participant hooks are omitted by the owner. The 4096-hook
/// host bound is checked only if this stage is reached; early guards stay inert.
pub fn blocked<E>(
    g: &Guards<'_>,
    hook: impl FnOnce() -> Result<bool, E>,
    participants: &mut [&mut dyn FnMut() -> Result<bool, E>],
) -> Result<bool, GuardError<E>> {
    if g.restoring
        || g.paused > 0
        || hook().map_err(GuardError::Callback)?
        || g.busy
        || g.image_busy
        || g.nominal_busy
        || g.align_running
        || g.map_running
        || g.thickness_running
        || draft_holds(g.draft)
        || dim_holds(g.dimension)
        || g.align_draft
        || g.section_draft
        || flat_draft_holds(g.flat_draft)
        || dim_holds(g.flat_dimension)
        || g.flat_tool
    {
        return Ok(true);
    }
    if participants.len() > 4096 {
        return Err(GuardError::Limit);
    }
    for participant in participants {
        if participant().map_err(GuardError::Callback)? {
            return Ok(true);
        }
    }
    Ok(false)
}
