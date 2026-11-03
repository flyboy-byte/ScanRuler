// SPDX-License-Identifier: AGPL-3.0-only
//! Command-driven activity patches. Delivery follows Zustand's MIT-licensed
//! vanilla store algorithm; attribution: rust/licenses/ZUSTAND-MIT.txt.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
/// Original running command payload, shared unchanged through progress patches.
#[derive(Debug)]
pub struct Running {
    /// Lossless command name.
    pub name: Rc<[u16]>,
    /// Lossless display title.
    pub title: Rc<[u16]>,
    /// Raw sampled epoch milliseconds, without Date clipping.
    pub since: f64,
}
/// Immutable published snapshot. Previous snapshots stay valid.
#[derive(Debug)]
pub struct State {
    /// Current command, or source null.
    pub running: Option<Rc<Running>>,
    /// Latest progress, including a valid empty string, or source null.
    pub progress: Option<Rc<[u16]>>,
}
/// The three patch shapes actually issued by guarded execution.
pub enum Patch {
    /// Set running payload and clear progress.
    Start(Rc<Running>),
    /// Preserve running payload and set progress.
    Progress(Rc<[u16]>),
    /// Clear both fields, even if already clear.
    Clear,
}
/// Finite host admission/work bounds, separate from source callback faults.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Live/tombstoned listener slots during a notification chain.
    pub slots: usize,
    /// Callback visits across the complete reentrant notification chain.
    pub visits: usize,
    /// Simultaneously nested publications.
    pub depth: usize,
    /// Combined start text or individual progress units.
    pub text_units: usize,
}
/// Failure leaves previously completed publications and callbacks visible.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// Exact listener exception, without normalization.
    Listener(E),
    /// Host slot, visit, recursion or input bound.
    Limit,
    /// Fallible listener-list reservation failed.
    Allocation,
}
/// Callback identity is the Rc identity. No mutable callback borrow spans calls.
pub type Listener<'a, E> = Rc<dyn Fn(&Activity<'a, E>, Rc<State>, Rc<State>) -> Result<(), E> + 'a>;
struct Inner<'a, E> {
    state: RefCell<Rc<State>>,
    initial: Rc<State>,
    listeners: RefCell<Vec<Option<Listener<'a, E>>>>,
    depth: Cell<usize>,
    visits: Cell<usize>,
    limits: Limits,
}
/// Session-owned activity state. Clones share exactly the same state/listeners.
pub struct Activity<'a, E> {
    inner: Rc<Inner<'a, E>>,
}
impl<'a, E> Clone for Activity<'a, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Explicit cleanup. Dropping it does not unsubscribe. Old releases remove a
/// later subscription of the same callback, matching Set.delete identity.
pub struct Release<'a, E> {
    inner: Weak<Inner<'a, E>>,
    callback: Listener<'a, E>,
}
impl<E> Release<'_, E> {
    /// Remove the currently live entry of this exact callback, if present.
    pub fn release(&self) {
        if let Some(inner) = self.inner.upgrade() {
            for entry in inner.listeners.borrow_mut().iter_mut() {
                if entry
                    .as_ref()
                    .is_some_and(|c| Rc::ptr_eq(c, &self.callback))
                {
                    *entry = None;
                    break;
                }
            }
        }
    }
}
struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl<'a, E> Activity<'a, E> {
    /// Fresh store with null running/progress fields and a retained initial state.
    pub fn new(limits: Limits) -> Self {
        let state = Rc::new(State {
            running: None,
            progress: None,
        });
        Self {
            inner: Rc::new(Inner {
                state: RefCell::new(state.clone()),
                initial: state,
                listeners: RefCell::new(vec![]),
                depth: Cell::new(0),
                visits: Cell::new(0),
                limits,
            }),
        }
    }
    /// Current state identity; no copy of fields or callbacks.
    pub fn current(&self) -> Rc<State> {
        self.inner.state.borrow().clone()
    }
    /// Original initial identity, even after publications.
    pub fn initial(&self) -> Rc<State> {
        self.inner.initial.clone()
    }
    /// Exactly running !== null; empty command names remain running.
    pub fn command_running(&self) -> bool {
        self.inner.state.borrow().running.is_some()
    }
    /// Add only absent callback identities. Subscription is not an initial emit.
    pub fn subscribe(&self, callback: Listener<'a, E>) -> Result<Release<'a, E>, Error<E>> {
        let mut entries = self.inner.listeners.borrow_mut();
        if !entries.iter().flatten().any(|c| Rc::ptr_eq(c, &callback)) {
            if self.inner.depth.get() == 0 {
                entries.retain(Option::is_some)
            }
            if entries.len() >= self.inner.limits.slots {
                return Err(Error::Limit);
            }
            entries.try_reserve(1).map_err(|_| Error::Allocation)?;
            entries.push(Some(callback.clone()));
        }
        Ok(Release {
            inner: Rc::downgrade(&self.inner),
            callback,
        })
    }
    /// Publish before notifying. Every patch creates a new snapshot, even if the
    /// field values match. Each callback sees the latest current state plus the
    /// fixed previous snapshot of this publication. Reentrant calls are allowed.
    pub fn publish(&self, patch: Patch) -> Result<(), Error<E>> {
        let limits = self.inner.limits;
        let text = match &patch {
            Patch::Start(r) => r
                .name
                .len()
                .checked_add(r.title.len())
                .ok_or(Error::Limit)?,
            Patch::Progress(p) => p.len(),
            Patch::Clear => 0,
        };
        if text > limits.text_units || self.inner.depth.get() >= limits.depth {
            return Err(Error::Limit);
        }
        if self.inner.depth.get() == 0 {
            self.inner.visits.set(0)
        }
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        let previous = self.current();
        let (running, progress) = match patch {
            Patch::Start(r) => (Some(r), None),
            Patch::Progress(p) => (previous.running.clone(), Some(p)),
            Patch::Clear => (None, None),
        };
        *self.inner.state.borrow_mut() = Rc::new(State { running, progress });
        let mut cursor = 0;
        loop {
            // Clone one live callback then release the list borrow before calling.
            let entry = {
                let entries = self.inner.listeners.borrow();
                entries.get(cursor).cloned()
            };
            let Some(entry) = entry else { break };
            cursor += 1;
            if let Some(callback) = entry {
                if self.inner.visits.get() >= limits.visits {
                    return Err(Error::Limit);
                }
                self.inner.visits.set(self.inner.visits.get() + 1);
                callback(self, self.current(), previous.clone()).map_err(Error::Listener)?;
            }
        }
        Ok(())
    }
}
