// SPDX-License-Identifier: AGPL-3.0-only
//! Complete execute orchestration at owned stop/settle/guarded provider boundaries.
use crate::settle::Work;
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};
struct Waiter {
    waker: RefCell<Option<Waker>>,
}
struct Completion {
    resolved: Cell<bool>,
    waiters: RefCell<Vec<Weak<Waiter>>>,
    limit: usize,
}
/// Shared completion capability. Only execution's finally resolves it. Dropping
/// a clone or a waiter does not resolve the promise.
#[derive(Clone)]
pub struct Done {
    inner: Rc<Completion>,
}
impl PartialEq for Done {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Done {}
/// Separate host admission failure for a native completion observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneError {
    /// Total observer slots exhausted; includes removed observers.
    Waiters,
    /// Fallible observer-list reservation failed.
    Allocation,
}
/// A single admitted observer. Each observer keeps its own waker.
#[must_use = "retain and poll a completion observer when waiting on done"]
pub struct DoneWait {
    done: Done,
    waiter: Option<Rc<Waiter>>,
}
impl Future for DoneWait {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.done.resolved() {
            Poll::Ready(())
        } else {
            if let Some(waiter) = &self.waiter {
                *waiter.waker.borrow_mut() = Some(cx.waker().clone())
            }
            Poll::Pending
        }
    }
}
impl Done {
    fn new(limit: usize) -> Self {
        Self {
            inner: Rc::new(Completion {
                resolved: Cell::new(false),
                waiters: RefCell::new(Vec::new()),
                limit,
            }),
        }
    }
    /// Whether the owning execute finally has run.
    pub fn resolved(&self) -> bool {
        self.inner.resolved.get()
    }
    /// Admit an independent native observer without losing another observer's wake.
    pub fn wait(&self) -> Result<DoneWait, DoneError> {
        if self.resolved() {
            return Ok(DoneWait {
                done: self.clone(),
                waiter: None,
            });
        }
        let mut waiters = self.inner.waiters.borrow_mut();
        if waiters.len() >= self.inner.limit {
            return Err(DoneError::Waiters);
        }
        waiters.try_reserve(1).map_err(|_| DoneError::Allocation)?;
        let waiter = Rc::new(Waiter {
            waker: RefCell::new(None),
        });
        waiters.push(Rc::downgrade(&waiter));
        Ok(DoneWait {
            done: self.clone(),
            waiter: Some(waiter),
        })
    }
    fn finish(&self) {
        if self.inner.resolved.replace(true) {
            return;
        }
        let waiters = std::mem::take(&mut *self.inner.waiters.borrow_mut());
        for waiter in waiters.into_iter().filter_map(|w| w.upgrade()) {
            let wake = waiter.waker.borrow_mut().take();
            if let Some(wake) = wake {
                wake.wake()
            }
        }
    }
}
/// Guarded provider work retains the exact result/error value.
pub type Body<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;
/// Owned providers, not a duplicate session store. Discards classifies strict
/// source `===true`; busy preserves null separately from any string, even empty.
/// Provider futures must already be started as their factory returns.
pub trait Host<'a> {
    /// Complete command outcome, without a Clone requirement.
    type Value;
    /// Exact source/provider fault, without Display conversion.
    type Error;
    /// Optional discards function; absent or non-true values classify false.
    fn discards(&mut self) -> Result<bool, Self::Error>;
    /// Raw guard answer; an empty Some still selects stopping in preflight.
    fn busy(&mut self) -> Result<Option<Rc<[u16]>>, Self::Error>;
    /// Start stopping, passing the exact shared done capability.
    fn stop(&mut self, done: Done) -> Work<'a, Self::Error>;
    /// Start settlement, including its source null/ready-await branch.
    fn settle(&mut self) -> Result<Work<'a, Self::Error>, Self::Error>;
    /// Start the separately owned guarded command provider with original deferred.
    fn guarded(&mut self, deferred: bool) -> Body<'a, Self::Value, Self::Error>;
}
/// Source faults retain their identity; repeated native polling is a host misuse.
#[derive(Debug, PartialEq, Eq)]
pub enum ExecuteError<E> {
    /// Original exact provider/source fault.
    Callback(E),
    /// A completed single-use execution was polled again.
    Completed,
}
enum Phase {
    Before,
    Body,
    Ready,
}
/// Own every started provider and body until finally. The executor must retain
/// and drain this run: Drop cannot emulate continuing original promises.
#[must_use = "retain and drain execute through its finally"]
pub struct Execution<'a, H: Host<'a>> {
    host: Box<H>,
    done: Option<Done>,
    deferred: bool,
    phase: Phase,
    before: Option<Work<'a, H::Error>>,
    body: Option<Body<'a, H::Value, H::Error>>,
    ready: Option<Box<Result<H::Value, H::Error>>>,
}
impl<'a, H: Host<'a>> Execution<'a, H> {
    fn complete(&mut self, result: Result<H::Value, H::Error>) {
        if let Some(done) = &self.done {
            done.finish()
        }
        self.ready = Some(Box::new(result));
        self.phase = Phase::Ready;
    }
    fn start_body(&mut self) {
        self.body = Some(self.host.guarded(self.deferred));
        self.phase = Phase::Body;
    }
    /// Completion capability exists only after successful preflight. Observation
    /// does not resolve it or alter provider selection.
    pub fn done(&self) -> Option<Done> {
        self.done.clone()
    }
}
/// Run preflight immediately, before constructing done or entering the source
/// try/finally. Stop/settle starts immediately; deferred changes start guarded
/// immediately. Subsequent awaits remain owned by the returned execution.
pub fn execute<'a, H: Host<'a>>(
    host: H,
    deferred: bool,
    max_done_waiters: usize,
) -> Execution<'a, H> {
    let mut run = Execution {
        host: Box::new(host),
        done: None,
        deferred,
        phase: Phase::Ready,
        before: None,
        body: None,
        ready: None,
    };
    let stops = match run.host.discards().and_then(|discards| {
        if discards {
            run.host.busy().map(|busy| busy.is_some())
        } else {
            Ok(false)
        }
    }) {
        Ok(stops) => stops,
        Err(e) => {
            run.ready = Some(Box::new(Err(e)));
            return run;
        }
    };
    let done = Done::new(max_done_waiters);
    run.done = Some(done.clone());
    if stops || !deferred {
        match if stops {
            Ok(run.host.stop(done))
        } else {
            run.host.settle()
        } {
            Ok(before) => {
                run.before = Some(before);
                run.phase = Phase::Before
            }
            Err(e) => run.complete(Err(e)),
        }
    } else {
        run.start_body()
    }
    run
}
impl<'a, H: Host<'a>> Future for Execution<'a, H> {
    type Output = Result<H::Value, ExecuteError<H::Error>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        loop {
            match this.phase {
                Phase::Before => {
                    let Some(before) = &mut this.before else {
                        return Poll::Pending;
                    };
                    match before.as_mut().poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(result) => {
                            this.before = None;
                            match result {
                                Ok(()) => this.start_body(),
                                Err(e) => this.complete(Err(e)),
                            }
                        }
                    }
                }
                Phase::Body => {
                    let Some(body) = &mut this.body else {
                        return Poll::Pending;
                    };
                    match body.as_mut().poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(result) => {
                            this.body = None;
                            this.complete(result)
                        }
                    }
                }
                Phase::Ready => {
                    return Poll::Ready(
                        this.ready.take().map_or(Err(ExecuteError::Completed), |v| {
                            (*v).map_err(ExecuteError::Callback)
                        }),
                    );
                }
            }
        }
    }
}
