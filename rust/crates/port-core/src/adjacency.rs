// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen geometry/adjacency.ts::buildAdjacency, with bounded host storage.

/// CSR graph with source append order and duplicate neighbors retained.
#[derive(Debug, PartialEq, Eq)]
pub struct Adjacency {
    /// Start/end offsets, one more entry than vertices.
    pub offsets: Vec<u32>,
    /// Neighbor identities, two per triangle corner.
    pub list: Vec<u32>,
}

/// Invalid mesh contract or bounded storage failure; no partial graph escapes.
#[derive(Debug, PartialEq, Eq)]
pub enum AdjacencyError {
    /// Incomplete triangles or an index outside the vertex prefix.
    InvalidMesh,
    /// Caller vertex/neighbor budget or u32 offset capacity exceeded.
    StorageLimit,
    /// A storage reservation failed.
    Allocation,
}
fn zeroes(n: usize) -> Result<Vec<u32>, AdjacencyError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| AdjacencyError::Allocation)?;
    v.resize(n, 0);
    Ok(v)
}

/// Build vertex-to-vertex adjacency without sorting or deduplicating neighbors.
/// Limits bound vertices and total neighbors (twice the input index count).
/// Invalid/oversized input fails before graph allocation or computation.
pub fn build_adjacency(
    indices: &[u32],
    vertex_count: usize,
    max_vertices: usize,
    max_neighbors: usize,
) -> Result<Adjacency, AdjacencyError> {
    if !indices.len().is_multiple_of(3) || indices.iter().any(|&i| i as usize >= vertex_count) {
        return Err(AdjacencyError::InvalidMesh);
    }
    let neighbors = indices
        .len()
        .checked_mul(2)
        .ok_or(AdjacencyError::StorageLimit)?;
    let offset_count = vertex_count
        .checked_add(1)
        .ok_or(AdjacencyError::StorageLimit)?;
    if vertex_count > max_vertices
        || vertex_count > u32::MAX as usize
        || neighbors > max_neighbors
        || neighbors > u32::MAX as usize
    {
        return Err(AdjacencyError::StorageLimit);
    }
    let mut offsets = zeroes(offset_count)?;
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        offsets[a as usize + 1] += 2;
        offsets[b as usize + 1] += 2;
        offsets[c as usize + 1] += 2;
    }
    for v in 0..vertex_count {
        offsets[v + 1] += offsets[v];
    }
    let mut list = zeroes(offsets[vertex_count] as usize)?;
    let mut cursor = zeroes(vertex_count)?;
    cursor.copy_from_slice(&offsets[..vertex_count]);
    for &[a, b, c] in indices.as_chunks::<3>().0 {
        for (v, neighbor) in [(a, b), (a, c), (b, a), (b, c), (c, a), (c, b)] {
            list[cursor[v as usize] as usize] = neighbor;
            cursor[v as usize] += 1;
        }
    }
    Ok(Adjacency { offsets, list })
}
