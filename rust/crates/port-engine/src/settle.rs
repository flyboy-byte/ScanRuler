// SPDX-License-Identifier: AGPL-3.0-only
//! Deferred-work settling over stable ordered callbacks from registry.ts.
//! Registration/live Set mutation and actual executor ownership are separate.
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Fresh per-callback next-command metadata; no mutable command object is shared.
#[derive(Clone, Copy)]
pub struct Next<'a> {
    /// Complete command name, preserving UTF-16.
    pub name: &'a [u16],
    /// The original Boolean(command.readOnly), already classified by the caller.
    pub read_only: bool,
}
/// Started asynchronous work. The runtime must keep and poll every returned run.
pub type Work<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + 'a>>;
/// A callback starts its work synchronously, then returns void or its completion.
pub type Settler<'a, E> = dyn FnMut(Next<'a>) -> Result<Option<Work<'a, E>>, E> + 'a;
/// Separate bounded-host failure before any callback starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettleError {
    /// Too many callbacks or too much command-name text.
    Limit,
    /// Fallible wait-list metadata allocation failed.
    Allocation,
}
/// Caller ownership of every started wait. Polls all pending work, suppressing
/// each failure and finishing only after all waits complete. Do not drop this
/// run to cancel waiting: the source's already-started promises keep running.
#[must_use = "the executor must retain and drain every started settle run"]
pub struct SettleRun<'a, E> {
    waits: Vec<Option<Work<'a, E>>>,
}
impl<'a, E> SettleRun<'a, E> {
    pub(crate) fn from_started(waits: Vec<Option<Work<'a, E>>>) -> Option<Self> {
        if waits.is_empty() {
            None
        } else {
            Some(Self { waits })
        }
    }
}
impl<E> Future for SettleRun<'_, E> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        let mut pending = false;
        for slot in &mut this.waits {
            if let Some(wait) = slot {
                match wait.as_mut().poll(cx) {
                    Poll::Ready(_) => *slot = None,
                    Poll::Pending => pending = true,
                }
            }
        }
        if pending {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
/// Invoke every stable callback in order before waiting. Synchronous exceptions
/// are ignored; no returned waits means None, matching the original null branch.
/// Admission is checked before callbacks, so a host refusal starts no new work.
pub fn settle<'a, E>(
    next: Next<'a>,
    callbacks: &mut [&mut Settler<'a, E>],
    max_callbacks: usize,
    max_name_units: usize,
) -> Result<Option<SettleRun<'a, E>>, SettleError> {
    if callbacks.len() > max_callbacks || next.name.len() > max_name_units {
        return Err(SettleError::Limit);
    }
    let mut waits = Vec::new();
    waits
        .try_reserve_exact(callbacks.len())
        .map_err(|_| SettleError::Allocation)?;
    for callback in callbacks {
        if let Ok(Some(wait)) = callback(next) {
            waits.push(Some(wait));
        }
    }
    Ok(SettleRun::from_started(waits))
}
