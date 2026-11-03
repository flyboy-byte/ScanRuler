// SPDX-License-Identifier: AGPL-3.0-only
//! Local-editor branch of pinned historyStore.ts. The session supplies the full
//! historyBlocked result; global restoration is an explicit continuation.
/// Requested history direction.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Undo in the active editor.
    Undo,
    /// Redo in the active editor.
    Redo,
}
/// Owned editor callbacks. UI capability hints are deliberately not consulted by
/// source dispatch. Callbacks are synchronous; arbitrary executable JS and ignored
/// callback promises are outside this typed boundary.
pub trait LocalHistory<E> {
    /// Test activity; called in registration order, stopping at the first true.
    fn active(&mut self) -> Result<bool, E>;
    /// Workspace belonging to this editor, read after activity succeeds.
    fn workspace(&self) -> &[u16];
    /// Perform the requested editor action, retaining opaque caller errors.
    fn step(&mut self, direction: Direction) -> Result<(), E>;
}
/// Explicit host bound on registered local editor metadata.
pub const MAX_LOCALS: usize = 4096;
/// Local callback and host admission failures remain distinct.
#[derive(Debug, PartialEq, Eq)]
pub enum LocalError<E> {
    /// Participant count or workspace text exceeds the host budget.
    Limit,
    /// Original active/workspace/action callback error, unmodified.
    Callback(E),
}
/// Result tells the owner whether global history travel still needs to run.
#[derive(Debug, PartialEq, Eq)]
pub enum Dispatch {
    /// Full session guard refused travel; no local callbacks were consulted.
    Blocked,
    /// No active local editor. Caller must continue through global travel.
    Global,
    /// This registered editor handled the request.
    Local(usize),
}
/// Original activeLocalHistory selection, independent of historyBlocked.
pub fn active<E>(locals: &mut [&mut dyn LocalHistory<E>]) -> Result<Option<usize>, LocalError<E>> {
    if locals.len() > MAX_LOCALS {
        return Err(LocalError::Limit);
    }
    for (i, local) in locals.iter_mut().enumerate() {
        if local.active().map_err(LocalError::Callback)? {
            return Ok(Some(i));
        }
    }
    Ok(None)
}
/// Execute only the complete local branch of travel. The workspace mutation
/// happens before the action, including when that action fails. This function
/// never reports a missing local editor as a completed global restore.
pub fn dispatch<E>(
    blocked: bool,
    locals: &mut [&mut dyn LocalHistory<E>],
    direction: Direction,
    mut set_workspace: impl FnMut(&[u16]) -> Result<(), E>,
) -> Result<Dispatch, LocalError<E>> {
    if blocked {
        return Ok(Dispatch::Blocked);
    }
    let Some(index) = active(locals)? else {
        return Ok(Dispatch::Global);
    };
    let workspace = locals[index].workspace();
    if workspace.len() > 4096 {
        return Err(LocalError::Limit);
    }
    set_workspace(workspace).map_err(LocalError::Callback)?;
    locals[index]
        .step(direction)
        .map_err(LocalError::Callback)?;
    Ok(Dispatch::Local(index))
}
