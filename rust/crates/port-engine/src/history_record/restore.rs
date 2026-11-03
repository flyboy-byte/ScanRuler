// SPDX-License-Identifier: AGPL-3.0-only
//! Global travel orchestration over a stable typed capture/apply host. Call only
//! after complete guards and local dispatch. Concrete family restore is separate.
use super::{Entry, HistoryError, Patch, Recorder, SAFE, Snapshot, State, snapshot_check};
use crate::schema_value::Text;
use std::{future::Future, pin::Pin};
/// Owned single-use restore job, polled by the session's queue/runtime.
pub type RestoreFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
/// Error-like callback failures expose a lossless source-normalized message.
/// Primitive JavaScript throw normalization belongs to the host boundary.
pub trait RestoreFault {
    /// UTF16 message, without human Display conversion.
    fn message(&self) -> &[u16];
}
impl RestoreFault for Text {
    fn message(&self) -> &[u16] {
        self
    }
}
/// Read-only bookkeeping at callback entry, with no second mutable state store.
#[derive(Debug, Clone, Copy)]
pub struct View {
    /// Restoring flag.
    pub restoring: bool,
    /// Undo stack size.
    pub past: usize,
    /// Redo stack size.
    pub future: usize,
    /// Source revision.
    pub revision: u64,
    /// Original importing flag.
    pub importing: bool,
}
impl From<&State> for View {
    fn from(s: &State) -> Self {
        Self {
            restoring: s.restoring,
            past: s.past.len(),
            future: s.future.len(),
            revision: s.revision,
            importing: s.importing,
        }
    }
}
/// Ordered restore callback stage.
#[derive(Debug, Clone, Copy)]
pub enum Hook {
    /// App beforeRestore.
    BeforeCore,
    /// Participant beforeRestore, registration index.
    BeforeParticipant(usize),
    /// App afterRestore.
    AfterCore,
    /// Participant afterRestore, registration index.
    AfterParticipant(usize),
}
/// The session owns actual document state, capture/apply and asynchronous work.
/// Participant registration stays stable during a restore. A run_restore job
/// may be skipped or invoked once and its error may be caught by that host.
/// Arbitrary concurrent/repeated JS job invocation is outside this typed host.
pub trait RestoreHost {
    /// Callback error, retained intact until source restoration error handling.
    type Error: RestoreFault;
    /// Number of stable registered participants, including no-op hook slots.
    fn participants(&self) -> usize;
    /// Capture the actual current owned document sections.
    fn capture(&mut self, view: View) -> Result<Snapshot, Self::Error>;
    /// Run within the import/restore scope; default is the original direct job.
    fn run_restore<'a>(
        &'a mut self,
        _view: View,
        job: impl FnOnce(&'a mut Self) -> RestoreFuture<'a, Result<(), Self::Error>> + 'a,
    ) -> RestoreFuture<'a, Result<(), Self::Error>>
    where
        Self: Sized,
    {
        job(self)
    }
    /// Await one app/participant hook; absent optional hooks are host no-ops.
    fn hook<'a>(
        &'a mut self,
        hook: Hook,
        patch: &'a Patch,
        previous: &'a Snapshot,
        view: View,
    ) -> RestoreFuture<'a, Result<(), Self::Error>>;
    /// Apply the actual patch synchronously, preserving partial effects on error.
    fn apply(&mut self, patch: &Patch, view: View) -> Result<(), Self::Error>;
    /// Switch workspace before moving history entries.
    fn workspace(&mut self, workspace: &[u16]) -> Result<(), Self::Error>;
    /// Set original success status (already formatted lossless UTF16).
    fn status(&mut self, status: Text) -> Result<(), Self::Error>;
    /// Source catch clears worker busy before setting the restore error.
    fn clear_busy(&mut self);
    /// Set source error text; owned state mutation cannot throw.
    fn error(&mut self, error: Text);
}
/// Global operation outcome. A caught restore error is stored by the host, as
/// in the original; it is not misreported as a propagated command exception.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// No entry: no capture, flag, epoch or callbacks changed.
    Empty,
    /// Restore scope resolved (possibly without invoking the job).
    Completed,
    /// Restore scope rejected; source error handler ran without rollback.
    Failed,
}
struct Restoring<'a>(&'a mut Recorder);
impl Drop for Restoring<'_> {
    fn drop(&mut self) {
        self.0.state.restoring = false;
    }
}
/// Run the global branch only. Guards/local dispatch remain caller prerequisites.
/// Host counters/metadata are bounded before mutation. Dropping the owned future
/// clears restoring without rolling back applied data; session cancellation and
/// busy cleanup still need an explicit host termination policy.
pub async fn travel<H: RestoreHost>(
    recorder: &mut Recorder,
    host: &mut H,
    redo: bool,
) -> Result<Outcome, HistoryError<H::Error>> {
    let entry = (if redo {
        &recorder.state.future
    } else {
        &recorder.state.past
    })
    .last();
    let Some(entry) = entry else {
        return Ok(Outcome::Empty);
    };
    if host.participants() > 4096
        || recorder.epoch >= SAFE
        || recorder.state.revision >= SAFE
        || recorder
            .state
            .past
            .len()
            .saturating_add(recorder.state.future.len())
            > super::HISTORY_LIMIT
        || entry.label.len() > recorder.limits.text
        || entry.workspace.len() > recorder.limits.text
    {
        return Err(HistoryError::Limit);
    }
    let entry: Entry = entry.clone();
    // Reserve bookkeeping before host work; payload references remain shared.
    recorder
        .state
        .past
        .try_reserve(1)
        .map_err(|_| HistoryError::Allocation)?;
    recorder
        .state
        .future
        .try_reserve(1)
        .map_err(|_| HistoryError::Allocation)?;
    let previous = host
        .capture(View::from(&recorder.state))
        .map_err(HistoryError::Capture)?;
    snapshot_check(&previous, recorder.limits)?;
    recorder.state.restoring = true;
    recorder.epoch += 1;
    let restoring = Restoring(recorder);
    let start = View::from(&restoring.0.state);
    let participants = host.participants();
    let result = host
        .run_restore(start, |host| {
            Box::pin(async {
                let patch = if redo { &entry.after } else { &entry.before };
                host.hook(
                    Hook::BeforeCore,
                    patch,
                    &previous,
                    View::from(&restoring.0.state),
                )
                .await?;
                for i in 0..participants {
                    host.hook(
                        Hook::BeforeParticipant(i),
                        patch,
                        &previous,
                        View::from(&restoring.0.state),
                    )
                    .await?;
                }
                host.apply(patch, View::from(&restoring.0.state))?;
                host.workspace(&entry.workspace)?;
                if redo {
                    restoring.0.state.future.pop();
                    restoring.0.state.past.push(entry.clone());
                } else {
                    restoring.0.state.past.pop();
                    restoring.0.state.future.push(entry.clone());
                }
                restoring.0.state.revision += 1;
                host.hook(
                    Hook::AfterCore,
                    patch,
                    &previous,
                    View::from(&restoring.0.state),
                )
                .await?;
                for i in 0..participants {
                    host.hook(
                        Hook::AfterParticipant(i),
                        patch,
                        &previous,
                        View::from(&restoring.0.state),
                    )
                    .await?;
                }
                let mut status: Text = (if redo { "Redone: " } else { "Undone: " })
                    .encode_utf16()
                    .collect();
                status.extend_from_slice(&entry.label);
                status.extend(".".encode_utf16());
                host.status(status)?;
                Ok(())
            })
        })
        .await;
    let outcome = match result {
        Ok(()) => Outcome::Completed,
        Err(error) => {
            host.clear_busy();
            let mut text: Text = "History could not be restored — ".encode_utf16().collect();
            text.extend_from_slice(error.message());
            host.error(text);
            Outcome::Failed
        }
    };
    drop(restoring);
    Ok(outcome)
}
