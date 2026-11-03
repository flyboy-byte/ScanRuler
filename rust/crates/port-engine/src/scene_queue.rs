// SPDX-License-Identifier: AGPL-3.0-only
//! Complete inScene helper, reusing a private listener-free owned queue.
use crate::import_queue::{ImportQueue, QueueError, QueueLimits};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
/// Route preserves the exact direct future value; queued routes own their turn.
/// The executor must retain/drain either route, including timed-out body work
/// owned outside this helper. Dropping a queued route quarantines the queue.
#[must_use = "retain and drain every command route"]
pub enum Route<'a, F, T, E> {
    /// Original future value returned immediately by a non-scene factory.
    Direct(F),
    /// Scene work starts after its previous scene turn completes or fails.
    Queued(Pin<Box<dyn Future<Output = Result<T, QueueError<E>>> + 'a>>),
}
impl<F, T, E> Future for Route<'_, F, T, E>
where
    F: Future<Output = Result<T, E>> + Unpin,
{
    type Output = Result<T, QueueError<E>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            Self::Direct(future) => Pin::new(future)
                .poll(cx)
                .map(|r| r.map_err(QueueError::Callback)),
            Self::Queued(future) => future.as_mut().poll(cx),
        }
    }
}
/// Only scene jobs enter this queue; non-scene calls do not inspect or mutate it.
/// No listeners can be installed, so ImportQueue's notification bodies have no
/// effects. All source outcomes and factory errors retain their opaque values.
pub struct SceneQueue<E> {
    queue: ImportQueue<E>,
}
impl<E> Clone for SceneQueue<E> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}
impl<E> SceneQueue<E> {
    /// Fresh viewport queue with a finite outstanding-turn bound.
    pub fn new(max_pending: usize) -> Self {
        Self {
            queue: ImportQueue::new(QueueLimits {
                pending: max_pending,
                registrations: 0,
                visits: 0,
                depth: 1,
            }),
        }
    }
    /// Invoke non-scene factories immediately, returning that exact future or
    /// synchronous error. Scene factory errors become asynchronous turn errors;
    /// either success or failure releases the next scene turn. There is no new
    /// timer, settle or history behavior in this helper.
    pub fn run<'a, J, F, T>(&self, scene: bool, run: J) -> Result<Route<'a, F, T, E>, QueueError<E>>
    where
        J: FnOnce() -> Result<F, E> + 'a,
        F: Future<Output = Result<T, E>> + 'a,
        T: 'a,
        E: 'a,
    {
        if !scene {
            return run().map(Route::Direct).map_err(QueueError::Callback);
        }
        let turn = self.queue.run(move || async move { run()?.await })?;
        Ok(Route::Queued(Box::pin(turn)))
    }
}
