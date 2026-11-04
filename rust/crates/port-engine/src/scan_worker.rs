// SPDX-License-Identifier: AGPL-3.0-only
//! Actual owned prepared-model slots and worker/client import commit lifecycle.
//! Parsing/render preparation and RPC/platform failure recovery are separate callers.
use port_core::mesh_graph::MeshGraph;
use std::sync::Arc;
/// Actual resident immutable resource storage supplied by a native geometry service.
pub trait Resident {
    /// Account owned buffers; overflow must return None.
    fn resident_bytes(&self) -> Option<usize>;
}
impl Resident for MeshGraph {
    fn resident_bytes(&self) -> Option<usize> {
        [
            self.positions.capacity(),
            self.indices.capacity(),
            self.normals.capacity(),
            self.adjacency.offsets.capacity(),
            self.adjacency.list.capacity(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, n| sum.checked_add(n.checked_mul(4)?))
    }
}
impl Resident for Vec<u8> {
    fn resident_bytes(&self) -> Option<usize> {
        Some(self.capacity())
    }
}
/// Native storage/work admission policy, not the source split budget.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Prepared handles retained across both model families.
    pub prepared: usize,
    /// Conservative sum across current and prepared resources.
    pub resident_bytes: usize,
    /// Requests within one owner lifetime; keeps source numeric IDs exact.
    pub requests: u64,
    /// IDs inspected by one discard request.
    pub discard_ids: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            prepared: 8,
            resident_bytes: 256 * 1024 * 1024,
            requests: 1_000_000,
            discard_ids: 4096,
        }
    }
}
/// Source client generations, captured by asynchronous result owners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Versions {
    /// Successful worker scan commits, including an explicit clear.
    pub scan: u64,
    /// Successful worker nominal commits, including an explicit clear.
    pub nominal: u64,
}
/// Absence is distinct from clearing an already-empty slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// No field in the source commit request.
    Unchanged,
    /// Explicit null.
    Clear,
    /// Consume this family's prepared handle.
    Prepared(u64),
}
/// Lifecycle failure with source missing-handle text kept distinct from host policy.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Either requested numeric handle is absent in its own family.
    Unavailable,
    /// Native resource or operation budget exceeded.
    HostLimit,
    /// Fallible map storage reservation failed.
    Allocation,
}
impl Error {
    /// Original worker/RPC error text for a missing prepared model.
    pub fn source_message(&self) -> Option<&'static str> {
        match self {
            Self::Unavailable => Some("The prepared import is no longer available."),
            _ => None,
        }
    }
}
/// Worker-handler callback failure is not an ordinary missing-ID reply.
#[derive(Debug, PartialEq, Eq)]
pub enum CommitError<E> {
    /// Admission or missing prepared handle.
    Lifecycle(Error),
    /// Plugin/service failed after the source's already-applied prefix.
    WorkerFailure(E),
}
/// One actual logical-geometry owner; snapshots share its immutable resources.
/// N/S are pure-Rust nominal/query-cache resources supplied by their owning services.
pub struct Worker<N: Resident, S: Resident> {
    graph: Option<Arc<MeshGraph>>,
    nominal: Option<Arc<N>>,
    solid: Option<Arc<S>>,
    scans: Vec<(u64, Arc<MeshGraph>)>,
    nominals: Vec<(u64, Arc<N>)>,
    versions: Versions,
    next_request: u64,
    depth: usize,
    limits: Limits,
}
impl<N: Resident, S: Resident> Worker<N, S> {
    /// Fresh worker and client generations; no duplicate business document.
    pub fn new(limits: Limits) -> Self {
        Self {
            graph: None,
            nominal: None,
            solid: None,
            scans: vec![],
            nominals: vec![],
            versions: Versions::default(),
            next_request: 1,
            depth: 0,
            limits,
        }
    }
    /// Source client reserves before requesting work, including failed requests.
    pub fn reserve_request(&mut self) -> Result<u64, Error> {
        if self.next_request > self.limits.requests || self.next_request >= (1u64 << 53) {
            return Err(Error::HostLimit);
        }
        let id = self.next_request;
        self.next_request += 1;
        Ok(id)
    }
    /// Next client request identity, without consuming it.
    pub fn next_request(&self) -> u64 {
        self.next_request
    }
    /// Actual committed logical graph, independent of render copies.
    pub fn graph(&self) -> Option<Arc<MeshGraph>> {
        self.graph.clone()
    }
    /// Actual nominal resource; no nominal kernel algorithm is claimed here.
    pub fn nominal(&self) -> Option<Arc<N>> {
        self.nominal.clone()
    }
    /// Query cache associated with the current scan.
    pub fn solid(&self) -> Option<Arc<S>> {
        self.solid.clone()
    }
    /// Current client generations.
    pub fn versions(&self) -> Versions {
        self.versions
    }
    /// Test an asynchronous result against the exact model generations it captured.
    pub fn is_current(&self, versions: Versions) -> bool {
        self.versions == versions
    }
    /// Scan-only work is not invalidated by an unrelated nominal replacement.
    pub fn is_scan_current(&self, version: u64) -> bool {
        self.versions.scan == version
    }
    /// Nominal-only work checks its own generation.
    pub fn is_nominal_current(&self, version: u64) -> bool {
        self.versions.nominal == version
    }
    /// Prepared scan resources in source Map insertion order.
    pub fn staged_scans(&self) -> &[(u64, Arc<MeshGraph>)] {
        &self.scans
    }
    /// Prepared nominal resources in source Map insertion order.
    pub fn staged_nominals(&self) -> &[(u64, Arc<N>)] {
        &self.nominals
    }
    fn bytes(&self) -> Option<usize> {
        let mut total = 0usize;
        for bytes in self
            .graph
            .iter()
            .map(|v| v.resident_bytes())
            .chain(self.nominal.iter().map(|v| v.resident_bytes()))
            .chain(self.solid.iter().map(|v| v.resident_bytes()))
            .chain(self.scans.iter().map(|(_, v)| v.resident_bytes()))
            .chain(self.nominals.iter().map(|(_, v)| v.resident_bytes()))
        {
            total = total.checked_add(bytes?)?;
        }
        Some(total)
    }
    fn admit(
        &self,
        bytes: Option<usize>,
        replaced: Option<usize>,
        added: bool,
    ) -> Result<(), Error> {
        let slots = self
            .scans
            .len()
            .checked_add(self.nominals.len())
            .ok_or(Error::HostLimit)?;
        if added && slots >= self.limits.prepared {
            return Err(Error::HostLimit);
        }
        let total = self
            .bytes()
            .and_then(|n| n.checked_sub(replaced.unwrap_or(0)))
            .and_then(|n| n.checked_add(bytes?))
            .ok_or(Error::HostLimit)?;
        if total > self.limits.resident_bytes {
            return Err(Error::HostLimit);
        }
        Ok(())
    }
    /// Retain a completely prepared logical scan; no parser/render continuation is faked.
    pub fn stage_scan(&mut self, id: u64, graph: Arc<MeshGraph>) -> Result<(), Error> {
        let existing = self.scans.iter().position(|(key, _)| *key == id);
        self.admit(
            graph.resident_bytes(),
            existing.and_then(|i| self.scans[i].1.resident_bytes()),
            existing.is_none(),
        )?;
        if let Some(i) = existing {
            self.scans[i].1 = graph;
        } else {
            self.scans.try_reserve(1).map_err(|_| Error::Allocation)?;
            self.scans.push((id, graph));
        }
        Ok(())
    }
    /// Retain a completely prepared nominal resource from its native service.
    pub fn stage_nominal(&mut self, id: u64, nominal: Arc<N>) -> Result<(), Error> {
        let existing = self.nominals.iter().position(|(key, _)| *key == id);
        self.admit(
            nominal.resident_bytes(),
            existing.and_then(|i| self.nominals[i].1.resident_bytes()),
            existing.is_none(),
        )?;
        if let Some(i) = existing {
            self.nominals[i].1 = nominal;
        } else {
            self.nominals
                .try_reserve(1)
                .map_err(|_| Error::Allocation)?;
            self.nominals.push((id, nominal));
        }
        Ok(())
    }
    /// Source plugin-context take: remove before returning the actual immutable graph.
    pub fn take_staged(&mut self, id: u64) -> Option<Arc<MeshGraph>> {
        self.scans
            .iter()
            .position(|(key, _)| *key == id)
            .map(|i| self.scans.remove(i).1)
    }
    /// Discard both families; unknown or repeated identities are harmless.
    pub fn discard(&mut self, ids: &[u64]) -> Result<(), Error> {
        if ids.len() > self.limits.discard_ids {
            return Err(Error::HostLimit);
        }
        for id in ids {
            self.scans.retain(|(key, _)| key != id);
            self.nominals.retain(|(key, _)| key != id);
        }
        Ok(())
    }
    /// Retain an actual native query cache; scan commit releases it before callbacks.
    pub fn set_solid(&mut self, solid: Option<Arc<S>>) -> Result<(), Error> {
        self.admit(
            solid.as_ref().map_or(Some(0), |v| v.resident_bytes()),
            self.solid.as_ref().and_then(|v| v.resident_bytes()),
            false,
        )?;
        self.solid = solid;
        Ok(())
    }
    /// Complete worker commit prefix and successful client generation update.
    /// Both handles are resolved before mutation. Callback failure keeps the applied
    /// scan prefix, leaves nominal/generations alone, and must be routed as worker
    /// failure by the platform/RPC owner (not silently converted into success).
    pub fn commit<E>(
        &mut self,
        scan: Change,
        nominal: Change,
        mut scan_replaced: impl FnMut(&mut Self) -> Result<(), E>,
    ) -> Result<(), CommitError<E>> {
        let next_scan = match scan {
            Change::Prepared(id) => Some(
                self.scans
                    .iter()
                    .find(|(key, _)| *key == id)
                    .ok_or(CommitError::Lifecycle(Error::Unavailable))?
                    .1
                    .clone(),
            ),
            _ => None,
        };
        let next_nominal = match nominal {
            Change::Prepared(id) => Some(
                self.nominals
                    .iter()
                    .find(|(key, _)| *key == id)
                    .ok_or(CommitError::Lifecycle(Error::Unavailable))?
                    .1
                    .clone(),
            ),
            _ => None,
        };
        if self.depth >= 32
            || self.versions.scan >= self.limits.requests
            || self.versions.nominal >= self.limits.requests
        {
            return Err(CommitError::Lifecycle(Error::HostLimit));
        }
        self.depth += 1;
        if scan != Change::Unchanged {
            self.graph = next_scan;
            self.solid = None;
            if let Change::Prepared(id) = scan {
                self.scans.retain(|(key, _)| *key != id);
            }
            if let Err(error) = scan_replaced(self) {
                self.depth -= 1;
                return Err(CommitError::WorkerFailure(error));
            }
        }
        if nominal != Change::Unchanged {
            self.nominal = next_nominal;
            if let Change::Prepared(id) = nominal {
                self.nominals.retain(|(key, _)| *key != id);
            }
        }
        // These advance only after a successful worker answer, as commitImport does.
        if scan != Change::Unchanged {
            self.versions.scan += 1;
        }
        if nominal != Change::Unchanged {
            self.versions.nominal += 1;
        }
        self.depth -= 1;
        Ok(())
    }
}
