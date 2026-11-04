// SPDX-License-Identifier: AGPL-3.0-only
//! Complete prepared-array wire-slot/crease/render bodies from the frozen original.
//! UI setting coercion, worker binding and final gate/evidence promotion remain open.
use crate::{adjacency::Adjacency, mesh_graph::MeshGraph, normals::compute_vertex_normals};
/// Native admission/storage failure, distinct from a source split-budget refusal.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Malformed prepared mesh/CSR contract.
    Invalid,
    /// Caller storage policy exceeded.
    Limit,
    /// Fallible reservation failed.
    Allocation,
}
/// Caller bounds for actual input and generated output storage.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Maximum logical vertices.
    pub vertices: usize,
    /// Maximum triangle index entries.
    pub indices: usize,
    /// Maximum logical vertices plus copies.
    pub render_vertices: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            vertices: 1_000_000,
            indices: 6_000_000,
            render_vertices: 2_000_000,
        }
    }
}
fn filled<T: Clone>(n: usize, value: T) -> Result<Vec<T>, Error> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
    v.resize(n, value);
    Ok(v)
}
fn copied<T: Clone>(v: &[T]) -> Result<Vec<T>, Error> {
    let mut out = Vec::new();
    out.try_reserve_exact(v.len())
        .map_err(|_| Error::Allocation)?;
    out.extend_from_slice(v);
    Ok(out)
}
fn check(p: &[f32], ix: &[u32], limits: Limits) -> Result<usize, Error> {
    let n = p.len() / 3;
    if !p.len().is_multiple_of(3)
        || !ix.len().is_multiple_of(3)
        || ix.iter().any(|&i| i as usize >= n)
    {
        return Err(Error::Invalid);
    }
    if n > limits.vertices
        || n > limits.render_vertices
        || ix.len() > limits.indices
        || ix.len() > u32::MAX as usize
        || n > u32::MAX as usize
    {
        return Err(Error::Limit);
    }
    Ok(n)
}
/// Lowest available eight-slot colour, with the original least-count fallback/ties.
pub fn wire_slots(adj: &Adjacency, n: usize, limits: Limits) -> Result<Vec<u8>, Error> {
    if n > limits.vertices
        || adj.list.len() > u32::MAX as usize
        || adj.list.len() > limits.indices.saturating_mul(2)
        || n > u32::MAX as usize
        || n > limits.render_vertices
    {
        return Err(Error::Limit);
    }
    if adj.offsets.len() != n + 1
        || adj.offsets.first() != Some(&0)
        || adj.offsets.last().copied() != Some(adj.list.len() as u32)
        || adj.offsets.windows(2).any(|x| x[0] > x[1])
        || adj.list.iter().any(|&i| i as usize >= n)
    {
        return Err(Error::Invalid);
    }
    let mut slots = filled(n, 255u8)?;
    let mut stamped = [0u32; 8];
    let mut counts = [0u32; 8];
    for v in 0..n {
        let stamp = v as u32 + 1;
        let neighbors = &adj.list[adj.offsets[v] as usize..adj.offsets[v + 1] as usize];
        for &w in neighbors {
            let s = slots[w as usize];
            if s != 255 {
                stamped[s as usize] = stamp
            }
        }
        let mut pick = (0..8).find(|&s| stamped[s] != stamp);
        if pick.is_none() {
            counts.fill(0);
            for &w in neighbors {
                let s = slots[w as usize];
                if s != 255 {
                    counts[s as usize] += 1
                }
            }
            let mut best = 0;
            for s in 1..8 {
                if counts[s] < counts[best] {
                    best = s
                }
            }
            pick = Some(best)
        }
        slots[v] = pick.ok_or(Error::Invalid)? as u8;
    }
    Ok(slots)
}
/// Exact triangle conflict count; repeated corners retain their source meaning.
pub fn wire_conflicts(ix: &[u32], slots: &[u8]) -> Result<usize, Error> {
    if !ix.len().is_multiple_of(3) || ix.iter().any(|&v| v as usize >= slots.len()) {
        return Err(Error::Invalid);
    }
    Ok(ix
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|&&[a, b, c]| {
            slots[a as usize] == slots[b as usize]
                || slots[b as usize] == slots[c as usize]
                || slots[a as usize] == slots[c as usize]
        })
        .count())
}
fn face_normals(p: &[f32], ix: &[u32]) -> Result<Vec<f64>, Error> {
    let mut out = filled(ix.len(), 0.)?;
    for (t, &[a, b, c]) in ix.as_chunks::<3>().0.iter().enumerate() {
        let a = a as usize * 3;
        let b = b as usize * 3;
        let c = c as usize * 3;
        let ab = [
            f64::from(p[b]) - f64::from(p[a]),
            f64::from(p[b + 1]) - f64::from(p[a + 1]),
            f64::from(p[b + 2]) - f64::from(p[a + 2]),
        ];
        let ac = [
            f64::from(p[c]) - f64::from(p[a]),
            f64::from(p[c + 1]) - f64::from(p[a + 1]),
            f64::from(p[c + 2]) - f64::from(p[a + 2]),
        ];
        out[t * 3] = ab[1] * ac[2] - ab[2] * ac[1];
        out[t * 3 + 1] = ab[2] * ac[0] - ab[0] * ac[2];
        out[t * 3 + 2] = ab[0] * ac[1] - ab[1] * ac[0]
    }
    Ok(out)
}
fn fan(ix: &[u32], n: usize) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let mut offsets = filled(n + 1, 0usize)?;
    for &i in ix {
        offsets[i as usize + 1] += 1
    }
    for v in 0..n {
        offsets[v + 1] += offsets[v]
    }
    let mut list = filled(ix.len(), 0usize)?;
    let mut cursor = copied(&offsets[..n])?;
    for (i, &v) in ix.iter().enumerate() {
        list[cursor[v as usize]] = i / 3;
        cursor[v as usize] += 1
    }
    Ok((offsets, list))
}
fn cos_between(f: &[f64], i: usize, j: usize) -> f64 {
    let i = i * 3;
    let j = j * 3;
    let li = f[i] * f[i] + f[i + 1] * f[i + 1] + f[i + 2] * f[i + 2];
    let lj = f[j] * f[j] + f[j + 1] * f[j + 1] + f[j + 2] * f[j + 2];
    if li < 1e-30 || lj < 1e-30 {
        return 1.;
    }
    (f[i] * f[j] + f[i + 1] * f[j + 1] + f[i + 2] * f[j + 2]) / (li * lj).sqrt()
}
/// Original doubled fan-edge flat share, including degenerate/non-manifold order.
pub fn coplanar_share(p: &[f32], ix: &[u32], limits: Limits) -> Result<f64, Error> {
    let n = check(p, ix, limits)?;
    let f = face_normals(p, ix)?;
    let (offsets, list) = fan(ix, n)?;
    let mut stamp = filled(n, 0usize)?;
    let mut first = filled(n, 0usize)?;
    let flat = (0.01 * std::f64::consts::PI / 180.).cos();
    let (mut edges, mut coplanar) = (0usize, 0usize);
    for v in 0..n {
        let begin = offsets[v];
        let end = offsets[v + 1];
        if end - begin < 2 {
            continue;
        }
        for k in begin..end {
            let t = list[k] * 3;
            for c in 0..3 {
                let w = ix[t + c] as usize;
                if w == v {
                    continue;
                }
                if stamp[w] == v + 1 {
                    edges += 1;
                    if cos_between(&f, list[begin + first[w]], list[k]) >= flat {
                        coplanar += 1
                    }
                } else {
                    stamp[w] = v + 1;
                    first[w] = k - begin
                }
            }
        }
    }
    Ok(if edges == 0 {
        0.
    } else {
        coplanar as f64 / edges as f64
    })
}
/// Original scan split budget, never below 250,000 copies.
pub fn crease_budget(n: usize) -> usize {
    250_000.max(n)
}
/// Owned render split arrays in original order.
#[derive(Debug)]
pub struct Split {
    /// Original positions followed by source-ordered copies.
    pub positions: Vec<f32>,
    /// Triangles retaining original order, with copy indices.
    pub indices: Vec<u32>,
    /// Per-side area-weighted normals.
    pub normals: Vec<f32>,
    /// One slot per original/copy vertex.
    pub wire_slots: Vec<u8>,
    /// Logical source identity of each appended vertex.
    pub copy_of: Vec<u32>,
}
fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i]
    }
    i
}
/// Full two-pass fan grouping/copy continuation. None is source budget refusal.
pub fn split_creases(
    p: &[f32],
    ix: &[u32],
    slots: &[u8],
    angle: f64,
    budget: usize,
    limits: Limits,
) -> Result<Option<Split>, Error> {
    let n = check(p, ix, limits)?;
    if slots.len() != n {
        return Err(Error::Invalid);
    }
    let f = face_normals(p, ix)?;
    let (offsets, list) = fan(ix, n)?;
    let smooth = (angle * std::f64::consts::PI / 180.).cos();
    let mut parent = filled(64, 0usize)?;
    let mut stamp = filled(n, 0usize)?;
    let mut first = filled(n, 0usize)?;
    let mut group = filled(ix.len(), 0usize)?;
    for v in 0..n {
        let begin = offsets[v];
        let k = offsets[v + 1] - begin;
        if k < 2 {
            continue;
        }
        if parent.len() < k {
            parent = filled(k * 2, 0)?
        }
        for (i, x) in parent[..k].iter_mut().enumerate() {
            *x = i
        }
        for i in 0..k {
            let t = list[begin + i] * 3;
            for c in 0..3 {
                let w = ix[t + c] as usize;
                if w == v {
                    continue;
                }
                if stamp[w] == v + 1 {
                    let j = first[w];
                    if cos_between(&f, list[begin + i], list[begin + j]) >= smooth {
                        let ri = root(&mut parent, i);
                        let rj = root(&mut parent, j);
                        if ri != rj {
                            parent[ri] = rj
                        }
                    }
                } else {
                    stamp[w] = v + 1;
                    first[w] = i
                }
            }
        }
        for i in 0..k {
            let t = list[begin + i] * 3;
            let c = if ix[t] as usize == v {
                0
            } else if ix[t + 1] as usize == v {
                1
            } else {
                2
            };
            group[t + c] = root(&mut parent, i)
        }
    }
    let mut copies = Vec::new();
    let mut new_ix = copied(ix)?;
    let mut assigned = filled(64, None)?;
    for v in 0..n {
        let begin = offsets[v];
        let k = offsets[v + 1] - begin;
        if k < 2 {
            continue;
        }
        if assigned.len() < k {
            assigned = filled(k * 2, None)?
        }
        assigned[..k].fill(None);
        let mut keep = None;
        for i in 0..k {
            let t = list[begin + i] * 3;
            let c = if ix[t] as usize == v {
                0
            } else if ix[t + 1] as usize == v {
                1
            } else {
                2
            };
            let g = group[t + c];
            if keep.is_none() {
                keep = Some(g)
            }
            if keep == Some(g) {
                continue;
            }
            if assigned[g].is_none() {
                if copies.len() >= budget {
                    return Ok(None);
                }
                let next = n + copies.len();
                if next >= limits.render_vertices || next > u32::MAX as usize {
                    return Err(Error::Limit);
                }
                copies.try_reserve(1).map_err(|_| Error::Allocation)?;
                assigned[g] = Some(next as u32);
                copies.push(v as u32)
            }
            new_ix[t + c] = assigned[g].ok_or(Error::Invalid)?;
        }
    }
    let total = n + copies.len();
    if total > limits.render_vertices {
        return Err(Error::Limit);
    }
    let mut positions = filled(total.checked_mul(3).ok_or(Error::Limit)?, 0f32)?;
    positions[..p.len()].copy_from_slice(p);
    let mut new_slots = filled(total, 0u8)?;
    new_slots[..n].copy_from_slice(slots);
    for (k, &v) in copies.iter().enumerate() {
        let d = (n + k) * 3;
        let v = v as usize;
        positions[d..d + 3].copy_from_slice(&p[v * 3..v * 3 + 3]);
        new_slots[n + k] = slots[v]
    }
    let normals = compute_vertex_normals(&positions, &new_ix, limits.render_vertices)
        .map_err(|_| Error::Allocation)?;
    Ok(Some(Split {
        positions,
        indices: new_ix,
        normals,
        wire_slots: new_slots,
        copy_of: copies,
    }))
}
/// Original on/off/auto modes.
#[derive(Clone, Copy)]
pub enum Mode {
    /// Always attempt splitting.
    On,
    /// Preserve smooth graph arrays.
    Off,
    /// Split only when the source flat-share test passes.
    Auto,
}
/// Source skip reason; None is a completed split, even if zero copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skipped {
    /// Preserve smooth graph arrays.
    Off,
    /// Auto classified this mesh as a scan.
    Scan,
    /// Source copy budget refused the whole split.
    Budget,
}
/// Source report with exact appended-copy count.
#[derive(Debug)]
pub struct Report {
    /// Number of appended vertex copies.
    pub added: usize,
    /// Original skip reason, or a completed split.
    pub skipped: Option<Skipped>,
}
/// Complete source mode and budget decisions.
pub fn crease_for(
    mode: Mode,
    p: &[f32],
    ix: &[u32],
    slots: &[u8],
    angle: f64,
    limits: Limits,
) -> Result<(Option<Split>, Report), Error> {
    if matches!(mode, Mode::Off) {
        return Ok((
            None,
            Report {
                added: 0,
                skipped: Some(Skipped::Off),
            },
        ));
    }
    if matches!(mode, Mode::Auto) && coplanar_share(p, ix, limits)? < 0.1 {
        return Ok((
            None,
            Report {
                added: 0,
                skipped: Some(Skipped::Scan),
            },
        ));
    }
    let split = split_creases(p, ix, slots, angle, crease_budget(p.len() / 3), limits)?;
    let report = Report {
        added: split.as_ref().map_or(0, |s| s.copy_of.len()),
        skipped: if split.is_some() {
            None
        } else {
            Some(Skipped::Budget)
        },
    };
    Ok((split, report))
}
/// Complete prepared graph render consumer; skipped modes clone original graph arrays.
pub fn scan_render(
    g: &MeshGraph,
    mode: Mode,
    angle: f64,
    limits: Limits,
) -> Result<(Split, Report), Error> {
    check(&g.positions, &g.indices, limits)?;
    if g.vertex_count != g.positions.len() / 3 || g.normals.len() != g.positions.len() {
        return Err(Error::Invalid);
    }
    let slots = wire_slots(&g.adjacency, g.vertex_count, limits)?;
    let (split, report) = crease_for(mode, &g.positions, &g.indices, &slots, angle, limits)?;
    Ok((
        match split {
            Some(s) => s,
            None => Split {
                positions: copied(&g.positions)?,
                indices: copied(&g.indices)?,
                normals: copied(&g.normals)?,
                wire_slots: slots,
                copy_of: vec![],
            },
        },
        report,
    ))
}
