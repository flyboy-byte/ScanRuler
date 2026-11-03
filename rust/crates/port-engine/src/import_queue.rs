// SPDX-License-Identifier: AGPL-3.0-only
//! ImportQueue from pinned src/app/importQueue.ts, over standard owned futures.
//! The runtime polls every returned Run. The queue retains metadata/signals only,
//! never a second business-state store or a foreign runtime.
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};
/// Shared function identity, as in the source Set of listener functions.
pub type Listener<E> = Rc<dyn Fn() -> Result<(), E>>;
/// Host bounds; exceeding one is separate from a source callback exception.
#[derive(Debug, Clone, Copy)]
pub struct QueueLimits {
    /// Outstanding jobs, including a source pending increment left by a listener error.
    pub pending: usize,
    /// Total listener registrations, including removed iteration slots.
    pub registrations: usize,
    /// Listener slots visited during a single notification.
    pub visits: usize,
    /// Reentrant notification depth.
    pub depth: usize,
}
impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            pending: 1024,
            registrations: 4096,
            visits: 4096,
            depth: 32,
        }
    }
}
/// Source errors retain their value; host failures are never counted as parity.
#[derive(Debug, PartialEq, Eq)]
pub enum QueueError<E> {
    /// Exact job or listener exception, without cloning or Display conversion.
    Callback(E),
    /// Host queue/listener metadata or notification-work limit.
    Limit,
    /// Fallible listener metadata reservation failed.
    Allocation,
    /// An unfinished owned future was dropped; this queue is quarantined.
    Abandoned,
    /// Caller polled a completed single-use Rust future again.
    Completed,
    /// Host bookkeeping was invalid.
    InvalidState,
}
struct Signal {
    ready: Cell<bool>,
    waker: RefCell<Option<Waker>>,
}
impl Signal {
    fn new(ready: bool) -> Self {
        Self {
            ready: Cell::new(ready),
            waker: RefCell::new(None),
        }
    }
    fn release(&self) {
        self.ready.set(true);
        let wake = self.waker.borrow_mut().take();
        if let Some(w) = wake {
            w.wake()
        }
    }
}
struct Registration<E> {
    callback: Listener<E>,
    active: Cell<bool>,
}
struct Inner<E> {
    pending: Cell<usize>,
    listeners: RefCell<Vec<Rc<Registration<E>>>>,
    tail: RefCell<Rc<Signal>>,
    abandoned: Cell<bool>,
    depth: Cell<usize>,
    limits: QueueLimits,
}
struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl<E> Inner<E> {
    fn changed(&self) -> Result<(), QueueError<E>> {
        if self.depth.get() >= self.limits.depth {
            return Err(QueueError::Limit);
        }
        self.depth.set(self.depth.get() + 1);
        let _depth = Depth(&self.depth);
        let mut index = 0;
        loop {
            // Release the list borrow before invoking a callback. Removal skips
            // unvisited slots; additions are visited in this same notification.
            let slot = self.listeners.borrow().get(index).cloned();
            let Some(slot) = slot else { return Ok(()) };
            if index >= self.limits.visits {
                return Err(QueueError::Limit);
            }
            index += 1;
            if slot.active.get() {
                (slot.callback)().map_err(QueueError::Callback)?;
            }
        }
    }
}
/// Shared local-thread queue metadata. Numerical/worker jobs may own external
/// resources; their executor/termination policy is the session's responsibility.
pub struct ImportQueue<E> {
    inner: Rc<Inner<E>>,
}
impl<E> Clone for ImportQueue<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<E> Default for ImportQueue<E> {
    fn default() -> Self {
        Self::new(QueueLimits::default())
    }
}
/// Weak handle for reentrant callbacks, avoiding a strong callback/queue cycle.
pub struct WeakQueue<E> {
    inner: Weak<Inner<E>>,
}
impl<E> Clone for WeakQueue<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<E> WeakQueue<E> {
    /// Recover a live queue handle without extending its lifetime beforehand.
    pub fn upgrade(&self) -> Option<ImportQueue<E>> {
        self.inner.upgrade().map(|inner| ImportQueue { inner })
    }
}
/// Explicit unsubscribe function capability. Dropping it does not unsubscribe,
/// matching an unused returned JS function. Old capabilities remove a later
/// registration of the same function, too; repeated unsubscribe is harmless.
pub struct Subscription<E> {
    inner: Rc<Inner<E>>,
    callback: Listener<E>,
}
impl<E> Clone for Subscription<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            callback: self.callback.clone(),
        }
    }
}
impl<E> Subscription<E> {
    /// Remove this function identity from the live set.
    pub fn unsubscribe(&self) {
        for slot in self.inner.listeners.borrow().iter() {
            if Rc::ptr_eq(&slot.callback, &self.callback) {
                slot.active.set(false)
            }
        }
    }
}
impl<E> ImportQueue<E> {
    /// Empty queue under explicit host bounds.
    pub fn new(limits: QueueLimits) -> Self {
        Self {
            inner: Rc::new(Inner {
                pending: Cell::new(0),
                listeners: RefCell::new(Vec::new()),
                tail: RefCell::new(Rc::new(Signal::new(true))),
                abandoned: Cell::new(false),
                depth: Cell::new(0),
                limits,
            }),
        }
    }
    /// Source busy getter: pending is greater than zero.
    pub fn busy(&self) -> bool {
        self.inner.pending.get() > 0
    }
    /// Outstanding count, also useful to the runtime's resource owner.
    pub fn pending(&self) -> usize {
        self.inner.pending.get()
    }
    /// Host abandonment is separate from source callback failure.
    pub fn abandoned(&self) -> bool {
        self.inner.abandoned.get()
    }
    /// Weak handle appropriate for callbacks that need this queue.
    pub fn downgrade(&self) -> WeakQueue<E> {
        WeakQueue {
            inner: Rc::downgrade(&self.inner),
        }
    }
    /// Subscribe by function identity. Adding an existing function does not
    /// duplicate or reorder it. Notifications do not run at subscribe time.
    pub fn subscribe(&self, callback: Listener<E>) -> Result<Subscription<E>, QueueError<E>> {
        let mut listeners = self.inner.listeners.borrow_mut();
        if !listeners
            .iter()
            .any(|slot| slot.active.get() && Rc::ptr_eq(&slot.callback, &callback))
        {
            if listeners.len() >= self.inner.limits.registrations {
                return Err(QueueError::Limit);
            }
            listeners
                .try_reserve(1)
                .map_err(|_| QueueError::Allocation)?;
            listeners.push(Rc::new(Registration {
                callback: callback.clone(),
                active: Cell::new(true),
            }));
        }
        Ok(Subscription {
            inner: self.inner.clone(),
            callback,
        })
    }
    /// Synchronously reserve a pending job and notify, then append its tail and
    /// return the owned future. A notifying listener exception occurs BEFORE
    /// enqueue and leaves pending incremented, exactly as in the source.
    #[must_use = "the runtime must retain and poll the returned job to completion"]
    pub fn run<J, F, T>(&self, job: J) -> Result<Run<J, F, E>, QueueError<E>>
    where
        J: FnOnce() -> F,
        F: Future<Output = Result<T, E>>,
    {
        if self.inner.abandoned.get() {
            return Err(QueueError::Abandoned);
        }
        let pending = self
            .inner
            .pending
            .get()
            .checked_add(1)
            .ok_or(QueueError::Limit)?;
        if pending > self.inner.limits.pending {
            return Err(QueueError::Limit);
        }
        self.inner.pending.set(pending);
        self.inner.changed()?;
        // Reentrant listener jobs have appended their tails already.
        let prior = self.inner.tail.borrow().clone();
        let own = Rc::new(Signal::new(false));
        *self.inner.tail.borrow_mut() = own.clone();
        Ok(Run {
            queue: self.inner.clone(),
            prior,
            own,
            job: Some(Box::new(job)),
            future: None,
            finished: false,
        })
    }
}
/// Scoped single-use job future. The factory may borrow the owning restore host;
/// the queue holds only completion signals, so it adds no artificial static bound.
#[must_use = "the runtime must retain and poll queue jobs to completion"]
pub struct Run<J, F, E> {
    queue: Rc<Inner<E>>,
    prior: Rc<Signal>,
    own: Rc<Signal>,
    job: Option<Box<J>>,
    future: Option<Pin<Box<F>>>,
    finished: bool,
}
impl<J, F, E> Drop for Run<J, F, E> {
    fn drop(&mut self) {
        if !self.finished {
            self.queue.abandoned.set(true);
            self.own.release();
        }
    }
}
impl<J, F, T, E> Future for Run<J, F, E>
where
    J: FnOnce() -> F,
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, QueueError<E>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Ready(Err(QueueError::Completed));
        }
        if this.queue.abandoned.get() {
            this.finished = true;
            this.own.release();
            return Poll::Ready(Err(QueueError::Abandoned));
        }
        if !this.prior.ready.get() {
            *this.prior.waker.borrow_mut() = Some(cx.waker().clone());
            return Poll::Pending;
        }
        if let Some(job) = this.job.take() {
            this.future = Some(Box::pin(job()));
        }
        let Some(future) = this.future.as_mut() else {
            this.queue.abandoned.set(true);
            this.finished = true;
            this.own.release();
            return Poll::Ready(Err(QueueError::InvalidState));
        };
        let result = match future.as_mut().poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(result) => result,
        };
        let Some(pending) = this.queue.pending.get().checked_sub(1) else {
            this.queue.abandoned.set(true);
            this.finished = true;
            this.own.release();
            return Poll::Ready(Err(QueueError::InvalidState));
        };
        this.queue.pending.set(pending);
        let notification = this.queue.changed();
        this.finished = true;
        this.own.release();
        // finally listener failure overrides job success OR its original error.
        Poll::Ready(match notification {
            Ok(()) => result.map_err(QueueError::Callback),
            Err(error) => Err(error),
        })
    }
}
