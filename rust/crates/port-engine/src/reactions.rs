// SPDX-License-Identifier: AGPL-3.0-only
//! Retained native reactions at the reviewed source promise boundaries.
//!
//! Report body settlement when its promise settles, not by polling after timer rejection.
//! An abort listener may resolve it synchronously; the factory-return boundary preserves
//! `[ran, late]` ties. Only the task owner runs finally after synchronous delivery returns.
//! Its task lives until caller cleanup AND body settlement, independent of observation.
//!
//! This is not a general JS Promise interpreter. Providers must report actual completion
//! and wake pending futures. `retain` starts factories eagerly and resumes returned work
//! asynchronously; nested source awaits belong to provider adapters. Platform timers remain
//! trusted host services. Full guarded/session/CLI integration is a separate continuation.
use crate::{
    command_deadline::{AbortEvent, Completion, Deadline, DeadlineError},
    error::CommandError,
    tasks::{self, Owner, Ticket},
};
use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};
/// Host admission failures, distinct from application faults/timeouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    /// Deadline metadata admission failed.
    Deadline(DeadlineError),
    /// Owner refused admission; no timer/factory was started.
    Owner(tasks::Error),
}
struct Shared<T, E> {
    deadline: RefCell<Deadline<T, E>>,
    body_done: Cell<bool>,
    cleared: Cell<bool>,
    wake: RefCell<Option<Waker>>,
    failure: Cell<Option<DeadlineError>>,
    caller: RefCell<Option<Waker>>,
    claimed: Cell<bool>,
    consumed: Cell<bool>,
}
impl<T, E> Shared<T, E> {
    fn wake(&self) {
        let wake = self.wake.borrow().clone();
        if let Some(wake) = wake {
            wake.wake()
        }
    }
}
/// Timer/body capability and single-use caller readout. Clones share identity;
/// none own cancellation. Retain and drain the session's Owner at shutdown.
pub struct Limited<T, E> {
    shared: Rc<Shared<T, E>>,
}
impl<T, E> Clone for Limited<T, E> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}
impl<T, E> Limited<T, E> {
    /// Admit the retained reaction BEFORE acquiring a timer or invoking the factory.
    /// `clear` is the trusted synchronous clearTimeout equivalent, called exactly once.
    pub fn new<'a>(
        owner: &Owner<'a>,
        name: &[u16],
        ms: f64,
        max_units: usize,
        mut clear: impl FnMut(u64) + 'a,
    ) -> Result<Self, StartError>
    where
        T: 'a,
        E: 'a,
    {
        let shared = Rc::new(Shared {
            deadline: RefCell::new(
                Deadline::new(name, ms, max_units).map_err(StartError::Deadline)?,
            ),
            body_done: Cell::new(false),
            cleared: Cell::new(false),
            wake: RefCell::new(None),
            failure: Cell::new(None),
            caller: RefCell::new(None),
            claimed: Cell::new(false),
            consumed: Cell::new(false),
        });
        let task = shared.clone();
        // Drop the observation ticket immediately: only Owner owns task lifetime.
        drop(
            owner
                .spawn(poll_fn(move |cx| {
                    *task.wake.borrow_mut() = Some(cx.waker().clone());
                    let timer = task.deadline.borrow_mut().begin_clear();
                    if let Some(timer) = timer {
                        // No deadline borrow across reentrant host effects.
                        clear(timer);
                        if let Err(e) = task.deadline.borrow_mut().cleared() {
                            task.failure.set(Some(e));
                        }
                        task.cleared.set(true);
                        let caller = task.caller.borrow_mut().take();
                        if let Some(caller) = caller {
                            caller.wake()
                        }
                    }
                    if task.cleared.get() && task.body_done.get() {
                        task.wake.borrow_mut().take();
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }))
                .map_err(StartError::Owner)?,
        );
        Ok(Self { shared })
    }
    /// Host timer acquired before eager body factory invocation.
    pub fn registered(&self, timer: u64) -> Result<(), DeadlineError> {
        self.shared.deadline.borrow_mut().registered(timer)
    }
    /// Actual body settlement, including late faults; a wake is NOT settlement.
    /// Duplicate values are consumed without another caller response.
    pub fn body_settled(&self, result: Result<T, E>) -> bool {
        let disposition = self.shared.deadline.borrow_mut().settle_owned(result);
        let accepted = disposition.is_ok();
        self.shared.body_done.set(true);
        self.shared.wake();
        // A late/duplicate fault may release resources that inspect the runtime.
        // Its destructor must never run with a RefMut<Deadline> still live.
        drop(disposition);
        accepted
    }
    /// Attach the race after the factory returns, including converted synchronous throws.
    pub fn factory_returned(&self) -> Result<(), DeadlineError> {
        self.shared.deadline.borrow_mut().factory_returned()?;
        self.shared.wake();
        Ok(())
    }
    /// Begin timer callback; dispatch synchronous abort listeners before fire_end.
    pub fn fire_begin(&self) -> Result<Option<AbortEvent>, DeadlineError> {
        self.shared.deadline.borrow_mut().fire_begin()
    }
    /// Reject late after abort delivery; finally runs on a later owner turn.
    pub fn fire_end(&self, event: AbortEvent) -> Result<(), DeadlineError> {
        self.shared.deadline.borrow_mut().fire_end(event)?;
        self.shared.wake();
        Ok(())
    }
    /// Shared signal reason, even when body wins.
    pub fn abort_reason(&self) -> Option<Arc<CommandError>> {
        self.shared.deadline.borrow().abort_reason().cloned()
    }
    /// Whether timer callback could still run.
    pub fn timer_active(&self) -> bool {
        self.shared.deadline.borrow().timer_active()
    }
    /// Transfer caller outcome once, after owned finally cleanup. This cannot cancel body work.
    pub fn take_outcome(&self) -> Result<Option<Completion<T, E>>, DeadlineError> {
        if self.shared.claimed.get() {
            return Err(DeadlineError::InvalidState);
        }
        if let Some(e) = self.shared.failure.get() {
            return Err(e);
        }
        let result = self.shared.deadline.borrow_mut().take_outcome();
        if result.is_some() {
            self.shared.consumed.set(true);
        }
        Ok(result)
    }
    /// Admit the sole owned caller future, allowing real queues/execute to await cleanup.
    /// This excludes manual take_outcome and a second observer (outputs need not be Clone).
    /// Dropping this observer cannot cancel the owner or late body work.
    pub fn observe(&self) -> Result<Outcome<T, E>, DeadlineError> {
        if self.shared.consumed.get() || self.shared.claimed.replace(true) {
            return Err(DeadlineError::InvalidState);
        }
        Ok(Outcome {
            shared: self.shared.clone(),
            done: false,
        })
    }
}
/// One owned caller continuation. Its waker is distinct from the retained body's driver.
pub struct Outcome<T, E> {
    shared: Rc<Shared<T, E>>,
    done: bool,
}
impl<T, E> Future for Outcome<T, E> {
    type Output = Result<Completion<T, E>, DeadlineError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.done {
            return Poll::Ready(Err(DeadlineError::InvalidState));
        }
        let result = if let Some(error) = this.shared.failure.get() {
            Some(Err(error))
        } else {
            this.shared.deadline.borrow_mut().take_outcome().map(Ok)
        };
        if let Some(result) = result {
            this.done = true;
            this.shared.consumed.set(true);
            this.shared.caller.borrow_mut().take();
            Poll::Ready(result)
        } else {
            *this.shared.caller.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
/// Eagerly invoke an admitted provider factory, asynchronously resume its returned work.
///
/// Execute preflight/stop/settle begins in the factory; await continuation resumes on a later
/// owner turn even for ready work. Deliver the exact outcome once to the captured receiver.
/// A connection-specific caller can capture its own connection, avoiding globally reused IDs.
/// Ticket drop cannot cancel work. Pending providers must arrange wakes; no busy polling or
/// arbitrary number of microtask flushes is used.
pub fn retain<'a, F: Future + 'a>(
    owner: &Owner<'a>,
    factory: impl FnOnce() -> F + 'a,
    receive: impl FnOnce(F::Output) + 'a,
) -> Result<Ticket, tasks::Error> {
    let mut factory = Some(factory);
    let mut run = None;
    let mut receive = Some(receive);
    owner.spawn(poll_fn(move |cx| {
        if let Some(factory) = factory.take() {
            run = Some(Box::pin(factory()));
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let Some(run) = &mut run else {
            return Poll::Pending;
        };
        match run.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => {
                if let Some(receive) = receive.take() {
                    receive(output)
                }
                Poll::Ready(())
            }
        }
    }))
}
