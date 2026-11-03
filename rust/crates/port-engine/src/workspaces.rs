// SPDX-License-Identifier: AGPL-3.0-only
//! Complete publicWorkspace/internalWorkspace/workspaceNames over owned metadata.
const ELEMENTS: &[u16] = &[101, 108, 101, 109, 101, 110, 116, 115];
const MEASURE: &[u16] = &[109, 101, 97, 115, 117, 114, 101];
const BASE: [&[u16]; 4] = [
    MEASURE,
    &[100, 101, 118, 105, 97, 116, 105, 111, 110],
    &[116, 104, 105, 99, 107, 110, 101, 115, 115],
    &[102, 108, 97, 116],
];
/// Only the internal elements name is remapped. Every other string is unchanged,
/// including unknown, empty, case-distinct and unpaired UTF-16 inputs.
pub fn public_workspace(name: &[u16]) -> &[u16] {
    if name == ELEMENTS { MEASURE } else { name }
}
/// Only the public measure name is remapped; this does not validate availability.
pub fn internal_workspace(name: &[u16]) -> &[u16] {
    if name == MEASURE { ELEMENTS } else { name }
}
/// Explicit host copy/work limits, separate from source workspace policy.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Number of installed plugins examined, including those without a workspace.
    pub plugins: usize,
    /// Number of names copied, including the four base names and empty IDs.
    pub names: usize,
    /// Total output UTF-16 units.
    pub text_units: usize,
}
/// Native refusal before publishing any partial name array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Explicit plugin/name/unit bound or count overflow.
    Limit,
    /// Fallible result/text reservation failed.
    Allocation,
}
/// Four source base names followed by every present workspace id in current
/// installed order. None represents no workspace; Some(empty) is still a name.
/// There is no extra alias, sorting, deduplication or ID validation. Every call
/// returns independent arrays/text so later caller/metadata changes stay separate.
pub fn workspace_names(
    workspaces: &[Option<&[u16]>],
    limits: Limits,
) -> Result<Vec<Vec<u16>>, Error> {
    if workspaces.len() > limits.plugins {
        return Err(Error::Limit);
    }
    let mut count = BASE.len();
    let mut units = BASE.iter().map(|s| s.len()).sum::<usize>();
    for id in workspaces.iter().flatten() {
        count = count.checked_add(1).ok_or(Error::Limit)?;
        units = units.checked_add(id.len()).ok_or(Error::Limit)?;
        if count > limits.names || units > limits.text_units {
            return Err(Error::Limit);
        }
    }
    if count > limits.names || units > limits.text_units {
        return Err(Error::Limit);
    }
    let mut out = Vec::new();
    out.try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    for id in BASE.into_iter().chain(workspaces.iter().filter_map(|s| *s)) {
        let mut text = Vec::new();
        text.try_reserve_exact(id.len())
            .map_err(|_| Error::Allocation)?;
        text.extend_from_slice(id);
        out.push(text);
    }
    Ok(out)
}
