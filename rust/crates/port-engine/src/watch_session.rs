// SPDX-License-Identifier: AGPL-3.0-only
//! Complete watchSession over owned source arrays and ordered plugin metadata.
use crate::watch_stores::{StoreList, StoreSources};
use std::rc::Rc;
/// Original shared listener identity. The host owns its payload and resource budget.
pub type Listener<'a, E> = Rc<dyn Fn() -> Result<(), E> + 'a>;
/// Explicit unsubscribe callback. Dropping it must not unsubscribe.
pub type Unsubscribe<'a, E> = Rc<dyn Fn() -> Result<(), E> + 'a>;
/// Native store boundary, corresponding to the original Watchable interface.
pub trait Watchable<'a, E> {
    /// Subscribe synchronously, retaining exact listener identity. On failure,
    /// already applied store effects remain. Returned cleanup is explicit only.
    fn subscribe(self: Rc<Self>, listener: Listener<'a, E>) -> Result<Unsubscribe<'a, E>, E>;
}
/// Source callback fault or separate host admission/storage failure.
#[derive(Debug, PartialEq, Eq)]
pub enum WatchError<E> {
    /// Original subscribe/listener fault, with no cloning or normalization.
    Callback(E),
    /// Total incoming references exceed the caller's finite budget/usize.
    Limit,
    /// Fallible deduplication/cleanup metadata reservation failed.
    Allocation,
}
/// Captured ordered cleanup functions. No cleanup runs on Drop.
pub struct WatchStop<'a, E> {
    stops: Vec<Unsubscribe<'a, E>>,
}
impl<E> WatchStop<'_, E> {
    /// Call from the beginning each time, stopping at the first fault. A failure
    /// neither rolls back earlier calls nor skips them on the next invocation.
    pub fn stop(&self) -> Result<(), E> {
        for stop in &self.stops {
            stop()?
        }
        Ok(())
    }
}
/// Snapshot session, also, then plugin members; deduplicate by store identity
/// before invoking any subscribe callback. Pass [] for the source's omitted also.
/// The plugin arrays are the current ordered busyStores.values() snapshot;
/// supplying that view from the native plugin registry remains separate wiring.
pub fn watch_session<'a, E, S: Watchable<'a, E> + ?Sized>(
    session: &StoreSources<S>,
    also: &[Rc<S>],
    plugins: &[StoreList<S>],
    listener: Listener<'a, E>,
    max_references: usize,
) -> Result<WatchStop<'a, E>, WatchError<E>> {
    let current = session.current();
    let mut count = current
        .len()
        .checked_add(also.len())
        .ok_or(WatchError::Limit)?;
    for stores in plugins {
        count = count.checked_add(stores.len()).ok_or(WatchError::Limit)?
    }
    if count > max_references {
        return Err(WatchError::Limit);
    }
    let mut unique: Vec<Rc<S>> = Vec::new();
    unique
        .try_reserve_exact(count)
        .map_err(|_| WatchError::Allocation)?;
    for store in current
        .iter()
        .chain(also)
        .chain(plugins.iter().flat_map(|s| s.iter()))
    {
        if !unique.iter().any(|s| Rc::ptr_eq(s, store)) {
            unique.push(store.clone())
        }
    }
    let mut stops = Vec::new();
    stops
        .try_reserve_exact(unique.len())
        .map_err(|_| WatchError::Allocation)?;
    for store in unique {
        stops.push(
            store
                .subscribe(listener.clone())
                .map_err(WatchError::Callback)?,
        )
    }
    Ok(WatchStop { stops })
}
