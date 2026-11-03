// SPDX-License-Identifier: AGPL-3.0-only
//! remember/recentOutcomes state at caller-owned run and delivered reaction boundaries.
use crate::{
    error::CommandError,
    recent_result::{self, Kept},
    schema_value::{Text, Value},
};
use port_core::js_date::{self, DateError};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
};
/// Source retention count. Native pending-work admission never changes this policy.
pub const RECENT_KEPT: usize = 100;
/// Source outcome state copied into each readout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// The run has no successfully recorded completion reaction.
    Running,
    /// Its success reaction recorded a result or omission.
    Done,
    /// Its failure reaction recorded an error.
    Failed,
}
/// Shared object field storage; array-index ordering is applied when serialized.
pub type Fields = Vec<(Text, Value)>;
/// A result's primitive value is copied; object/array payloads are shared by
/// shallow snapshots. The root kind cannot change through the shared cell.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultValue {
    /// Null/boolean/number/string result, copied when a snapshot is copied.
    Primitive(Value),
    /// Shared array contents, independent of the original command result.
    Array(Rc<RefCell<Vec<Value>>>),
    /// Shared object fields, independent of the original command result.
    Object(Rc<RefCell<Fields>>),
}
/// Plain source error JSON, shared between shallow readout snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorJson {
    /// Source error code spelling.
    pub code: Rc<[u16]>,
    /// Complete UTF-16 message, including unpaired units.
    pub message: Rc<[u16]>,
}
/// Fresh shallow copy. Replacing these fields changes only this snapshot;
/// mutating shared object/array/error contents remains visible in later readouts.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    /// Caller job name; empty strings remain valid here.
    pub job: Rc<[u16]>,
    /// Complete command name.
    pub name: Rc<[u16]>,
    /// State at the time this snapshot was taken.
    pub state: State,
    /// Clipped source UTC ISO start text.
    pub started: Rc<[u16]>,
    /// Raw end-minus-start milliseconds, None while still running.
    pub ms: Option<f64>,
    /// Optional kept value; root undefined leaves it absent.
    pub result: Option<ResultValue>,
    /// Source size-only omission text, when present.
    pub result_omitted: Option<Rc<[u16]>>,
    /// Optional shared plain error JSON.
    pub error: Option<Rc<RefCell<ErrorJson>>>,
}
/// Resource bounds are distinct from source recent retention and source refusals.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Outstanding delivered-reaction capabilities, including evicted records.
    pub pending: usize,
    /// Combined job/name units and each copied error-message bound.
    pub text_units: usize,
    /// Owned retained-data work limits, not the source 16,000-character policy.
    pub data: recent_result::Limits,
}
/// Source/provider errors stay separate from bounded native bookkeeping failures.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// Original Date.now provider exception, before any corresponding assignment.
    Clock(E),
    /// Original normalization/toJSON provider exception, before assignment.
    Normalize(E),
    /// Exact Date clipping/ISO source refusal or allocation failure.
    Date(DateError),
    /// Owned data host failure; never relabelled as original non-JSON.
    Data(recent_result::Error),
    /// Host pending/text bound.
    Limit,
    /// Fallible metadata/text reservation failed.
    Allocation,
    /// A once-only reaction was delivered again.
    Completed,
    /// An undelivered capability was dropped; new remembering is quarantined.
    Abandoned,
}
struct Budget {
    pending: Cell<usize>,
    abandoned: Cell<bool>,
    limits: Limits,
}
struct Inner {
    items: RefCell<VecDeque<Rc<RefCell<Snapshot>>>>,
    budget: Rc<Budget>,
}
/// Session-owned recent state. Clones share this same list, never a business-state copy.
#[derive(Clone)]
pub struct Recent {
    inner: Rc<Inner>,
}
/// Once-only delivered reaction. It retains its own record after eviction but
/// does not keep the entire recent list alive. Executors attach/retain this
/// capability alongside their already-owned run; this module does not poll it.
#[must_use = "attach and retain a once-only completion reaction for the caller-owned run"]
pub struct Ticket {
    outcome: Rc<RefCell<Snapshot>>,
    since: f64,
    budget: Rc<Budget>,
    attempted: bool,
}
fn copy<E>(s: &[u16], max: usize) -> Result<Rc<[u16]>, Error<E>> {
    if s.len() > max {
        return Err(Error::Limit);
    }
    let mut out = Vec::new();
    out.try_reserve_exact(s.len())
        .map_err(|_| Error::Allocation)?;
    out.extend_from_slice(s);
    Ok(Rc::from(out))
}
impl Recent {
    /// Fresh recent list with exactly the original retention policy.
    pub fn new(limits: Limits) -> Self {
        Self {
            inner: Rc::new(Inner {
                items: RefCell::new(VecDeque::new()),
                budget: Rc::new(Budget {
                    pending: Cell::new(0),
                    abandoned: Cell::new(false),
                    limits,
                }),
            }),
        }
    }
    /// Create the record and reaction capability. The exact caller-owned run
    /// value is always returned, including when native/source admission fails;
    /// a failed observer must not cancel or replace the already-started run.
    /// Date sampling/ISO conversion precedes publishing the running record.
    pub fn remember<R, E>(
        &self,
        job: &[u16],
        name: &[u16],
        run: R,
        clock: impl FnOnce() -> Result<f64, E>,
    ) -> (R, Result<Ticket, Error<E>>) {
        let prepare = || {
            let budget = &self.inner.budget;
            if budget.abandoned.get() {
                return Err(Error::Abandoned);
            }
            if budget.pending.get() >= budget.limits.pending
                || job
                    .len()
                    .checked_add(name.len())
                    .is_none_or(|n| n > budget.limits.text_units)
            {
                return Err(Error::Limit);
            }
            let since = clock().map_err(Error::Clock)?;
            let started = js_date::utc_iso_millis(since).map_err(Error::Date)?;
            let job = copy(job, budget.limits.text_units)?;
            let name = copy(name, budget.limits.text_units)?;
            // A clock provider may reenter. Recheck admission without holding a
            // list borrow across it, then publish atomically within this turn.
            if budget.abandoned.get() {
                return Err(Error::Abandoned);
            }
            let pending = budget.pending.get().checked_add(1).ok_or(Error::Limit)?;
            if pending > budget.limits.pending {
                return Err(Error::Limit);
            }
            let mut items = self.inner.items.borrow_mut();
            items.try_reserve(1).map_err(|_| Error::Allocation)?;
            let outcome = Rc::new(RefCell::new(Snapshot {
                job,
                name,
                state: State::Running,
                started: Rc::from(started),
                ms: None,
                result: None,
                result_omitted: None,
                error: None,
            }));
            budget.pending.set(pending);
            items.push_back(outcome.clone());
            if items.len() > RECENT_KEPT {
                items.pop_front();
            }
            Ok(Ticket {
                outcome,
                since,
                budget: budget.clone(),
                attempted: false,
            })
        };
        (run, prepare())
    }
    /// Newest first, only the exact job when Some (including empty). Source
    /// undefined is None. Readout does not reorder or mutate the stored list.
    pub fn outcomes(&self, job: Option<&[u16]>) -> Result<Vec<Snapshot>, Error<()>> {
        let items = self.inner.items.borrow();
        let mut out = Vec::new();
        out.try_reserve_exact(items.len())
            .map_err(|_| Error::Allocation)?;
        out.extend(items.iter().rev().filter_map(|o| {
            let o = o.borrow();
            job.is_none_or(|job| job == o.job.as_ref())
                .then(|| o.clone())
        }));
        Ok(out)
    }
}
impl Ticket {
    fn begin<E>(&mut self) -> Result<(), Error<E>> {
        if self.attempted {
            return Err(Error::Completed);
        }
        self.attempted = true;
        let Some(pending) = self.budget.pending.get().checked_sub(1) else {
            self.budget.abandoned.set(true);
            return Err(Error::Abandoned);
        };
        self.budget.pending.set(pending);
        Ok(())
    }
    /// Fresh shallow view of this record, including after it was evicted.
    pub fn snapshot(&self) -> Snapshot {
        self.outcome.borrow().clone()
    }
    /// Deliver a success reaction. Clock and retained-result preparation finish
    /// before assigning state/ms/result, so provider/host failure leaves running.
    pub fn complete<E>(
        &mut self,
        result: &Value,
        clock: impl FnOnce() -> Result<f64, E>,
    ) -> Result<(), Error<E>> {
        self.begin()?;
        let ms = clock().map_err(Error::Clock)? - self.since;
        let kept = recent_result::keep(result, self.budget.limits.data).map_err(Error::Data)?;
        let (mut value, mut omitted) = (None, None);
        match kept {
            Kept::Absent => {}
            Kept::Omitted { characters } => {
                let message = format!(
                    "{characters} characters, past the {} kept",
                    recent_result::RESULT_CHARS
                );
                omitted = Some(Rc::from(message.encode_utf16().collect::<Vec<_>>()));
            }
            Kept::Result(v) => {
                value = Some(match v {
                    Value::Array(items) => ResultValue::Array(Rc::new(RefCell::new(items))),
                    Value::Object(fields) => ResultValue::Object(Rc::new(RefCell::new(fields))),
                    v => ResultValue::Primitive(v),
                })
            }
        }
        let mut outcome = self.outcome.borrow_mut();
        outcome.state = State::Done;
        outcome.ms = Some(ms);
        outcome.result = value;
        outcome.result_omitted = omitted;
        Ok(())
    }
    /// Deliver a failure reaction. Normalization/toJSON happens after the clock
    /// sample and before any assignment, retaining the original provider fault.
    pub fn fail<E>(
        &mut self,
        clock: impl FnOnce() -> Result<f64, E>,
        normalize: impl FnOnce() -> Result<Arc<CommandError>, E>,
    ) -> Result<(), Error<E>> {
        self.begin()?;
        let ms = clock().map_err(Error::Clock)? - self.since;
        let error = normalize().map_err(Error::Normalize)?;
        let code: Vec<_> = error.code().as_str().encode_utf16().collect();
        let code = Rc::from(code);
        let message = copy(error.message_units(), self.budget.limits.text_units)?;
        let error = Rc::new(RefCell::new(ErrorJson { code, message }));
        let mut outcome = self.outcome.borrow_mut();
        outcome.state = State::Failed;
        outcome.ms = Some(ms);
        outcome.error = Some(error);
        Ok(())
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        if !self.attempted {
            self.budget.abandoned.set(true)
        }
    }
}
