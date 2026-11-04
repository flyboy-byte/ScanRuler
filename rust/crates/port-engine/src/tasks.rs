// SPDX-License-Identifier: AGPL-3.0-only
//! Cooperative host ownership, independent of caller tickets. This is Rust host
//! machinery, not a JavaScript microtask model or a command-runtime parity claim.
use std::{
    cell::Cell,
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};
/// Session host quotas, unrelated to source command timeout policies.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum retained pending/running futures.
    pub tasks: usize,
    /// Maximum eager first-poll nesting through reentrant spawn calls.
    pub depth: usize,
    /// Maximum polls in one explicit pump turn.
    pub polls_per_turn: usize,
}
/// Admission/driver failures, never substituted for an application fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// No task slot, recursion budget, or valid nonzero turn/depth budget.
    Limit,
    /// Fallible queue/slot reservation failed.
    Allocation,
    /// Task identity counter cannot advance without wrapping.
    Identity,
    /// A non-idle owner cannot finish.
    Busy,
    /// An explicitly finished owner cannot admit more work.
    Closed,
    /// Pumping from inside a task poll would overlap future polling.
    Reentrant,
    /// Wake-queue invariant failed; driver is quarantined.
    Queue,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Waiting,
    Running,
    Complete,
}
/// Observation only. Dropping every ticket cannot cancel the owner's future.
#[derive(Clone)]
pub struct Ticket {
    status: Rc<Cell<Status>>,
}
impl Ticket {
    /// True only after the retained future returned Ready, not caller timeout.
    pub fn complete(&self) -> bool {
        self.status.get() == Status::Complete
    }
}
struct Ready {
    entries: VecDeque<std::sync::Weak<Signal>>,
    closed: bool,
    broken: bool,
    capacity: usize,
}
struct Queue(Mutex<Ready>);
impl Queue {
    fn lock(&self) -> std::sync::MutexGuard<'_, Ready> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
struct Signal {
    queue: Arc<Queue>,
    index: usize,
    identity: u64,
    live: AtomicBool,
    queued: AtomicBool,
}
impl Signal {
    fn schedule(self: &Arc<Self>) {
        let mut queue = self.queue.lock();
        if queue.closed || !self.live.load(Ordering::Acquire) || self.queued.load(Ordering::Relaxed)
        {
            return;
        }
        // At most one token for each admitted live slot. All queue storage was
        // reserved at construction; Wake never runs application code/allocates.
        if queue.entries.len() >= queue.capacity {
            queue.broken = true;
            return;
        }
        self.queued.store(true, Ordering::Release);
        queue.entries.push_back(Arc::downgrade(self));
    }
    fn retire(&self) {
        self.live.store(false, Ordering::Release);
        let mut queue = self.queue.lock();
        self.queued.store(false, Ordering::Release);
        queue.entries.retain(|weak| {
            weak.upgrade()
                .is_some_and(|s| s.live.load(Ordering::Acquire))
        });
    }
}
impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.schedule()
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule()
    }
}
type Work<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;
struct Slot<'a> {
    future: Option<Work<'a>>,
    signal: Arc<Signal>,
    status: Rc<Cell<Status>>,
}
struct Inner<'a> {
    slots: RefCell<Vec<Option<Slot<'a>>>>,
    queue: Arc<Queue>,
    limits: Limits,
    active: Cell<usize>,
    depth: Cell<usize>,
    next: Cell<u64>,
}
impl Drop for Inner<'_> {
    fn drop(&mut self) {
        self.queue.lock().closed = true
    }
}
/// The session owns this driver for its lifetime. Futures may borrow the host and
/// use Rc business data; only thread-safe wake tokens cross a thread boundary.
pub struct Owner<'a> {
    inner: Rc<Inner<'a>>,
}
impl Clone for Owner<'_> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Non-owning runtime capability for task captures. This prevents a retained future from
/// keeping its own Owner alive through a session/core reference during host shutdown.
pub struct WeakOwner<'a> {
    inner: Weak<Inner<'a>>,
}
impl Clone for WeakOwner<'_> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<'a> WeakOwner<'a> {
    /// Upgrade only while the external session still owns its driver.
    pub fn upgrade(&self) -> Option<Owner<'a>> {
        self.inner.upgrade().map(|inner| Owner { inner })
    }
}
/// One finite pump's state. Waiting work is not the same as a finished owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    /// Futures polled during this pump, excluding earlier eager first polls.
    pub polls: usize,
    /// Pending or currently running retained futures.
    pub active: usize,
    /// Deduplicated ready tokens remaining for another turn.
    pub ready: usize,
}
struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl<'a> Owner<'a> {
    /// Give retained task captures a non-owning driver reference.
    pub fn downgrade(&self) -> WeakOwner<'a> {
        WeakOwner {
            inner: Rc::downgrade(&self.inner),
        }
    }
    /// Reserve all slot/wake metadata before accepting work. Future/Rc/Arc
    /// allocations are ordinary Rust allocations, not process-wide OOM isolation.
    pub fn new(limits: Limits) -> Result<Self, Error> {
        if limits.depth == 0 || limits.polls_per_turn == 0 {
            return Err(Error::Limit);
        }
        let mut entries = VecDeque::new();
        entries
            .try_reserve_exact(limits.tasks)
            .map_err(|_| Error::Allocation)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(limits.tasks)
            .map_err(|_| Error::Allocation)?;
        Ok(Self {
            inner: Rc::new(Inner {
                slots: RefCell::new(slots),
                queue: Arc::new(Queue(Mutex::new(Ready {
                    entries,
                    closed: false,
                    broken: false,
                    capacity: limits.tasks,
                }))),
                limits,
                active: Cell::new(0),
                depth: Cell::new(0),
                next: Cell::new(0),
            }),
        })
    }
    /// Retain and immediately first-poll this work. A refused future has never
    /// been polled. Work/result handling belongs to the admitted future; a ticket
    /// carries no cancellation power. Reentrant submission is depth-bounded.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'a) -> Result<Ticket, Error> {
        {
            let queue = self.inner.queue.lock();
            if queue.broken {
                return Err(Error::Queue);
            }
            if queue.closed {
                return Err(Error::Closed);
            }
        }
        if self.inner.active.get() >= self.inner.limits.tasks
            || self.inner.depth.get() >= self.inner.limits.depth
        {
            return Err(Error::Limit);
        }
        let identity = self
            .inner
            .next
            .get()
            .checked_add(1)
            .ok_or(Error::Identity)?;
        let mut slots = self.inner.slots.borrow_mut();
        let index = slots
            .iter()
            .position(Option::is_none)
            .unwrap_or(slots.len());
        let status = Rc::new(Cell::new(Status::Waiting));
        let signal = Arc::new(Signal {
            queue: self.inner.queue.clone(),
            index,
            identity,
            live: AtomicBool::new(true),
            queued: AtomicBool::new(false),
        });
        let slot = Some(Slot {
            future: Some(Box::pin(future)),
            signal: signal.clone(),
            status: status.clone(),
        });
        if index == slots.len() {
            slots.push(slot)
        } else {
            slots[index] = slot
        }
        drop(slots);
        self.inner.next.set(identity);
        self.inner.active.set(self.inner.active.get() + 1);
        self.poll_one(signal);
        Ok(Ticket { status })
    }
    fn poll_one(&self, signal: Arc<Signal>) {
        let (mut future, status) = {
            let mut slots = self.inner.slots.borrow_mut();
            let Some(Some(slot)) = slots.get_mut(signal.index) else {
                return;
            };
            if slot.signal.identity != signal.identity {
                return;
            }
            let Some(future) = slot.future.take() else {
                return;
            };
            (future, slot.status.clone())
        };
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        status.set(Status::Running);
        let waker = Waker::from(signal.clone());
        let mut context = Context::from_waker(&waker);
        match future.as_mut().poll(&mut context) {
            Poll::Pending => {
                status.set(Status::Waiting);
                if let Some(Some(slot)) = self.inner.slots.borrow_mut().get_mut(signal.index) {
                    slot.future = Some(future)
                }
            }
            Poll::Ready(()) => {
                // Drop ready work after releasing slot borrows as its captures
                // may release/submit host work in their own destructors.
                signal.retire();
                status.set(Status::Complete);
                self.inner.slots.borrow_mut()[signal.index] = None;
                self.inner.active.set(self.inner.active.get() - 1);
                drop(future);
            }
        }
    }
    /// Poll at most the configured finite budget, in FIFO wake order. A self-wake
    /// goes to the back and duplicates coalesce. Never blocks waiting for a wake.
    pub fn turn(&self) -> Result<Turn, Error> {
        if self.inner.depth.get() != 0 {
            return Err(Error::Reentrant);
        }
        let mut polls = 0;
        while polls < self.inner.limits.polls_per_turn {
            let next = {
                let mut queue = self.inner.queue.lock();
                if queue.broken {
                    return Err(Error::Queue);
                }
                let next = queue.entries.pop_front().and_then(|weak| weak.upgrade());
                if let Some(signal) = &next {
                    signal.queued.store(false, Ordering::Release)
                }
                next
            };
            let Some(signal) = next else { break };
            if signal.live.load(Ordering::Acquire) {
                self.poll_one(signal);
                polls += 1
            }
        }
        let queue = self.inner.queue.lock();
        if queue.broken {
            return Err(Error::Queue);
        }
        Ok(Turn {
            polls,
            active: self.inner.active.get(),
            ready: queue.entries.len(),
        })
    }
    /// Number of pending/running futures, independent of ready wake tokens.
    pub fn active(&self) -> usize {
        self.inner.active.get()
    }
    /// Close admission only when fully drained. This never cancels live work.
    /// Retain the session owner until this succeeds; dropping the session itself
    /// is runtime shutdown, not the original caller promise's cancellation.
    pub fn finish(&self) -> Result<(), Error> {
        if self.inner.active.get() != 0 {
            return Err(Error::Busy);
        }
        let mut queue = self.inner.queue.lock();
        if queue.broken {
            return Err(Error::Queue);
        }
        queue.closed = true;
        Ok(())
    }
}
