// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen region.ts::trianglesWithin and remap.ts::remapVertices.

/// Bounded storage failure; no partial selection escapes.
#[derive(Debug, PartialEq, Eq)]
pub enum SelectionError {
    /// Caller vertex or output-index budget exceeded.
    StorageLimit,
    /// Storage reservation failed.
    Allocation,
}
fn reserve<T>(n: usize) -> Result<Vec<T>, SelectionError> {
    let mut out = Vec::new();
    out.try_reserve_exact(n)
        .map_err(|_| SelectionError::Allocation)?;
    Ok(out)
}

/// Renumber selected IDs through the signed map, dropping missing/negative maps.
/// Input order and duplicate IDs are retained, without sorting the result.
pub fn remap_vertices(
    ids: &[u32],
    vertex_map: &[i32],
    max_output: usize,
) -> Result<Vec<u32>, SelectionError> {
    let count = ids
        .iter()
        .filter(|&&id| vertex_map.get(id as usize).is_some_and(|&w| w >= 0))
        .count();
    if count > max_output {
        return Err(SelectionError::StorageLimit);
    }
    let mut out = reserve(count)?;
    for &id in ids {
        let w = vertex_map.get(id as usize).copied().unwrap_or(-1);
        if w >= 0 {
            out.push(w as u32);
        }
    }
    Ok(out)
}

/// Keep only complete triangles with all three corners in the marked set.
/// Out-of-range marks/corners are ignored/excluded; incomplete trailing index
/// triples are skipped, just as in the source. Triangle order/repeats remain.
pub fn triangles_within(
    indices: &[u32],
    vertices: &[u32],
    vertex_count: usize,
    max_vertices: usize,
    max_output: usize,
) -> Result<Vec<u32>, SelectionError> {
    if vertex_count > max_vertices {
        return Err(SelectionError::StorageLimit);
    }
    let mut inside = reserve(vertex_count)?;
    inside.resize(vertex_count, 0u8);
    for &v in vertices {
        if let Some(mark) = inside.get_mut(v as usize) {
            *mark = 1;
        }
    }
    let keep = |triangle: &[u32; 3]| triangle.iter().all(|&v| inside.get(v as usize) == Some(&1));
    let count = indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|t| keep(t))
        .count();
    let output_count = count.checked_mul(3).ok_or(SelectionError::StorageLimit)?;
    if output_count > max_output {
        return Err(SelectionError::StorageLimit);
    }
    let mut out = reserve(output_count)?;
    for triangle in indices.as_chunks::<3>().0 {
        if keep(triangle) {
            out.extend_from_slice(triangle);
        }
    }
    Ok(out)
}
