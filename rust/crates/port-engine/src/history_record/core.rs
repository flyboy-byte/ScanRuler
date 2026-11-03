// SPDX-License-Identifier: AGPL-3.0-only
//! Complete captureCore and apply over borrowed owned data; no second document store.
use super::{Cell, Compare, Limits, Patch, Section, Snapshot};
use crate::schema_value::Text;
use std::sync::Arc;
/// Original measure picked fields, in source order.
pub const MEASURE: &[&str] = &[
    "elements",
    "dimensions",
    "sections",
    "nextId",
    "nextNumber",
    "nextOfKind",
    "nextDimensionId",
    "nextOfDimGroup",
    "nextSectionNumber",
    "appliedAlignment",
    "modelCenter",
];
/// Original flat picked fields, in source order.
pub const FLAT: &[&str] = &[
    "subject",
    "sheets",
    "pxPerMm",
    "calSource",
    "elements",
    "nextId",
    "nameCounts",
    "dimensions",
    "nextDimId",
    "dimCounts",
    "datum",
    "counts",
    "nextCountId",
    "notes",
    "nextNoteId",
    "turns",
    "mirror",
    "splitAxes",
    "edgeSensitivity",
    "showEdges",
    "snapToEdge",
    "showGrid",
];
/// Original deviation picked fields, in source order.
pub const DEVIATION: &[&str] = &[
    "source",
    "align",
    "globalAlign",
    "pairs",
    "localMaxDistance",
    "targetId",
    "targetSide",
    "targetFacingDeg",
    "mapFacingDeg",
    "targetScope",
    "scopeCount",
    "showElement",
    "range",
    "rangeAuto",
    "maxDistance",
    "maxDistanceAuto",
    "bands",
    "tolerance",
    "showHistogram",
    "showNominal",
    "showScan",
    "showMap",
    "mapOn",
    "split",
    "probes",
    "nextProbeId",
];
/// Original thickness picked fields, in source order.
pub const THICKNESS: &[&str] = &[
    "method",
    "maxThickness",
    "maxThicknessAuto",
    "coneRays",
    "coneAngleDeg",
    "normalDeviationDeg",
    "low",
    "high",
    "scaleAuto",
    "bands",
    "limit",
    "showHistogram",
    "probes",
    "nextProbeId",
];
/// Owned data fields borrowed from the real state owner. Composite cells share
/// payloads. Executable JS/prototypes/reactive subscribers are not represented.
pub type Fields = Vec<(Text, Cell)>;
/// Mutable access to the owner's four core sections, without a duplicate store.
pub struct Stores<'a> {
    /// 3D measure data.
    pub measure: &'a mut Fields,
    /// Flat data, including transient drafts/tool/version.
    pub flat: &'a mut Fields,
    /// Deviation data, including transient picking/scope version.
    pub deviation: &'a mut Fields,
    /// Thickness data.
    pub thickness: &'a mut Fields,
}
/// Core data and callback failures remain separate from original refusals.
#[derive(Debug, PartialEq, Eq)]
pub enum CoreError<E> {
    /// Host metadata budget exceeded.
    Limit,
    /// Fallible metadata reservation failed.
    Allocation,
    /// Invalid owned data shape (not an application-family validation claim).
    InvalidData,
    /// Opaque scope/participant callback error, unmodified.
    Callback(E),
}
fn key(k: &[u16], s: &str) -> bool {
    k.iter().copied().eq(s.encode_utf16())
}
/// Read an own data field, defaulting to undefined like source pick().
pub fn get<'a>(fields: &'a Fields, name: &str) -> &'a Cell {
    fields
        .iter()
        .find(|(k, _)| key(k, name))
        .map(|(_, v)| v)
        .unwrap_or(&Cell::Undefined)
}
fn set(fields: &mut Fields, k: Text, v: Cell) {
    if let Some((_, value)) = fields.iter_mut().find(|(name, _)| *name == k) {
        *value = v
    } else {
        fields.push((k, v))
    }
}
fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn check<E>(fields: &Fields, limits: Limits) -> Result<(), CoreError<E>> {
    if fields.len() > limits.fields || fields.len() > limits.cells {
        return Err(CoreError::Limit);
    }
    let mut units = 0usize;
    for (i, (k, value)) in fields.iter().enumerate() {
        units = units.checked_add(k.len()).ok_or(CoreError::Limit)?;
        if let Cell::String(value) = value {
            units = units.checked_add(value.len()).ok_or(CoreError::Limit)?;
        }
        if k.len() > limits.text || units > limits.units {
            return Err(CoreError::Limit);
        }
        if key(k, "__proto__") || fields[..i].iter().any(|(other, _)| other == k) {
            return Err(CoreError::InvalidData);
        }
    }
    Ok(())
}
fn merge<E>(target: &mut Fields, patch: &Fields, limits: Limits) -> Result<(), CoreError<E>> {
    check(target, limits)?;
    check(patch, limits)?;
    let added = patch
        .iter()
        .filter(|(k, _)| !target.iter().any(|(old, _)| old == k))
        .count();
    if target.len().saturating_add(added) > limits.fields {
        return Err(CoreError::Limit);
    }
    target
        .try_reserve(added)
        .map_err(|_| CoreError::Allocation)?;
    for (k, v) in patch {
        set(target, k.clone(), v.clone())
    }
    Ok(())
}
fn picked<E>(fields: &Fields, keys: &[&str], limits: Limits) -> Result<Cell, CoreError<E>> {
    check(fields, limits)?;
    if keys.len() > limits.fields || keys.len() > limits.cells {
        return Err(CoreError::Limit);
    }
    let mut picked = Vec::new();
    picked
        .try_reserve(keys.len())
        .map_err(|_| CoreError::Allocation)?;
    for k in keys {
        picked.push((text(k), get(fields, k).clone()))
    }
    Ok(Cell::Object(Arc::new(picked)))
}
/// Original captureCore: all four picked objects first, then scope and scan
/// callbacks. Scope and scan keep their actual shared identity.
pub fn capture<E>(
    stores: &Stores<'_>,
    limits: Limits,
    get_scope: impl FnOnce() -> Result<Cell, E>,
    get_scan: impl FnOnce() -> Result<Cell, E>,
) -> Result<Snapshot, CoreError<E>> {
    if limits.sections < 6 {
        return Err(CoreError::Limit);
    }
    let mut sections = Vec::new();
    sections.try_reserve(6).map_err(|_| CoreError::Allocation)?;
    for (name, fields, keys) in [
        ("measure", &*stores.measure, MEASURE),
        ("flat", &*stores.flat, FLAT),
        ("deviation", &*stores.deviation, DEVIATION),
        ("thickness", &*stores.thickness, THICKNESS),
    ] {
        sections.push(Section {
            key: text(name),
            value: picked(fields, keys, limits)?,
            compare: Compare::Fields,
        });
    }
    sections.push(Section {
        key: text("scope"),
        value: get_scope().map_err(CoreError::Callback)?,
        compare: Compare::Identity,
    });
    sections.push(Section {
        key: text("scan"),
        value: get_scan().map_err(CoreError::Callback)?,
        compare: Compare::Identity,
    });
    let snapshot = Snapshot { sections };
    super::snapshot_check(&snapshot, limits).map_err(|error| match error {
        super::HistoryError::Limit => CoreError::Limit,
        super::HistoryError::Allocation => CoreError::Allocation,
        super::HistoryError::Capture(e) => CoreError::Callback(e),
        _ => CoreError::InvalidData,
    })?;
    Ok(snapshot)
}
fn truthy(c: &Cell) -> bool {
    match c {
        Cell::Undefined | Cell::Null => false,
        Cell::Boolean(b) => *b,
        Cell::Number(n) => *n != 0. && !n.is_nan(),
        Cell::String(s) => !s.is_empty(),
        _ => true,
    }
}
fn object<E>(c: &Cell) -> Result<&Fields, CoreError<E>> {
    if let Cell::Object(fields) = c {
        Ok(fields)
    } else {
        Err(CoreError::InvalidData)
    }
}
fn held<E>(c: &Cell, flat: bool) -> Result<bool, CoreError<E>> {
    if matches!(c, Cell::Null) {
        return Ok(false);
    }
    let fields = object(c)?;
    if !matches!(get(fields, "editId"), Cell::Undefined) {
        return Ok(true);
    }
    let picks = if flat {
        if let Cell::Array(p) = get(fields, "picks") {
            p.len()
        } else {
            return Err(CoreError::InvalidData);
        }
    } else {
        0
    };
    if picks > 0 {
        return Ok(true);
    }
    let filled = if let Cell::Array(r) = get(fields, "refs") {
        r.iter().any(|r| !matches!(r, Cell::Null))
    } else {
        return Err(CoreError::InvalidData);
    };
    let refs = [filled];
    let d = crate::history_guards::DraftWork {
        edit_present: false,
        picks,
        selection: 0,
        references: &refs,
    };
    Ok(if flat {
        crate::history_guards::flat_draft_holds(Some(d))
    } else {
        crate::history_guards::dim_holds(Some(d))
    })
}
fn increment<E>(fields: &Fields, name: &str) -> Result<Cell, CoreError<E>> {
    if let Cell::Number(n) = get(fields, name) {
        Ok(Cell::Number(n + 1.))
    } else {
        Err(CoreError::InvalidData)
    }
}
/// Stable participant section application, in original registration order.
pub struct Participant<'a, E> {
    /// Non-prototype section key.
    pub key: &'a [u16],
    /// Apply the exact captured cell, retaining identity and opaque errors.
    pub apply: &'a mut dyn FnMut(&Cell, &Stores<'_>) -> Result<(), E>,
}
/// Original apply: merge core sections, transient flat/deviation cleanup,
/// participants, then scope callback and scope-version increment. Scan is left
/// to beforeRestore, as in the original. Callback failures retain prior effects.
pub fn apply<E>(
    stores: &mut Stores<'_>,
    patch: &Patch,
    participants: &mut [Participant<'_, E>],
    limits: Limits,
    mut set_scope: impl FnMut(Cell, &Stores<'_>) -> Result<(), E>,
) -> Result<(), CoreError<E>> {
    check(patch, limits)?;
    if participants.len() > limits.sections
        || participants.iter().any(|p| p.key.len() > limits.text)
    {
        return Err(CoreError::Limit);
    }
    if participants.iter().any(|p| super::builtin(p.key)) {
        return Err(CoreError::InvalidData);
    }
    let measure = get(patch, "measure");
    if truthy(measure) {
        merge(stores.measure, object(measure)?, limits)?;
    }
    let flat = get(patch, "flat");
    if truthy(flat) {
        let mut next = object(flat)?.clone();
        let draft = get(stores.flat, "draft");
        let draft = if held(draft, true)? {
            Cell::Null
        } else {
            draft.clone()
        };
        let dim = get(stores.flat, "dimDraft");
        let dim = if held(dim, false)? {
            Cell::Null
        } else {
            dim.clone()
        };
        let version = increment(stores.flat, "subjectVersion")?;
        set(&mut next, text("draft"), draft);
        set(&mut next, text("dimDraft"), dim);
        set(
            &mut next,
            text("tool"),
            Cell::Object(Arc::new(vec![(text("kind"), Cell::String(text("none")))])),
        );
        set(&mut next, text("subjectVersion"), version);
        merge(stores.flat, &next, limits)?;
    }
    let deviation = get(patch, "deviation");
    if truthy(deviation) {
        let mut next = object(deviation)?.clone();
        set(&mut next, text("picking"), Cell::Boolean(false));
        set(&mut next, text("pendingScan"), Cell::Null);
        set(&mut next, text("marking"), Cell::Boolean(false));
        merge(stores.deviation, &next, limits)?;
    }
    let thickness = get(patch, "thickness");
    if truthy(thickness) {
        merge(stores.thickness, object(thickness)?, limits)?;
    }
    for participant in participants {
        if let Some((_, value)) = patch.iter().find(|(k, _)| k == participant.key) {
            (participant.apply)(value, stores).map_err(CoreError::Callback)?;
        }
    }
    if let Some((_, scope)) = patch.iter().find(|(k, _)| key(k, "scope")) {
        set_scope(
            if matches!(scope, Cell::Null | Cell::Undefined) {
                Cell::Null
            } else {
                scope.clone()
            },
            stores,
        )
        .map_err(CoreError::Callback)?;
        let version = increment(stores.deviation, "scopeVersion")?;
        set(stores.deviation, text("scopeVersion"), version);
    }
    Ok(())
}
