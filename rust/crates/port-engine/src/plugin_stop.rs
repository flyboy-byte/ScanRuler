// SPDX-License-Identifier: AGPL-3.0-only
//! Complete stopPluginWork at the typed check/stop/promise boundary.
use crate::{
    plugin_busy::Plugins,
    settle::{SettleRun, Work},
};
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
/// Shared stop-function identity; every call receives the identical done capability.
pub type Stop<'a, D, E> = dyn Fn(Rc<D>) -> Result<Work<'a, E>, E> + 'a;
/// Host refusals are separate from source callback/rejection failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopError {
    /// Nested starts exceed the registry's bound before work starts.
    Depth,
    /// Wait-list admission fails before work starts.
    Allocation,
    /// Live iteration ran out of visits; all earlier waits are still drained.
    Visits,
}
/// Own every started wait, including when a later host limit stops iteration.
/// The executor must retain/drain it; dropping a Rust future would cancel work
/// that an original already-started promise continues to run.
#[must_use = "retain and drain all started plugin stops"]
pub struct StopRun<'a, E> {
    waits: Option<SettleRun<'a, E>>,
    error: Option<StopError>,
}
impl<E> Future for StopRun<'_, E> {
    type Output = Result<(), StopError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(waits) = &mut this.waits
            && Pin::new(waits).poll(cx).is_pending()
        {
            return Poll::Pending;
        }
        this.waits = None;
        Poll::Ready(this.error.map_or(Ok(()), Err))
    }
}
struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl<'a, S: ?Sized, D: ?Sized + 'a, E: 'a> Plugins<'a, S, Stop<'a, D, E>, E> {
    /// Start all eligible stops before waiting. Check/stop synchronous errors and
    /// rejected waits are suppressed. Registry mutation stays live; the stop
    /// selected before its check is called even if that check replaces it.
    pub fn stop_work(&self, done: Rc<D>) -> Result<StopRun<'a, E>, StopError> {
        if self.inner.depth.get() >= self.inner.limits.depth {
            return Err(StopError::Depth);
        }
        let mut waits = Vec::new();
        waits
            .try_reserve_exact(self.inner.limits.slots)
            .map_err(|_| StopError::Allocation)?;
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        let mut index = 0;
        let mut visits = 0;
        let mut error = None;
        loop {
            let entry = {
                let entries = self.inner.entries.borrow();
                if index == entries.len() {
                    break;
                }
                entries[index]
                    .as_ref()
                    .filter(|e| e.stop.is_some())
                    .cloned()
            };
            index += 1;
            let Some(entry) = entry else { continue };
            if visits >= self.inner.limits.visits {
                error = Some(StopError::Visits);
                break;
            }
            visits += 1;
            if let Ok(Some(busy)) = (entry.check)()
                && !busy.is_empty()
                && let Some(stop) = entry.stop
                && let Ok(wait) = stop(done.clone())
            {
                waits.push(Some(wait))
            }
        }
        Ok(StopRun {
            waits: SettleRun::from_started(waits),
            error,
        })
    }
}
