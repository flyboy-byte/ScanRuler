// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen geometry/unionFind.ts: path halving and source-directed roots.

/// Invalid input or bounded storage failure.
#[derive(Debug, PartialEq, Eq)]
pub enum ComponentError {
    /// Index outside the parent array, or malformed triangle indices.
    InvalidIndex,
    /// Standalone parent links are cyclic or out of range.
    InvalidParent,
    /// Caller budget or original signed parent capacity exceeded.
    VertexLimit,
    /// Parent storage reservation failed.
    Allocation,
}
fn halve(parent: &mut [u32], mut i: usize) -> u32 {
    while parent[i] as usize != i {
        parent[i] = parent[parent[i] as usize];
        i = parent[i] as usize;
    }
    i as u32
}

/// Source findRoot on a caller-owned valid integer parent forest.
/// The queried chain is checked before mutation; unrelated links are untouched.
/// Invalid/cyclic chains fail safely, instead of looping or indexing out of range.
pub fn find_root(parent: &mut [u32], i: u32) -> Result<u32, ComponentError> {
    if i as usize >= parent.len() {
        return Err(ComponentError::InvalidIndex);
    }
    let mut at = i as usize;
    let mut rooted = false;
    for _ in 0..parent.len() {
        let next = parent[at] as usize;
        if next >= parent.len() {
            return Err(ComponentError::InvalidParent);
        }
        if next == at {
            rooted = true;
            break;
        }
        at = next;
    }
    if !rooted {
        return Err(ComponentError::InvalidParent);
    }
    Ok(halve(parent, i as usize))
}

/// The original directed disjoint sets, with internally maintained forest validity.
#[derive(Debug, PartialEq, Eq)]
pub struct UnionFind {
    parent: Vec<u32>,
}
impl UnionFind {
    /// Read-only parent state, including observable path-halving mutations.
    pub fn parent(&self) -> &[u32] {
        &self.parent
    }
    /// Find a member's root, halving its path in the original order.
    pub fn find(&mut self, i: u32) -> Result<u32, ComponentError> {
        if i as usize >= self.parent.len() {
            return Err(ComponentError::InvalidIndex);
        }
        Ok(halve(&mut self.parent, i as usize))
    }
    /// Join a's root under b's root; both finds still run for same-set joins.
    /// Both indices are checked before any mutation.
    pub fn union(&mut self, a: u32, b: u32) -> Result<(), ComponentError> {
        if a as usize >= self.parent.len() || b as usize >= self.parent.len() {
            return Err(ComponentError::InvalidIndex);
        }
        let ra = halve(&mut self.parent, a as usize);
        let rb = halve(&mut self.parent, b as usize);
        if ra != rb {
            self.parent[ra as usize] = rb;
        }
        Ok(())
    }
}

/// Make members 0..n, each initially its own root.
/// A conservative i32::MAX vertex cap keeps every parent representable in the
/// original signed Int32Array; caller budgets can further limit storage.
pub fn make_union_find(n: usize, max_vertices: usize) -> Result<UnionFind, ComponentError> {
    if n > max_vertices || n > i32::MAX as usize {
        return Err(ComponentError::VertexLimit);
    }
    let mut parent = Vec::new();
    parent
        .try_reserve_exact(n)
        .map_err(|_| ComponentError::Allocation)?;
    parent.extend((0..n).map(|i| i as u32));
    Ok(UnionFind { parent })
}

/// Connected vertices of triangles: the second/third roots go under the first.
/// Isolated vertices remain independent; triangle order and representative IDs
/// are preserved. Indices are validated before allocation or union mutations.
pub fn vertex_components(
    indices: &[u32],
    vertex_count: usize,
    max_vertices: usize,
) -> Result<UnionFind, ComponentError> {
    if !indices.len().is_multiple_of(3) || indices.iter().any(|&i| i as usize >= vertex_count) {
        return Err(ComponentError::InvalidIndex);
    }
    let mut sets = make_union_find(vertex_count, max_vertices)?;
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        let a = halve(&mut sets.parent, a as usize);
        for corner in [b, c] {
            let b = halve(&mut sets.parent, corner as usize);
            if b != a {
                sets.parent[b as usize] = a;
            }
        }
    }
    Ok(sets)
}
