// SPDX-License-Identifier: AGPL-3.0-only
//! Complete frozen buildGraph.ts pipeline, composing pure-Rust geometry helpers.

/// Parsed coordinate/index ownership transferred into graph construction.
#[derive(Debug)]
pub struct ParsedMesh {
    /// Packed f32 XYZ coordinates, retained as-is for indexed input.
    pub positions: Vec<f32>,
    /// Source soup kind: weld all corners; otherwise retain supplied indexing.
    pub soup: bool,
    /// Optional source indices (validated even when soup ignores them afterward).
    pub indices: Option<Vec<u32>>,
    /// Source keepWinding flag, default false at the caller boundary.
    pub keep_winding: bool,
}
/// Explicit host limits; raw soup corners count toward the vertex budget.
#[derive(Debug, Clone, Copy)]
pub struct GraphLimits {
    /// Maximum input vertex/corner count.
    pub vertices: usize,
    /// Maximum raw index count (including soup's generated index count).
    pub indices: usize,
    /// Maximum adjacency neighbor count.
    pub neighbors: usize,
}
/// One indexed graph, retaining source output identities and arithmetic.
#[derive(Debug)]
pub struct MeshGraph {
    /// Packed source/welded coordinates.
    pub positions: Vec<f32>,
    /// Filtered, optionally reversed triangle indices.
    pub indices: Vec<u32>,
    /// Area-weighted, optionally reversed vertex normals.
    pub normals: Vec<f32>,
    /// Topology computed before winding reversal, as in the source.
    pub adjacency: crate::adjacency::Adjacency,
    /// Welded/indexed vertex count.
    pub vertex_count: usize,
    /// Usable triangle count.
    pub triangle_count: usize,
    /// Full source bounding-box diagonal, with source squared-sum order.
    pub bbox_diag: f64,
}
/// Source refusals or explicit bounded host failure.
#[derive(Debug, PartialEq, Eq)]
pub enum GraphError {
    /// Exact original validation message.
    Source(&'static str),
    /// Caller storage limit or helper capacity was exceeded.
    StorageLimit,
    /// A fallible allocation failed.
    Allocation,
    /// Internal helper input contract was violated.
    HelperContract,
}
fn weld_error(e: crate::weld::WeldError) -> GraphError {
    match e {
        crate::weld::WeldError::Allocation => GraphError::Allocation,
        crate::weld::WeldError::VertexLimit => GraphError::StorageLimit,
        crate::weld::WeldError::IncompleteTriple => GraphError::HelperContract,
    }
}
/// Build a full graph, preserving source validation, progress and continuation order.
/// Source failures return their unchanged message. No partially built graph escapes;
/// native ownership replaces JavaScript's shared indexed-array aliasing.
pub fn build_mesh_graph(
    parsed: ParsedMesh,
    limits: GraphLimits,
    mut on_progress: impl FnMut(&str),
) -> Result<MeshGraph, GraphError> {
    if parsed.positions.is_empty()
        || !parsed.positions.len().is_multiple_of(3)
        || (parsed.soup && !parsed.positions.len().is_multiple_of(9))
    {
        return Err(GraphError::Source(
            "The file contains incomplete vertex or triangle coordinates.",
        ));
    }
    if parsed.positions.iter().any(|v| !v.is_finite()) {
        return Err(GraphError::Source(
            "The file contains non-finite vertex coordinates.",
        ));
    }
    if let Some(indices) = &parsed.indices {
        if !indices.len().is_multiple_of(3) {
            return Err(GraphError::Source(
                "The file contains incomplete triangle indices.",
            ));
        }
        if indices
            .iter()
            .any(|&i| i as usize >= parsed.positions.len() / 3)
        {
            return Err(GraphError::Source(
                "The file references vertices that do not exist.",
            ));
        }
    }
    if parsed.positions.len() / 3 > limits.vertices
        || parsed
            .indices
            .as_ref()
            .is_some_and(|i| i.len() > limits.indices)
        || (parsed.soup && parsed.positions.len() / 3 > limits.indices)
    {
        return Err(GraphError::StorageLimit);
    }
    let (positions, indices) = if parsed.soup {
        let welded =
            crate::weld::weld_triangle_soup(&parsed.positions, limits.vertices, &mut on_progress)
                .map_err(weld_error)?;
        (welded.positions, welded.indices)
    } else {
        (parsed.positions, parsed.indices.unwrap_or_default())
    };
    let mut indices = crate::weld::filter_degenerate_triangles(&indices).map_err(weld_error)?;
    if indices.is_empty() {
        return Err(GraphError::Source(
            "The file contains no usable triangles — point clouds are not supported yet.",
        ));
    }
    let vertex_count = positions.len() / 3;
    if indices.iter().any(|&i| i as usize >= vertex_count) {
        return Err(GraphError::Source(
            "The file references vertices that do not exist.",
        ));
    }
    on_progress("Building surface topology…");
    let adjacency = crate::adjacency::build_adjacency(
        &indices,
        vertex_count,
        limits.vertices,
        limits.neighbors,
    )
    .map_err(|e| match e {
        crate::adjacency::AdjacencyError::Allocation => GraphError::Allocation,
        crate::adjacency::AdjacencyError::StorageLimit => GraphError::StorageLimit,
        crate::adjacency::AdjacencyError::InvalidMesh => GraphError::HelperContract,
    })?;
    on_progress("Computing normals…");
    let mut normals = crate::normals::compute_vertex_normals(&positions, &indices, limits.vertices)
        .map_err(|e| match e {
            crate::normals::NormalError::Allocation => GraphError::Allocation,
            crate::normals::NormalError::VertexLimit => GraphError::StorageLimit,
            crate::normals::NormalError::InvalidMesh => GraphError::HelperContract,
        })?;
    if !parsed.keep_winding {
        crate::orientation::orient_normals_outward(&positions, &mut indices, &mut normals)
            .map_err(|_| GraphError::HelperContract)?;
    }
    let crate::bounds::Extent {
        min: [min_x, min_y, min_z],
        max: [max_x, max_y, max_z],
    } = crate::bounds::extent_of(&positions, vertex_count)
        .map_err(|_| GraphError::HelperContract)?;
    let x = max_x - min_x;
    let y = max_y - min_y;
    let z = max_z - min_z;
    let bbox_diag = (x * x + y * y + z * z).sqrt();
    let triangle_count = indices.len() / 3;
    Ok(MeshGraph {
        positions,
        indices,
        normals,
        adjacency,
        vertex_count,
        triangle_count,
        bbox_diag,
    })
}
