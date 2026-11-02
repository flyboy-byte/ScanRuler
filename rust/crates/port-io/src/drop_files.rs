// SPDX-License-Identifier: AGPL-3.0-only
//! Complete droppedFiles/fileOf/childrenOf algorithm over thin platform callbacks.
//! No recursion, file decoding, engine state or production executor is hidden here.
use std::future::Future;
/// One platform item; non-file items do not invoke an entry getter.
#[derive(Clone, Copy, Debug)]
pub struct DropItem {
    /// Whether its source kind is `file`.
    pub is_file: bool,
    /// Opaque item handle interpreted only by the platform adapter.
    pub handle: usize,
}
/// One entry snapshot. Keep both flags: source file precedence is observable.
#[derive(Clone, Copy, Debug)]
pub struct DropEntry {
    /// Platform entry handle.
    pub handle: usize,
    /// Source entry.isFile.
    pub is_file: bool,
    /// Source entry.isDirectory.
    pub is_directory: bool,
}
/// Snapshot of a transfer's items and browser fallback file list.
pub struct DropData<F> {
    /// Items in source order.
    pub items: Vec<DropItem>,
    /// Unmodified fallback file identities.
    pub files: Vec<F>,
}
/// Thin ordinary platform I/O. Implementations must bound any buffers they create
/// at their own boundary. Errors are opaque and never normalized by traversal.
pub trait DropSource {
    /// Platform-owned file identity (for example scan_files::FileId).
    type File: Copy;
    /// Original platform error, preserved in full.
    type Error;
    /// Optional webkitGetAsEntry; absent method/null result both return None.
    fn entry(&mut self, item: usize) -> Result<Option<DropEntry>, Self::Error>;
    /// Create a fresh reader, including for repeated directory references.
    fn reader(&mut self, entry: DropEntry) -> Result<usize, Self::Error>;
    /// One asynchronous readEntries batch; empty means finished.
    fn batch(&mut self, reader: usize)
    -> impl Future<Output = Result<Vec<DropEntry>, Self::Error>>;
    /// Asynchronous fileOf, preserving the platform error/value.
    fn file(&mut self, entry: DropEntry) -> impl Future<Output = Result<Self::File, Self::Error>>;
}
/// Host resource caps, separate from the original's platform failures.
#[derive(Clone, Copy, Debug)]
pub struct DropLimits {
    /// Maximum input transfer items.
    pub max_items: usize,
    /// Maximum aggregate entries returned by directory batches.
    pub max_entries: usize,
    /// Maximum returned files (including fallback).
    pub max_files: usize,
    /// Maximum readEntries calls, including final empty batches.
    pub max_batches: usize,
    /// Maximum aggregate getter/reader/batch/file calls.
    pub max_calls: usize,
}
/// Distinguishes platform outcomes from host safeguards.
#[derive(Debug, PartialEq, Eq)]
pub enum DropError<E> {
    /// The original opaque platform error, without lossy Display conversion.
    Platform(E),
    /// A host count/work budget was exhausted.
    Limit,
    /// A fallible temporary/output allocation failed.
    Allocation,
}
struct Budget {
    calls: usize,
    batches: usize,
    entries: usize,
    limits: DropLimits,
}
impl Budget {
    fn call<E>(&mut self) -> Result<(), DropError<E>> {
        if self.calls >= self.limits.max_calls {
            return Err(DropError::Limit);
        }
        self.calls += 1;
        Ok(())
    }
}
fn push<F, E>(out: &mut Vec<F>, file: F, max: usize) -> Result<(), DropError<E>> {
    if out.len() >= max {
        return Err(DropError::Limit);
    }
    out.try_reserve(1).map_err(|_| DropError::Allocation)?;
    out.push(file);
    Ok(())
}
/// Complete original traversal over snapshot data and async callbacks. Collect
/// every directory batch before reading its children; nested directories stay
/// ignored. No partial output is returned on a platform or resource failure.
pub async fn dropped_files<S: DropSource>(
    data: Option<&DropData<S::File>>,
    source: &mut S,
    limits: DropLimits,
) -> Result<Vec<S::File>, DropError<S::Error>> {
    let Some(data) = data else {
        return Ok(Vec::new());
    };
    if data.items.len() > limits.max_items {
        return Err(DropError::Limit);
    }
    let mut b = Budget {
        calls: 0,
        batches: 0,
        entries: 0,
        limits,
    };
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(data.items.len())
        .map_err(|_| DropError::Allocation)?;
    for item in &data.items {
        if item.is_file {
            b.call()?;
            entries.push(source.entry(item.handle).map_err(DropError::Platform)?);
        }
    }
    if !entries.iter().flatten().any(|e| e.is_directory) {
        if data.files.len() > limits.max_files {
            return Err(DropError::Limit);
        }
        let mut out = Vec::new();
        out.try_reserve_exact(data.files.len())
            .map_err(|_| DropError::Allocation)?;
        out.extend(data.files.iter().copied());
        return Ok(out);
    }
    let mut out = Vec::new();
    for entry in entries.into_iter().flatten() {
        if entry.is_file {
            b.call()?;
            let file = source.file(entry).await.map_err(DropError::Platform)?;
            push(&mut out, file, limits.max_files)?;
        } else if entry.is_directory {
            b.call()?;
            let reader = source.reader(entry).map_err(DropError::Platform)?;
            let mut children = Vec::new();
            loop {
                if b.batches >= limits.max_batches {
                    return Err(DropError::Limit);
                }
                b.call()?;
                b.batches += 1;
                let batch = source.batch(reader).await.map_err(DropError::Platform)?;
                if batch.is_empty() {
                    break;
                }
                b.entries = b.entries.checked_add(batch.len()).ok_or(DropError::Limit)?;
                if b.entries > limits.max_entries {
                    return Err(DropError::Limit);
                }
                children
                    .try_reserve(batch.len())
                    .map_err(|_| DropError::Allocation)?;
                children.extend(batch);
            }
            for child in children {
                if child.is_file {
                    b.call()?;
                    let file = source.file(child).await.map_err(DropError::Platform)?;
                    push(&mut out, file, limits.max_files)?;
                }
            }
        }
    }
    Ok(out)
}
