// SPDX-License-Identifier: AGPL-3.0-only
//! Ordered plugin busy metadata from registerBusy/pluginBusy. Stop invocation is separate.
use crate::watch_stores::StoreList;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
/// A source string, including unpaired UTF-16 code units.
pub type Text = Rc<[u16]>;
/// Check identity is the Rc allocation. Errors are suppressed by pluginBusy.
pub type Check<'a, E> = Rc<dyn Fn() -> Result<Option<Text>, E> + 'a>;
/// Host bounds are distinct from source duplicate-id errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit;
/// Registration refuses duplicates before checking host resource limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterError {
    /// The exact already-registered id.
    Duplicate(Text),
    /// Explicit host storage/id budget or allocation refusal.
    Limit,
}
impl RegisterError {
    /// Exact original duplicate message; host allocation failure stays separate.
    pub fn message(&self) -> Result<Option<Vec<u16>>, Limit> {
        let Self::Duplicate(id) = self else {
            return Ok(None);
        };
        let mut text = Vec::new();
        text.try_reserve(id.len().checked_add(40).ok_or(Limit)?)
            .map_err(|_| Limit)?;
        text.extend("Two busy checks share the id \"".encode_utf16());
        text.extend(id.iter().copied());
        text.extend("\".".encode_utf16());
        Ok(Some(text))
    }
}
/// Explicit session budgets; slots include deleted entries to preserve live iteration.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Retained slots, including tombstones.
    pub slots: usize,
    /// Maximum callback invocations per lookup.
    pub visits: usize,
    /// Maximum id UTF-16 units for a new registration.
    pub id_units: usize,
    /// Maximum nested lookup depth.
    pub depth: usize,
}
/// Snapshot metadata; store arrays and optional stop handles retain exact identities.
pub struct Entry<'a, S: ?Sized, T: ?Sized, E> {
    /// Original UTF-16 id.
    pub id: Text,
    /// Registered function allocation identity.
    pub check: Check<'a, E>,
    /// Readonly store-array allocation identity.
    pub stores: StoreList<S>,
    /// Optional opaque stop-function identity; never invoked by this unit.
    pub stop: Option<Rc<T>>,
}
impl<S: ?Sized, T: ?Sized, E> Clone for Entry<'_, S, T, E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            check: self.check.clone(),
            stores: self.stores.clone(),
            stop: self.stop.clone(),
        }
    }
}
struct Inner<'a, S: ?Sized, T: ?Sized, E> {
    entries: RefCell<Vec<Option<Entry<'a, S, T, E>>>>,
    limits: Limits,
    depth: Cell<usize>,
}
/// Shared ordered registry, safe for callback reentry without borrowing across callbacks.
pub struct Plugins<'a, S: ?Sized, T: ?Sized, E> {
    inner: Rc<Inner<'a, S, T, E>>,
}
impl<S: ?Sized, T: ?Sized, E> Clone for Plugins<'_, S, T, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Repeatable source cleanup. Drop does nothing; handles do not retain a dead registry.
pub struct Release<'a, S: ?Sized, T: ?Sized, E> {
    inner: Weak<Inner<'a, S, T, E>>,
    id: Text,
    check: Check<'a, E>,
}
impl<S: ?Sized, T: ?Sized, E> Release<'_, S, T, E> {
    /// Match the original function identity, even after removal/re-registration.
    pub fn release(&self) {
        if let Some(inner) = self.inner.upgrade() {
            for slot in inner.entries.borrow_mut().iter_mut() {
                if slot
                    .as_ref()
                    .is_some_and(|e| e.id == self.id && Rc::ptr_eq(&e.check, &self.check))
                {
                    *slot = None;
                    break;
                }
            }
        }
    }
}
impl<'a, S: ?Sized, T: ?Sized, E> Plugins<'a, S, T, E> {
    /// Start an empty session registry with explicit host bounds.
    pub fn new(limits: Limits) -> Self {
        Self {
            inner: Rc::new(Inner {
                entries: RefCell::new(Vec::new()),
                limits,
                depth: Cell::new(0),
            }),
        }
    }
    /// Preserve insertion order, array identity, stop identity and source cleanup semantics.
    pub fn register(
        &self,
        id: Text,
        check: Check<'a, E>,
        stores: StoreList<S>,
        stop: Option<Rc<T>>,
    ) -> Result<Release<'a, S, T, E>, RegisterError> {
        let mut entries = self.inner.entries.borrow_mut();
        if entries.iter().flatten().any(|e| e.id == id) {
            return Err(RegisterError::Duplicate(id));
        }
        if entries.len() >= self.inner.limits.slots || id.len() > self.inner.limits.id_units {
            return Err(RegisterError::Limit);
        }
        entries.try_reserve(1).map_err(|_| RegisterError::Limit)?;
        entries.push(Some(Entry {
            id: id.clone(),
            check: check.clone(),
            stores,
            stop,
        }));
        Ok(Release {
            inner: Rc::downgrade(&self.inner),
            id,
            check,
        })
    }
    /// Snapshot all three ordered maps without invoking checks or stops.
    pub fn snapshot(&self) -> Result<Vec<Entry<'a, S, T, E>>, Limit> {
        let entries = self.inner.entries.borrow();
        let mut out = Vec::new();
        out.try_reserve(entries.iter().flatten().count())
            .map_err(|_| Limit)?;
        out.extend(entries.iter().flatten().cloned());
        Ok(out)
    }
    /// First truthy message. Empty strings and check errors are ignored. New entries
    /// appended by a check are visited; deleted future entries are skipped.
    pub fn busy(&self) -> Result<Option<Text>, Limit> {
        if self.inner.depth.get() >= self.inner.limits.depth {
            return Err(Limit);
        }
        struct Depth<'a>(&'a Cell<usize>);
        impl Drop for Depth<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() - 1);
            }
        }
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        let mut index = 0;
        let mut visits = 0;
        loop {
            let check = {
                let entries = self.inner.entries.borrow();
                if index == entries.len() {
                    return Ok(None);
                }
                entries[index].as_ref().map(|e| e.check.clone())
            };
            index += 1;
            if let Some(check) = check {
                if visits >= self.inner.limits.visits {
                    return Err(Limit);
                }
                visits += 1;
                if let Ok(Some(text)) = check()
                    && !text.is_empty()
                {
                    return Ok(Some(text));
                }
            }
        }
    }
}
