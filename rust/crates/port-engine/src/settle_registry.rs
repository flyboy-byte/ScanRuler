// SPDX-License-Identifier: AGPL-3.0-only
//! Function identity and live Set iteration for deferred-work settlement.
use crate::settle::{SettleRun, Work};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

/// Fresh immutable metadata for each invocation; name storage may outlive start.
#[derive(Clone)]
pub struct Next {
    /// Original complete UTF-16 command name.
    pub name: Rc<[u16]>,
    /// Original Boolean(command.readOnly).
    pub read_only: bool,
}
/// Shared function identity. Interior host state may change through owned cells.
pub type Callback<'a, E> = Rc<dyn Fn(Next) -> Result<Option<Work<'a, E>>, E> + 'a>;
/// Bounded-host failure; never a swallowed source callback exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryError {
    /// Total registration slots, including removed slots, exhausted.
    Registrations,
    /// Reentrant start nesting exceeds the host bound.
    Depth,
    /// Command-name text exceeds the host bound.
    Name,
    /// Fallible metadata storage reservation failed.
    Allocation,
}
/// Explicit finite metadata and work bounds.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Total insertion slots; removals retain bounded iteration tombstones.
    pub registrations: usize,
    /// Maximum nested start calls.
    pub depth: usize,
    /// Maximum command-name UTF-16 units.
    pub name_units: usize,
}
struct Slot<'a, E> {
    callback: Callback<'a, E>,
    active: Cell<bool>,
}
struct Inner<'a, E> {
    slots: RefCell<Vec<Rc<Slot<'a, E>>>>,
    depth: Cell<usize>,
    limits: Limits,
}
/// Session-owned callback Set. Clones refer to this same Set.
pub struct Settlers<'a, E> {
    inner: Rc<Inner<'a, E>>,
}
impl<E> Clone for Settlers<'_, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Weak handle for callbacks that mutate registration without an ownership cycle.
pub struct WeakSettlers<'a, E> {
    inner: Weak<Inner<'a, E>>,
}
impl<E> Clone for WeakSettlers<'_, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<'a, E> WeakSettlers<'a, E> {
    /// Recover the registry while its session owner remains alive.
    pub fn upgrade(&self) -> Option<Settlers<'a, E>> {
        self.inner.upgrade().map(|inner| Settlers { inner })
    }
}
/// Returned explicit unsubscribe capability. Dropping it does not unsubscribe.
/// An old capability also removes later registrations of its same function.
pub struct Subscription<'a, E> {
    inner: Weak<Inner<'a, E>>,
    callback: Callback<'a, E>,
}
impl<E> Subscription<'_, E> {
    /// Remove this function identity from the live Set; repeated calls are safe.
    pub fn unsubscribe(&self) {
        if let Some(inner) = self.inner.upgrade() {
            for slot in inner.slots.borrow().iter() {
                if Rc::ptr_eq(&slot.callback, &self.callback) {
                    slot.active.set(false)
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
impl<'a, E> Settlers<'a, E> {
    /// Empty callback Set under explicit host bounds.
    pub fn new(limits: Limits) -> Self {
        Self {
            inner: Rc::new(Inner {
                slots: RefCell::new(Vec::new()),
                depth: Cell::new(0),
                limits,
            }),
        }
    }
    /// Weak handle suitable for a reentrant callback capture.
    pub fn downgrade(&self) -> WeakSettlers<'a, E> {
        WeakSettlers {
            inner: Rc::downgrade(&self.inner),
        }
    }
    /// Add by function identity; duplicate adds do not reorder the live Set.
    pub fn register(
        &self,
        callback: Callback<'a, E>,
    ) -> Result<Subscription<'a, E>, RegistryError> {
        let mut slots = self.inner.slots.borrow_mut();
        if !slots
            .iter()
            .any(|s| s.active.get() && Rc::ptr_eq(&s.callback, &callback))
        {
            if slots.len() >= self.inner.limits.registrations {
                return Err(RegistryError::Registrations);
            }
            slots
                .try_reserve(1)
                .map_err(|_| RegistryError::Allocation)?;
            slots.push(Rc::new(Slot {
                callback: callback.clone(),
                active: Cell::new(true),
            }));
        }
        Ok(Subscription {
            inner: Rc::downgrade(&self.inner),
            callback,
        })
    }
    /// Invoke the live Set. Deleted unvisited callbacks are skipped; newly added
    /// and deleted/re-added functions are visited later in this same iteration.
    /// Host metadata is admitted before calls; source callback errors are ignored.
    /// The returned run owns all started waits and must be retained and drained.
    pub fn start(
        &self,
        name: &[u16],
        read_only: bool,
    ) -> Result<Option<SettleRun<'a, E>>, RegistryError> {
        if self.inner.depth.get() >= self.inner.limits.depth {
            return Err(RegistryError::Depth);
        }
        if name.len() > self.inner.limits.name_units {
            return Err(RegistryError::Name);
        }
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(name.len())
            .map_err(|_| RegistryError::Allocation)?;
        owned.extend_from_slice(name);
        let name: Rc<[u16]> = Rc::from(owned);
        let mut waits = Vec::new();
        waits
            .try_reserve_exact(self.inner.limits.registrations)
            .map_err(|_| RegistryError::Allocation)?;
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        let mut index = 0;
        loop {
            let slot = self.inner.slots.borrow().get(index).cloned();
            let Some(slot) = slot else { break };
            index += 1;
            if slot.active.get()
                && let Ok(Some(wait)) = (slot.callback)(Next {
                    name: name.clone(),
                    read_only,
                })
            {
                waits.push(Some(wait))
            }
        }
        Ok(SettleRun::from_started(waits))
    }
}
