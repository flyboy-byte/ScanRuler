// SPDX-License-Identifier: AGPL-3.0-only
//! Command guard object identity and unconditional configureCommands cleanup.
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

/// Owned host busy callback. Empty text remains distinct from null; faults keep E.
pub type Busy<'a, E> = Rc<dyn Fn() -> Result<Option<Vec<u16>>, E> + 'a>;
/// Shared guard object. Two objects may contain the same callback and still have
/// different identity, exactly as the source's CommandGuard objects do.
pub struct Guard<'a, E> {
    busy: Busy<'a, E>,
}
impl<'a, E> Guard<'a, E> {
    /// Construct an owned guard object around a stable host callback.
    pub fn new(busy: Busy<'a, E>) -> Self {
        Self { busy }
    }
    /// Invoke without normalization, truthiness conversion or catching a fault.
    pub fn busy(&self) -> Result<Option<Vec<u16>>, E> {
        (self.busy)()
    }
}
struct Inner<'a, E> {
    free: Rc<Guard<'a, E>>,
    current: RefCell<Rc<Guard<'a, E>>>,
}
/// Session-owned guard slot; clones operate on the same slot and FREE object.
pub struct Guards<'a, E> {
    inner: Rc<Inner<'a, E>>,
}
impl<E> Clone for Guards<'_, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Weak callback back-reference, avoiding a guard/slot ownership cycle.
pub struct WeakGuards<'a, E> {
    inner: Weak<Inner<'a, E>>,
}
impl<E> Clone for WeakGuards<'_, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<'a, E> WeakGuards<'a, E> {
    /// Recover the live session slot, if its owner remains alive.
    pub fn upgrade(&self) -> Option<Guards<'a, E>> {
        self.inner.upgrade().map(|inner| Guards { inner })
    }
}
/// Explicit returned cleanup. Dropping it does not call reset.
pub struct Reset<'a, E> {
    inner: Weak<Inner<'a, E>>,
}
impl<E> Reset<'_, E> {
    /// Unconditionally restore this session's original FREE object. An old
    /// cleanup resets even a newer configuration; no identity guard is added.
    pub fn reset(&self) {
        if let Some(inner) = self.inner.upgrade() {
            *inner.current.borrow_mut() = inner.free.clone()
        }
    }
}
impl<E> Default for Guards<'_, E> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a, E> Guards<'a, E> {
    /// Fresh session with one FREE object whose busy callback returns null.
    pub fn new() -> Self {
        let free = Rc::new(Guard::new(Rc::new(|| Ok(None))));
        Self {
            inner: Rc::new(Inner {
                current: RefCell::new(free.clone()),
                free,
            }),
        }
    }
    /// Original commandGuard object, without calling its busy function. The slot
    /// borrow ends before the returned guard's callback can reconfigure it.
    pub fn current(&self) -> Rc<Guard<'a, E>> {
        self.inner.current.borrow().clone()
    }
    /// Replace the slot and return the source's explicit unconditional reset.
    pub fn configure(&self, next: Rc<Guard<'a, E>>) -> Reset<'a, E> {
        *self.inner.current.borrow_mut() = next;
        Reset {
            inner: Rc::downgrade(&self.inner),
        }
    }
    /// Weak back-reference suitable for reentrant busy callbacks.
    pub fn downgrade(&self) -> WeakGuards<'a, E> {
        WeakGuards {
            inner: Rc::downgrade(&self.inner),
        }
    }
}
