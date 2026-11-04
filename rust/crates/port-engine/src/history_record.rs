// SPDX-License-Identifier: AGPL-3.0-only
//! Identity-aware history recording from pinned historyStore.ts, over stable
//! typed section snapshots. A session owns the document; this recorder holds
//! shared immutable patch references, never another mutable business-state store.
//! The restore child supplies global orchestration; concrete family capture/apply
//! and scheduling remain separate continuations.
use crate::schema_value::{Text, Value, order};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// Source limit, independent of host metadata budgets.
pub const HISTORY_LIMIT: usize = 64;
const SAFE: u64 = 9_007_199_254_740_991;
/// Owned immutable JavaScript data with composite reference identity. Composite
/// payloads are shared rather than copied. Objects have unique own field keys;
/// no getters, custom prototypes, functions, cycles or array holes are represented.
#[derive(Debug, Clone)]
pub enum Cell {
    /// Undefined.
    Undefined,
    /// Null.
    Null,
    /// Boolean.
    Boolean(bool),
    /// IEEE number, including nonfinite and signed zero.
    Number(f64),
    /// Lossless UTF16 primitive string.
    String(Text),
    /// Ordinary array (element comparison is distinct from its identity).
    Array(Arc<Vec<Cell>>),
    /// Ordinary object fields, insertion order.
    Object(Arc<Vec<(Text, Cell)>>),
    /// Uint8Array; strict reference comparison, not ordinary-array comparison.
    Bytes(Arc<Vec<u8>>),
    /// Uint32Array, including marked scopes; never copied by snapshots.
    Words(Arc<Vec<u32>>),
}
impl Cell {
    /// JavaScript strict equality for the represented data. NaN differs from
    /// itself; signed zero compares equal. Composite values compare by identity.
    pub fn strict_equal(&self, b: &Self) -> bool {
        match (self, b) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => Arc::ptr_eq(a, b),
            (Self::Object(a), Self::Object(b)) => Arc::ptr_eq(a, b),
            (Self::Bytes(a), Self::Bytes(b)) => Arc::ptr_eq(a, b),
            (Self::Words(a), Self::Words(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
fn same_value(a: &Cell, b: &Cell) -> bool {
    a.strict_equal(b)
        || match (a, b) {
            (Cell::Array(a), Cell::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a.strict_equal(b))
            }
            _ => false,
        }
}
fn builtin(key: &[u16]) -> bool {
    [
        "constructor",
        "__defineGetter__",
        "__defineSetter__",
        "hasOwnProperty",
        "__lookupGetter__",
        "__lookupSetter__",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toString",
        "valueOf",
        "__proto__",
        "toLocaleString",
    ]
    .iter()
    .any(|s| key.iter().copied().eq(s.encode_utf16()))
}
fn fields_equal<E>(a: &Cell, b: &Cell) -> Result<bool, HistoryError<E>> {
    let (Cell::Object(a), Cell::Object(b)) = (a, b) else {
        return Err(HistoryError::InvalidSnapshot);
    };
    let lookup: BTreeMap<_, _> = b.iter().map(|(k, v)| (k.as_slice(), v)).collect();
    // No root-reference shortcut: an unchanged object with a NaN field still
    // differs according to source same(). Added after-only fields are ignored.
    let keys: Vec<_> = a
        .iter()
        .map(|(k, _)| (k.clone(), Value::Undefined))
        .collect();
    Ok(order(&keys).into_iter().all(|i| {
        let (key, a) = &a[i];
        match lookup.get(key.as_slice()) {
            Some(b) => same_value(a, b),
            None if builtin(key) => false,
            None => same_value(a, &Cell::Undefined),
        }
    }))
}
/// A source participant's section-comparison mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    /// Walk the before object's fields; arrays compare their elements strictly.
    Fields,
    /// Strict comparison of the captured value itself.
    Identity,
}
/// Captured owned section value and its currently registered policy.
#[derive(Debug, Clone)]
pub struct Section {
    /// Capture key; unique and in original Object.keys order.
    pub key: Text,
    /// Shared immutable captured data.
    pub value: Cell,
    /// Current participant policy; scope/scan always use Identity.
    pub compare: Compare,
}
/// A stable section-key set captured from the session's actual document.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Original own-key order (array-index keys first); no __proto__ setter key.
    pub sections: Vec<Section>,
}
/// Only changed section keys; entries preserve original spread/key order.
pub type Patch = Vec<(Text, Cell)>;
/// One original history entry.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Earliest captured values for changed sections.
    pub before: Patch,
    /// Latest captured values for changed sections.
    pub after: Patch,
    /// First step's label is kept when groups merge.
    pub label: Text,
    /// Workspace at wrapper entry.
    pub workspace: Text,
    /// Zero when outside an explicit group.
    pub group: u64,
}
/// Owned bookkeeping used by this recorder and restore implementation.
/// Patch references share document assets. No current mutable document is here.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Undo entries, oldest first.
    pub past: Vec<Entry>,
    /// Redo entries, oldest first; restore owns their movement.
    pub future: Vec<Entry>,
    /// Restore in progress (wrapper suppression only here).
    pub restoring: bool,
    /// Original flag retained but not invented as a recording guard.
    pub importing: bool,
    /// Changes on clear/travel, not record.
    pub revision: u64,
}
/// Host budgets for recording metadata and examined cells. Large immutable
/// payload references are not copied/traversed: their session asset owner must
/// account for their lifetime/memory as part of the broader P05 contract.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Stable registered section count.
    pub sections: usize,
    /// Fields examined per default-comparison object.
    pub fields: usize,
    /// Examined scalar/reference cells across one capture.
    pub cells: usize,
    /// Examined UTF16 metadata/string units.
    pub units: usize,
    /// Open recording wrappers.
    pub pending: usize,
    /// Label/workspace UTF16 length each.
    pub text: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            sections: 64,
            fields: 2048,
            cells: 200_000,
            units: 2_000_000,
            pending: 128,
            text: 4096,
        }
    }
}
/// Synchronous wrapper guards; async intentionally ignores busy/hook_blocked.
#[derive(Debug, Clone, Copy, Default)]
pub struct Guards {
    /// Original useStore.busy.
    pub busy: bool,
    /// Original configureHistory hooks.blocked().
    pub hook_blocked: bool,
}
/// Host failure or an unchanged caller-owned capture fault. CommandError codes
/// and other error payloads need not be reduced to Display/message strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError<E = Text> {
    /// Explicit metadata/count/counter budget exceeded.
    Limit,
    /// Fallible reservation failed.
    Allocation,
    /// Not a stable typed snapshot or valid source-shaped bookkeeping.
    InvalidSnapshot,
    /// Repeated completion or a token from another recorder.
    InvalidToken,
    /// The actual section capture failed; preserve its opaque caller-owned error.
    Capture(E),
}
impl<E> HistoryError<E> {
    /// Map only the opaque capture fault; host failures keep their identity.
    pub fn map_capture<F>(self, map: impl FnOnce(E) -> F) -> HistoryError<F> {
        match self {
            Self::Limit => HistoryError::Limit,
            Self::Allocation => HistoryError::Allocation,
            Self::InvalidSnapshot => HistoryError::InvalidSnapshot,
            Self::InvalidToken => HistoryError::InvalidToken,
            Self::Capture(e) => HistoryError::Capture(map(e)),
        }
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Sync,
    Async,
}
struct Pending {
    id: u64,
    kind: Kind,
    before: Snapshot,
    label: Text,
    workspace: Text,
    generation: u64,
}
/// Completion capability bound to the recorder that opened it.
#[derive(Debug, Clone)]
pub struct Token {
    domain: Arc<()>,
    id: u64,
}
// Owned capture continuation: counters have already dropped; no mutable recorder borrow
// spans the application's after-capture callback.
pub(crate) struct CaptureFinish(Pending);

/// Dependency-free event-driven wrapper/recording state. The host always reports
/// completion, including body failure or timeout; no automatic rollback is added.
pub struct Recorder {
    /// Source-shaped history bookkeeping, also used by the future travel layer.
    pub state: State,
    limits: Limits,
    domain: Arc<()>,
    pending: Vec<Pending>,
    next: u64,
    depth: usize,
    paused: usize,
    group: u64,
    next_group: u64,
    epoch: u64,
}
impl Default for Recorder {
    fn default() -> Self {
        Self::new(Limits::default())
    }
}
fn is(key: &[u16], s: &str) -> bool {
    key.iter().copied().eq(s.encode_utf16())
}
fn identity(s: &Section) -> bool {
    s.compare == Compare::Identity || is(&s.key, "scope") || is(&s.key, "scan")
}
fn spend<E>(left: &mut usize, n: usize) -> Result<(), HistoryError<E>> {
    *left = left.checked_sub(n).ok_or(HistoryError::Limit)?;
    Ok(())
}
fn scalar_budget<E>(c: &Cell, cells: &mut usize, units: &mut usize) -> Result<(), HistoryError<E>> {
    spend(cells, 1)?;
    if let Cell::String(s) = c {
        spend(units, s.len())?;
    }
    Ok(())
}
fn snapshot_check<E>(s: &Snapshot, l: Limits) -> Result<(), HistoryError<E>> {
    if s.sections.len() > l.sections {
        return Err(HistoryError::Limit);
    }
    let (mut cells, mut units) = (l.cells, l.units);
    let mut keys = BTreeSet::new();
    for section in &s.sections {
        spend(&mut units, section.key.len())?;
        if is(&section.key, "__proto__") || !keys.insert(&section.key) {
            return Err(HistoryError::InvalidSnapshot);
        }
        scalar_budget(&section.value, &mut cells, &mut units)?;
        if identity(section) {
            continue;
        }
        let Cell::Object(fields) = &section.value else {
            return Err(HistoryError::InvalidSnapshot);
        };
        if fields.len() > l.fields {
            return Err(HistoryError::Limit);
        }
        let mut keys = BTreeSet::new();
        for (k, v) in fields.iter() {
            spend(&mut units, k.len())?;
            if !keys.insert(k) {
                return Err(HistoryError::InvalidSnapshot);
            }
            scalar_budget(v, &mut cells, &mut units)?;
            if let Cell::Array(a) = v {
                for item in a.iter() {
                    scalar_budget(item, &mut cells, &mut units)?;
                }
            }
        }
    }
    Ok(())
}
fn spread<E>(first: &Patch, second: &Patch) -> Result<Patch, HistoryError<E>> {
    let mut out = Vec::new();
    out.try_reserve(first.len().saturating_add(second.len()))
        .map_err(|_| HistoryError::Allocation)?;
    out.extend(first.iter().cloned());
    for (k, v) in second {
        if let Some((_, old)) = out.iter_mut().find(|(key, _)| key == k) {
            *old = v.clone()
        } else {
            out.push((k.clone(), v.clone()))
        }
    }
    Ok(out)
}
impl Recorder {
    /// Empty recorder, original zero counters and no captured document.
    pub fn new(limits: Limits) -> Self {
        Self {
            state: State::default(),
            limits,
            domain: Arc::new(()),
            pending: Vec::new(),
            next: 0,
            depth: 0,
            paused: 0,
            group: 0,
            next_group: 0,
            epoch: 0,
        }
    }
    /// Source history epoch; no additional worker/session epoch is invented.
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Whether an async history wrapper is open (not a full historyBlocked()).
    pub const fn paused(&self) -> bool {
        self.paused > 0
    }
    /// Begin/reset explicit group. Nested begins select a new group, not a stack.
    pub fn begin_group(&mut self) -> Result<(), HistoryError> {
        let next = self
            .next_group
            .checked_add(1)
            .filter(|n| *n <= SAFE)
            .ok_or(HistoryError::Limit)?;
        self.next_group = next;
        self.group = next;
        Ok(())
    }
    /// Leave the current group.
    pub fn end_group(&mut self) {
        self.group = 0;
    }
    /// Clear history and increment epoch/revision, leaving open wrapper counters
    /// and nextGroup intact so late completions are discarded rather than reset.
    pub fn clear(&mut self) -> Result<(), HistoryError> {
        let epoch = self
            .epoch
            .checked_add(1)
            .filter(|n| *n <= SAFE)
            .ok_or(HistoryError::Limit)?;
        let revision = self
            .state
            .revision
            .checked_add(1)
            .filter(|n| *n <= SAFE)
            .ok_or(HistoryError::Limit)?;
        self.epoch = epoch;
        self.state.revision = revision;
        self.group = 0;
        self.state.past.clear();
        self.state.future.clear();
        Ok(())
    }
    /// Open a sync wrapper. A skipped wrapper still runs its body, but has no
    /// token and must not capture or record a step.
    pub fn begin_sync<E>(
        &mut self,
        label: &[u16],
        workspace: &[u16],
        guards: Guards,
        capture: impl FnOnce(&Self) -> Result<Snapshot, E>,
    ) -> Result<Option<Token>, HistoryError<E>> {
        if self.depth > 0
            || self.paused > 0
            || self.state.restoring
            || guards.busy
            || guards.hook_blocked
        {
            return Ok(None);
        }
        self.begin(Kind::Sync, label, workspace, capture)
    }
    /// Open an async wrapper: source checks only paused/restoring, ignoring sync
    /// depth, store busy, participant blocking and the hook guard.
    pub fn begin_async<E>(
        &mut self,
        label: &[u16],
        workspace: &[u16],
        capture: impl FnOnce(&Self) -> Result<Snapshot, E>,
    ) -> Result<Option<Token>, HistoryError<E>> {
        if self.paused > 0 || self.state.restoring {
            return Ok(None);
        }
        self.begin(Kind::Async, label, workspace, capture)
    }
    // Runtime integration samples paused/restoring before calling the document's capture
    // and workspace providers. Commit that admitted capture without rechecking the source
    // guard after potentially reentrant providers; no RefMut spans a provider callback.
    pub(crate) fn begin_captured_async<E>(
        &mut self,
        label: &[u16],
        workspace: &[u16],
        snapshot: Snapshot,
    ) -> Result<Option<Token>, HistoryError<E>> {
        self.begin(Kind::Async, label, workspace, |_| Ok(snapshot))
    }
    fn begin<E>(
        &mut self,
        kind: Kind,
        label: &[u16],
        workspace: &[u16],
        capture: impl FnOnce(&Self) -> Result<Snapshot, E>,
    ) -> Result<Option<Token>, HistoryError<E>> {
        if self.pending.len() >= self.limits.pending
            || label.len() > self.limits.text
            || workspace.len() > self.limits.text
        {
            return Err(HistoryError::Limit);
        }
        let id = self.next.checked_add(1).ok_or(HistoryError::Limit)?;
        self.pending
            .try_reserve(1)
            .map_err(|_| HistoryError::Allocation)?;
        let before = capture(self).map_err(HistoryError::Capture)?;
        snapshot_check(&before, self.limits)?;
        self.next = id;
        self.pending.push(Pending {
            id,
            kind,
            before,
            label: label.to_vec(),
            workspace: workspace.to_vec(),
            generation: self.epoch,
        });
        match kind {
            Kind::Sync => self.depth += 1,
            Kind::Async => self.paused += 1,
        };
        Ok(Some(Token {
            domain: self.domain.clone(),
            id,
        }))
    }
    /// Finish once even if the body failed. Counters drop BEFORE after-capture,
    /// just like the original finally block; capture failure leaves body changes
    /// in the document and creates no step. A stale epoch drops the record.
    pub fn finish<E>(
        &mut self,
        token: Token,
        capture: impl FnOnce(&Self) -> Result<Snapshot, E>,
    ) -> Result<(), HistoryError<E>> {
        let finishing = self.finish_begin(token)?;
        let after = capture(self).map_err(HistoryError::Capture)?;
        self.finish_captured(finishing, after)
    }
    pub(crate) fn finish_begin<E>(
        &mut self,
        token: Token,
    ) -> Result<CaptureFinish, HistoryError<E>> {
        if !Arc::ptr_eq(&token.domain, &self.domain) {
            return Err(HistoryError::InvalidToken);
        }
        let index = self
            .pending
            .iter()
            .position(|p| p.id == token.id)
            .ok_or(HistoryError::InvalidToken)?;
        let pending = self.pending.remove(index);
        match pending.kind {
            Kind::Sync => self.depth -= 1,
            Kind::Async => self.paused -= 1,
        };
        Ok(CaptureFinish(pending))
    }
    pub(crate) fn finish_captured<E>(
        &mut self,
        finishing: CaptureFinish,
        after: Snapshot,
    ) -> Result<(), HistoryError<E>> {
        self.record(finishing.0, after)
    }
    fn record<E>(&mut self, p: Pending, after: Snapshot) -> Result<(), HistoryError<E>> {
        if p.generation != self.epoch {
            return Ok(());
        }
        snapshot_check(&after, self.limits)?;
        if self.state.past.len() > HISTORY_LIMIT || self.state.future.len() > HISTORY_LIMIT {
            return Err(HistoryError::InvalidSnapshot);
        }
        let mut before_patch = Vec::new();
        let mut after_patch = Vec::new();
        before_patch
            .try_reserve(p.before.sections.len())
            .map_err(|_| HistoryError::Allocation)?;
        after_patch
            .try_reserve(p.before.sections.len())
            .map_err(|_| HistoryError::Allocation)?;
        for before in p.before.sections {
            let next = after
                .sections
                .iter()
                .find(|s| s.key == before.key)
                .ok_or(HistoryError::InvalidSnapshot)?;
            // Source compares by the current participant policy, not entry policy.
            let same = if identity(next) {
                before.value.strict_equal(&next.value)
            } else {
                fields_equal(&before.value, &next.value)?
            };
            if !same {
                before_patch.push((before.key.clone(), before.value));
                after_patch.push((before.key, next.value.clone()));
            }
        }
        if before_patch.is_empty() {
            return Ok(());
        }
        if self.group != 0
            && self.state.future.is_empty()
            && let Some(last) = self
                .state
                .past
                .last()
                .filter(|e| e.group == self.group && e.workspace == p.workspace)
        {
            let before = spread(&before_patch, &last.before)?;
            let after = spread(&last.after, &after_patch)?;
            let mut merged = last.clone();
            merged.before = before;
            merged.after = after;
            if let Some(last) = self.state.past.last_mut() {
                *last = merged;
            }
            return Ok(());
        }
        self.state
            .past
            .try_reserve(1)
            .map_err(|_| HistoryError::Allocation)?;
        if self.state.past.len() >= HISTORY_LIMIT {
            self.state
                .past
                .drain(..self.state.past.len() - (HISTORY_LIMIT - 1));
        }
        self.state.past.push(Entry {
            before: before_patch,
            after: after_patch,
            label: p.label,
            workspace: p.workspace,
            group: self.group,
        });
        self.state.future.clear();
        Ok(())
    }
}

/// Global restoration orchestration over owned typed capture/apply callbacks.
pub mod restore;

/// Concrete core capture/patch rules over borrowed owned data.
pub mod core;
