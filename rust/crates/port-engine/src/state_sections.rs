// SPDX-License-Identifier: AGPL-3.0-only
//! Plugin-owned readout sections, live registration and plain-object assignment.
use crate::{
    history_record::Cell as Value,
    schema_value::{Value as Schema, order},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};
/// Caught source values expose message/String conversion only when readAll
/// reaches its catch. Conversion can throw without re-entering that same catch.
pub enum Fault<'a, E> {
    /// Actual Error: read message as-is, without String coercion.
    Error(Box<dyn FnOnce() -> Result<Value, E> + 'a>),
    /// Other thrown value: source String conversion, preserving UTF16.
    Other(Box<dyn FnOnce() -> Result<Vec<u16>, E> + 'a>),
}
/// Function reference identity is the Rc identity; readers may reenter/register.
pub type Reader<'a, E> = Rc<dyn Fn(&Sections<'a, E>) -> Result<Value, Fault<'a, E>> + 'a>;
/// Explicit host bounds, independent of source duplicate/reader faults.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Registration slots including tombstones during reads.
    pub slots: usize,
    /// Callback visits across nested reads.
    pub visits: usize,
    /// Nested read depth.
    pub depth: usize,
    /// ID length in UTF16 units.
    pub key_units: usize,
}
/// Source duplicate error text or exact conversion fault; host errors stay apart.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// Exact ordinary Error.message text for a duplicate section ID.
    Duplicate(Vec<u16>),
    /// Original lazy message/String provider fault, unmodified.
    Conversion(E),
    /// Slot/visit/depth/ID host admission failure.
    Limit,
    /// Fallible metadata allocation failure.
    Allocation,
}
/// Result prototype after assigning __proto__ on a fresh ordinary object.
#[derive(Debug, Clone)]
pub enum Prototype {
    /// Unchanged Object.prototype.
    Default,
    /// Explicit source null prototype.
    Null,
    /// Shared owned object/array/typed-array returned by a reader.
    Data(Value),
}
/// Own properties plus separate prototype, never an own __proto__ map entry.
#[derive(Debug, Clone)]
pub struct Readout {
    fields: Vec<(Vec<u16>, Value)>,
    /// Prototype can affect inherited lookup but never Object.keys ordering.
    pub prototype: Prototype,
}
impl Readout {
    /// Own data entries in source Object.keys order, indices before string keys.
    pub fn own(&self) -> Vec<(&[u16], &Value)> {
        let keys = self
            .fields
            .iter()
            .map(|(k, _)| (k.clone(), Schema::Undefined))
            .collect::<Vec<_>>();
        order(&keys)
            .into_iter()
            .map(|i| (self.fields[i].0.as_slice(), &self.fields[i].1))
            .collect()
    }
    fn assign<E>(&mut self, key: &[u16], value: Value) -> Result<(), Error<E>> {
        if key.iter().copied().eq("__proto__".encode_utf16()) {
            match value {
                Value::Null => self.prototype = Prototype::Null,
                Value::Object(_) | Value::Array(_) | Value::Bytes(_) | Value::Words(_) => {
                    self.prototype = Prototype::Data(value)
                }
                _ => {}
            }
            return Ok(());
        }
        if let Some((_, old)) = self.fields.iter_mut().find(|(k, _)| k == key) {
            *old = value;
            return Ok(());
        }
        self.fields.try_reserve(1).map_err(|_| Error::Allocation)?;
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(key.len())
            .map_err(|_| Error::Allocation)?;
        owned.extend_from_slice(key);
        self.fields.push((owned, value));
        Ok(())
    }
}
struct Entry<'a, E> {
    key: Rc<[u16]>,
    describe: Reader<'a, E>,
    brief: Option<Reader<'a, E>>,
}
struct Inner<'a, E> {
    entries: RefCell<Vec<Option<Entry<'a, E>>>>,
    limits: Limits,
    depth: Cell<usize>,
    visits: Cell<usize>,
}
/// One session's live section and brief maps. Clones share the same registrations.
pub struct Sections<'a, E> {
    inner: Rc<Inner<'a, E>>,
}
impl<'a, E> Clone for Sections<'a, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Explicit release. Old releases only delete a matching describe function,
/// including a later same-function registration; Drop does not release.
pub struct Release<'a, E> {
    inner: Weak<Inner<'a, E>>,
    key: Rc<[u16]>,
    describe: Reader<'a, E>,
}
impl<E> Release<'_, E> {
    /// Remove both section and brief only if the exact describe still owns ID.
    pub fn release(&self) {
        if let Some(inner) = self.inner.upgrade() {
            for slot in inner.entries.borrow_mut().iter_mut() {
                if slot.as_ref().is_some_and(|entry| {
                    entry.key == self.key && Rc::ptr_eq(&entry.describe, &self.describe)
                }) {
                    *slot = None;
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
impl<'a, E> Sections<'a, E> {
    /// Empty live readout maps, unrelated to the business document's ownership.
    pub fn new(limits: Limits) -> Self {
        Self {
            inner: Rc::new(Inner {
                entries: RefCell::new(vec![]),
                limits,
                depth: Cell::new(0),
                visits: Cell::new(0),
            }),
        }
    }
    /// Reject duplicates before installing either describe or brief.
    pub fn register(
        &self,
        key: Rc<[u16]>,
        describe: Reader<'a, E>,
        brief: Option<Reader<'a, E>>,
    ) -> Result<Release<'a, E>, Error<E>> {
        let mut entries = self.inner.entries.borrow_mut();
        if entries.iter().flatten().any(|e| e.key == key) {
            let mut message = "Two state sections share the id \""
                .encode_utf16()
                .collect::<Vec<_>>();
            message.extend(key.iter());
            message.extend("\".".encode_utf16());
            return Err(Error::Duplicate(message));
        }
        if key.len() > self.inner.limits.key_units {
            return Err(Error::Limit);
        }
        if self.inner.depth.get() == 0 {
            entries.retain(Option::is_some)
        }
        if entries.len() >= self.inner.limits.slots {
            return Err(Error::Limit);
        }
        entries.try_reserve(1).map_err(|_| Error::Allocation)?;
        entries.push(Some(Entry {
            key: key.clone(),
            describe: describe.clone(),
            brief,
        }));
        Ok(Release {
            inner: Rc::downgrade(&self.inner),
            key,
            describe,
        })
    }
    /// Every section, read now, without copying returned shared composite data.
    pub fn read(&self) -> Result<Readout, Error<E>> {
        self.read_all(false)
    }
    /// Only registered briefs, in live brief-map insertion order.
    pub fn briefs(&self) -> Result<Readout, Error<E>> {
        self.read_all(true)
    }
    fn read_all(&self, brief: bool) -> Result<Readout, Error<E>> {
        if self.inner.depth.get() >= self.inner.limits.depth {
            return Err(Error::Limit);
        }
        if self.inner.depth.get() == 0 {
            self.inner.visits.set(0)
        }
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = Depth(&self.inner.depth);
        let mut out = Readout {
            fields: vec![],
            prototype: Prototype::Default,
        };
        let mut cursor = 0;
        loop {
            let next = {
                let entries = self.inner.entries.borrow();
                let Some(slot) = entries.get(cursor) else {
                    break;
                };
                slot.as_ref().and_then(|e| {
                    let reader = if brief {
                        e.brief.clone()
                    } else {
                        Some(e.describe.clone())
                    };
                    reader.map(|reader| (e.key.clone(), reader))
                })
            };
            cursor += 1;
            let Some((key, reader)) = next else { continue };
            if self.inner.visits.get() >= self.inner.limits.visits {
                return Err(Error::Limit);
            }
            self.inner.visits.set(self.inner.visits.get() + 1);
            let value = match reader(self) {
                Ok(value) => value,
                Err(fault) => {
                    let message = match fault {
                        Fault::Error(get) => get().map_err(Error::Conversion)?,
                        Fault::Other(string) => Value::String(string().map_err(Error::Conversion)?),
                    };
                    Value::Object(Arc::new(vec![("error".encode_utf16().collect(), message)]))
                }
            };
            out.assign(&key, value)?;
        }
        Ok(out)
    }
}
