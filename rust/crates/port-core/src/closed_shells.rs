// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen closedShells.ts parity classification, including private bit/hash rules.

/// Original triangle flags and total classified by the two-word parity hash.
#[derive(Debug, PartialEq, Eq)]
pub struct ClosedShells {
    /// One byte per complete triangle, in source order.
    pub closed: Vec<u8>,
    /// Number of triangles whose component's parity tally is zero.
    pub count: usize,
}
/// Bounded host errors; source trailing coordinate/index elements remain ignored.
#[derive(Debug, PartialEq, Eq)]
pub enum ShellError {
    /// A complete triangle references a missing vertex.
    InvalidIndex,
    /// Caller storage budget or safe source hash-table capacity exceeded.
    StorageLimit,
    /// A storage reservation failed.
    Allocation,
    /// An internal disjoint-set contract was violated.
    HelperContract,
}
fn storage<T: Clone>(n: usize, value: T) -> Result<Vec<T>, ShellError> {
    let mut out = Vec::new();
    out.try_reserve_exact(n)
        .map_err(|_| ShellError::Allocation)?;
    out.resize(n, value);
    Ok(out)
}
fn hash3(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(0x9e3779b1);
    h ^= b;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h ^= c;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^ (h >> 16)
}
fn edge_hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = (x ^ seed).wrapping_mul(0xcc9e2d51);
    h = h.rotate_left(15);
    h = h.wrapping_mul(0x1b873593);
    h ^= y;
    h = (h ^ (h >> 16)).wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^ (h >> 16)
}
fn weld_ids(positions: &[f32]) -> Result<(Vec<u32>, usize), ShellError> {
    let n = positions.len() / 3;
    let bit = |i: usize| {
        let b = positions[i].to_bits();
        if b == 0x80000000 { 0 } else { b }
    };
    let mut cap = 16;
    while cap < n * 2 {
        cap *= 2;
    }
    let mask = cap - 1;
    let mut table = storage(cap, u32::MAX)?;
    let mut first = storage(n, 0usize)?;
    let mut id = storage(n, 0u32)?;
    let mut count = 0;
    for (v, id_v) in id.iter_mut().enumerate() {
        let (bx, by, bz) = (bit(3 * v), bit(3 * v + 1), bit(3 * v + 2));
        let mut h = hash3(bx, by, bz) as usize & mask;
        loop {
            let slot = table[h];
            if slot == u32::MAX {
                table[h] = count as u32;
                first[count] = v;
                *id_v = count as u32;
                count += 1;
                break;
            }
            let w = first[slot as usize];
            if bit(3 * w) == bx && bit(3 * w + 1) == by && bit(3 * w + 2) == bz {
                *id_v = slot;
                break;
            }
            h = (h + 1) & mask;
        }
    }
    Ok((id, count))
}
fn component_error(e: crate::components::ComponentError) -> ShellError {
    match e {
        crate::components::ComponentError::Allocation => ShellError::Allocation,
        crate::components::ComponentError::VertexLimit => ShellError::StorageLimit,
        _ => ShellError::HelperContract,
    }
}
/// Classify complete triangles using source bit welding and two-word edge parity.
/// This preserves the original probabilistic/even-edge criterion, not a manifold
/// guarantee. Signed zero welds together here; nonfinite coordinate bits remain.
/// Incomplete tails are ignored. Only consumed indices are validated, before work.
pub fn closed_shells(
    positions: &[f32],
    indices: Option<&[u32]>,
    max_vertices: usize,
    max_triangles: usize,
) -> Result<ClosedShells, ShellError> {
    let vertex_count = positions.len() / 3;
    let triangles = indices.map_or(vertex_count, |i| i.len()) / 3;
    if vertex_count > max_vertices || vertex_count > (1 << 29) || triangles > max_triangles {
        return Err(ShellError::StorageLimit);
    }
    if indices.is_some_and(|i| {
        i[..triangles * 3]
            .iter()
            .any(|&v| v as usize >= vertex_count)
    }) {
        return Err(ShellError::InvalidIndex);
    }
    let corner = |i: usize| indices.map_or(i, |idx| idx[i] as usize);
    let (id, welded) = weld_ids(positions)?;
    let mut sets =
        crate::components::make_union_find(welded, max_vertices).map_err(component_error)?;
    for t in 0..triangles {
        let a = id[corner(3 * t)];
        sets.union(id[corner(3 * t + 1)], a)
            .map_err(component_error)?;
        sets.union(id[corner(3 * t + 2)], a)
            .map_err(component_error)?;
    }
    let mut lo = storage(welded, 0u32)?;
    let mut hi = storage(welded, 0u32)?;
    for t in 0..triangles {
        let [c0, c1, c2] = [
            id[corner(3 * t)],
            id[corner(3 * t + 1)],
            id[corner(3 * t + 2)],
        ];
        let r = sets.find(c0).map_err(component_error)? as usize;
        for (p, q) in [(c0, c1), (c1, c2), (c2, c0)] {
            if p == q {
                continue;
            }
            let (x, y) = if p < q { (p, q) } else { (q, p) };
            lo[r] ^= edge_hash(x, y, 0x9e3779b1);
            hi[r] ^= edge_hash(x, y, 0x85ebca77);
        }
    }
    let mut closed = storage(triangles, 0u8)?;
    let mut count = 0;
    for (t, flag) in closed.iter_mut().enumerate() {
        let r = sets.find(id[corner(3 * t)]).map_err(component_error)? as usize;
        if lo[r] == 0 && hi[r] == 0 {
            *flag = 1;
            count += 1;
        }
    }
    Ok(ClosedShells { closed, count })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    struct Case {
        id: String,
        p: Vec<f32>,
        indices: Option<Vec<u32>>,
    }
    fn cases() -> Vec<Case> {
        let tetra = vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.];
        let faces = vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
        let triangle = vec![0., 0., 0., 1., 0., 0., 0., 1., 0.];
        let mut out = vec![
            Case {
                id: "empty".into(),
                p: vec![],
                indices: None,
            },
            Case {
                id: "coordinate_tail".into(),
                p: vec![f32::NAN, 0.],
                indices: None,
            },
            Case {
                id: "triangle".into(),
                p: triangle.clone(),
                indices: None,
            },
            Case {
                id: "duplicate_even_parity".into(),
                p: triangle.clone(),
                indices: Some(vec![0, 1, 2, 0, 1, 2]),
            },
            Case {
                id: "degenerate_closed_parity".into(),
                p: triangle.clone(),
                indices: Some(vec![0, 0, 0]),
            },
            Case {
                id: "empty_indices".into(),
                p: tetra.clone(),
                indices: Some(vec![]),
            },
            Case {
                id: "ignored_index_tail".into(),
                p: triangle.clone(),
                indices: Some(vec![0, 1, 2, u32::MAX, u32::MAX]),
            },
            Case {
                id: "tetra_indexed".into(),
                p: tetra.clone(),
                indices: Some(faces.clone()),
            },
            Case {
                id: "tetra_missing_face".into(),
                p: tetra.clone(),
                indices: Some(faces[..9].to_vec()),
            },
        ];
        for signed_zero in [false, true] {
            let mut p: Vec<_> = faces
                .iter()
                .flat_map(|&i| tetra[i as usize * 3..i as usize * 3 + 3].iter().copied())
                .collect();
            if signed_zero {
                for (i, v) in p.iter_mut().enumerate() {
                    if *v == 0. && i % 2 == 0 {
                        *v = -0.;
                    }
                }
            }
            out.push(Case {
                id: format!("tetra_split_zero{signed_zero}"),
                p,
                indices: None,
            });
        }
        for bits in [0x7fc00001, 0x7fa00001, 0xffc00001, 0x7f800000] {
            let value = f32::from_bits(bits);
            out.push(Case {
                id: format!("raw_bits_{bits:08x}"),
                p: vec![
                    value,
                    0.,
                    0.,
                    value,
                    0.,
                    0.,
                    f32::from_bits(bits ^ 1),
                    0.,
                    0.,
                ],
                indices: None,
            });
        }
        let mut two = tetra.clone();
        two.extend(
            tetra
                .iter()
                .enumerate()
                .map(|(i, &v)| v + if i % 3 == 0 { 4. } else { 0. }),
        );
        let mut both = faces.clone();
        both.extend(faces.iter().map(|&i| i + 4));
        out.push(Case {
            id: "disconnected_closed".into(),
            p: two,
            indices: Some(both),
        });
        for seed in [1u32, 7, 99, 1039] {
            for n in [1, 17, 64] {
                for repeated in [false, true] {
                    let mut state = seed;
                    let mut next = || {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        state
                    };
                    let p: Vec<_> = (0..n * 3).map(|_| (next() % 19) as f32 - 9.).collect();
                    let mut idx: Vec<_> = (0..93).map(|_| next() % n as u32).collect();
                    if repeated {
                        idx.extend(idx.clone());
                    }
                    for soup in [false, true] {
                        let positions = if soup {
                            idx.iter()
                                .flat_map(|&i| {
                                    p[i as usize * 3..i as usize * 3 + 3].iter().copied()
                                })
                                .collect()
                        } else {
                            p.clone()
                        };
                        out.push(Case {
                            id: format!("seed_{seed}_n{n}_repeat{repeated}_s{soup}"),
                            p: positions,
                            indices: if soup { None } else { Some(idx.clone()) },
                        });
                    }
                }
            }
        }
        out
    }
    fn output(c: &Case) -> Vec<u8> {
        let result = closed_shells(&c.p, c.indices.as_deref(), 10000, 10000).unwrap();
        let (id, count) = weld_ids(&c.p).unwrap();
        let mut out = (result.count as u32).to_le_bytes().to_vec();
        out.extend((result.closed.len() as u32).to_le_bytes());
        out.extend(result.closed);
        out.extend((count as u32).to_le_bytes());
        out.extend((id.len() as u32).to_le_bytes());
        out.extend(id.iter().flat_map(|v| v.to_le_bytes()));
        for (x, y) in [
            (0, 1),
            (1, 0),
            (1, 1),
            (u32::MAX, 1),
            (0x80000000, 0x7fffffff),
        ] {
            for seed in [0, 1, 0x9e3779b1, u32::MAX] {
                out.extend(edge_hash(x, y, seed).to_le_bytes());
                out.extend(hash3(x, y, seed).to_le_bytes());
            }
        }
        out
    }
    #[test]
    fn original_parity_and_host_guards() {
        let p = [0., 0., 0., 1., 0., 0., 0., 1., 0.];
        assert_eq!(
            closed_shells(&p, Some(&[0, 1, 2, 0, 1, 2]), 3, 2)
                .unwrap()
                .closed,
            [1, 1]
        );
        assert_eq!(
            closed_shells(&p, Some(&[0, 0, 0]), 3, 1).unwrap().closed,
            [1]
        );
        assert_eq!(closed_shells(&p, None, 3, 1).unwrap().closed, [0]);
        assert_eq!(closed_shells(&p, None, 2, 1), Err(ShellError::StorageLimit));
        assert_eq!(closed_shells(&p, None, 3, 0), Err(ShellError::StorageLimit));
        assert_eq!(
            closed_shells(&p, Some(&[0, 1, 3]), 3, 1),
            Err(ShellError::InvalidIndex)
        );
        assert!(
            closed_shells(&p, Some(&[u32::MAX, u32::MAX]), 3, 0)
                .unwrap()
                .closed
                .is_empty()
        );
    }
    #[test]
    #[ignore = "requires frozen live Node oracle"]
    fn pinned_closed_shells_comparison() {
        use std::{
            io::Write,
            process::{Command, Stdio},
        };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = root.join(format!("rust/target/closed-shells-diff/run-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(sha.status.success());
        std::fs::write(dir.join("candidate.txt"), sha.stdout).unwrap();
        println!("raw closed-shell evidence: {}", dir.display());
        let cases = cases();
        assert_eq!(cases.len(), 64);
        for c in cases {
            let raw = output(&c);
            let mut input = u32::from(c.indices.is_some()).to_le_bytes().to_vec();
            input.extend((c.p.len() as u32).to_le_bytes());
            input.extend(c.p.iter().flat_map(|v| v.to_le_bytes()));
            if let Some(idx) = &c.indices {
                input.extend(idx.iter().flat_map(|v| v.to_le_bytes()));
            }
            let mut child = Command::new("node")
                .arg(root.join("docs/port-plan/oracle/closed-shells.cjs"))
                .current_dir(&root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(&input).unwrap();
            let original = child.wait_with_output().unwrap();
            let d = dir.join(&c.id);
            std::fs::create_dir_all(&d).unwrap();
            for (name, data) in [
                ("input.bin", input.as_slice()),
                ("original.bin", original.stdout.as_slice()),
                ("rust.bin", raw.as_slice()),
            ] {
                std::fs::write(d.join(name), data).unwrap();
            }
            assert!(
                original.status.success(),
                "{}: {}",
                c.id,
                String::from_utf8_lossy(&original.stderr)
            );
            assert_eq!(
                raw, original.stdout,
                "{}: flags/private ID/hash state differs",
                c.id
            );
            println!("{}: flags, welded IDs and 40 hash words exact", c.id);
        }
    }
}
