// SPDX-License-Identifier: AGPL-3.0-only
//! Session store-source ownership from watchSessionStores, before subscription.
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
/// Readonly shared array of store identities. Duplicates and order are preserved.
pub type StoreList<S> = Rc<[Rc<S>]>;
struct Inner<S: ?Sized> {
    current: RefCell<StoreList<S>>,
}
/// Session-owned source array. This does not call any store's subscribe method.
pub struct StoreSources<S: ?Sized> {
    inner: Rc<Inner<S>>,
}
impl<S: ?Sized> Clone for StoreSources<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Explicit returned release. Dropping it does not release the source array.
pub struct Release<S: ?Sized> {
    inner: Weak<Inner<S>>,
    stores: StoreList<S>,
}
impl<S: ?Sized> Release<S> {
    /// Clear only if the exact captured array is still current. Reintroducing
    /// that same array makes an old release effective again. A successful clear
    /// creates a fresh empty array; repeated release then does nothing.
    pub fn release(&self) {
        if let Some(inner) = self.inner.upgrade() {
            let mut current = inner.current.borrow_mut();
            if Rc::ptr_eq(&current, &self.stores) {
                *current = Rc::from(Vec::<Rc<S>>::new())
            }
        }
    }
}
impl<S: ?Sized> Default for StoreSources<S> {
    fn default() -> Self {
        Self::new()
    }
}
impl<S: ?Sized> StoreSources<S> {
    /// Fresh session with its own empty source array.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner {
                current: RefCell::new(Rc::from(Vec::<Rc<S>>::new())),
            }),
        }
    }
    /// Current original array identity, without invoking any store method.
    pub fn current(&self) -> StoreList<S> {
        self.inner.current.borrow().clone()
    }
    /// Install the exact supplied array and return source-compatible cleanup.
    pub fn configure(&self, stores: StoreList<S>) -> Release<S> {
        *self.inner.current.borrow_mut() = stores.clone();
        Release {
            inner: Rc::downgrade(&self.inner),
            stores,
        }
    }
}
