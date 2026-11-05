// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen manifest.ts / validation.ts checks, in source order.
use crate::schema_value::Value as V;
fn invalid<T>(p: &str) -> Result<T, String> {
    Err(format!("Malformed project: {p}."))
}
fn object<'a>(v: &'a V, p: &str) -> Result<&'a [(Vec<u16>, V)], String> {
    if let V::Object(f) = v {
        Ok(f)
    } else {
        invalid(p)
    }
}
fn array<'a>(v: &'a V, p: &str) -> Result<&'a [V], String> {
    v.list().ok_or_else(|| format!("Malformed project: {p}."))
}
fn fields(v: &V, keys: &str, t: &str, p: &str) -> Result<(), String> {
    for k in keys.split(' ') {
        let x = v.get(k);
        let ok = match t {
            "number" => matches!(x,V::Number(n) if n.is_finite()),
            "string" => matches!(x, V::String(_)),
            _ => matches!(x, V::Boolean(_)),
        };
        if !ok {
            return invalid(&format!("{p}.{k}"));
        }
    }
    Ok(())
}
fn indices(v: &V, p: &str) -> Result<(), String> {
    for x in array(v, p)? {
        if !matches!(x,V::Number(n) if n.is_finite()&&n.fract()==0.&&*n>=0.&&*n<=9_007_199_254_740_991.)
        {
            return invalid(p);
        }
    }
    Ok(())
}
fn vector(v: &V, n: usize, p: &str) -> Result<(), String> {
    let a = array(v, p)?;
    if a.len() != n || a.iter().any(|x| !matches!(x,V::Number(n) if n.is_finite())) {
        return invalid(p);
    }
    Ok(())
}
fn entries(v: &V, p: &str, mut visit: impl FnMut(&V) -> Result<(), String>) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for e in array(v, p)? {
        object(e, p)?;
        let Some(n) = e.get("id").number() else {
            return invalid(&format!("{p} IDs"));
        };
        if !n.is_finite()
            || n < 0.
            || n.fract() != 0.
            || n > 9_007_199_254_740_991.
            || ids.contains(&(n as u64))
        {
            return invalid(&format!("{p} IDs"));
        }
        ids.insert(n as u64);
        visit(e)?
    }
    Ok(())
}
fn file(v: &V, p: &str) -> Result<(), String> {
    if matches!(v, V::Null) {
        return Ok(());
    }
    object(v, p)?;
    fields(v, "fileName member", "string", p)?;
    let name = v.get("fileName").text().unwrap_or_default();
    let member = v.get("member").text().unwrap_or_default();
    if name.is_empty()
        || member.is_empty()
        || member
            .iter()
            .any(|c| !matches!(c,45|46|32|48..=57|65..=90|95|97..=122))
        || v.get("member").is("project.json")
    {
        return invalid(&format!("{p}.member"));
    }
    if !matches!(v.get("units"), V::Undefined)
        && !["mm", "cm", "m", "in"].iter().any(|s| v.get("units").is(s))
    {
        return invalid(&format!("{p}.units"));
    }
    Ok(())
}
fn rigid(v: &V, p: &str) -> Result<(), String> {
    if matches!(v, V::Null) {
        return Ok(());
    }
    object(v, p)?;
    vector(v.get("r"), 9, p)?;
    vector(v.get("t"), 3, p)
}
fn sheet(s: &V, p: &str) -> Result<(), String> {
    object(s, p)?;
    fields(s, "nextId nextDimId nextCountId", "number", p)?;
    fields(s, "calSource", "string", p)?;
    object(s.get("nameCounts"), &format!("{p}.nameCounts"))?;
    object(s.get("dimCounts"), &format!("{p}.dimCounts"))?;
    entries(s.get("elements"), &format!("{p}.elements"), |e| {
        fields(e, "kind name color", "string", p)?;
        fields(e, "visible", "boolean", p)?;
        let fit = e.get("fit");
        if !matches!(fit, V::Null) {
            object(fit, &format!("{p}.fit"))?;
            if fit.get("kind") != e.get("kind") {
                return invalid(&format!("{p}.fit.kind"));
            }
            fields(fit, "sigma usedPoints", "number", &format!("{p}.fit"))?;
            if fit.get("kind").is("point") {
                vector(fit.get("at"), 2, &format!("{p}.fit.at"))?
            } else if fit.get("kind").is("spline") {
                for k in ["points", "tangents"] {
                    for a in array(fit.get(k), &format!("{p}.fit.{k}"))? {
                        vector(a, 2, &format!("{p}.fit.{k}"))?
                    }
                }
            } else {
                vector(fit.get("center"), 2, &format!("{p}.fit.center"))?
            }
        }
        let src = e.get("source");
        object(src, &format!("{p}.source"))?;
        fields(src, "method", "string", &format!("{p}.source"))?;
        if src.get("type").is("picks") {
            for a in array(src.get("picks"), &format!("{p}.picks"))? {
                vector(a, 2, &format!("{p}.picks"))?
            }
        } else if src.get("type").is("construct") {
            indices(src.get("refs"), &format!("{p}.refs"))?
        } else {
            return invalid(&format!("{p}.source.type"));
        }
        Ok(())
    })?;
    entries(s.get("dimensions"), &format!("{p}.dimensions"), |e| {
        fields(e, "type name", "string", p)?;
        indices(e.get("refs"), &format!("{p}.refs"))
    })?;
    entries(s.get("counts"), &format!("{p}.counts"), |e| {
        fields(e, "name color", "string", p)?;
        array(e.get("picks"), &format!("{p}.picks"))?;
        Ok(())
    })?;
    if !matches!(s.get("notes"), V::Undefined) {
        entries(s.get("notes"), &format!("{p}.notes"), |e| {
            fields(e, "text", "string", p)?;
            array(e.get("at"), &format!("{p}.at"))?;
            Ok(())
        })?
    }
    if !matches!(s.get("pxPerMm"), V::Null) {
        let a = s.get("pxPerMm");
        object(a, &format!("{p}.pxPerMm"))?;
        fields(a, "x y", "number", p)?;
        if a.get("x").number().unwrap_or(0.) <= 0. || a.get("y").number().unwrap_or(0.) <= 0. {
            return invalid(&format!("{p}.pxPerMm"));
        }
    }
    Ok(())
}
pub(super) fn validate(m: &V) -> Result<(), String> {
    if !matches!(m, V::Object(_))
        || !m.get("app").is("ScanRuler")
        || !matches!(m.get("schemaVersion"), V::Number(_))
    {
        return Err("Not a ScanRuler project file.".into());
    }
    let n = m.get("schemaVersion").number().unwrap_or(0.);
    if !n.is_finite() || n.fract() != 0. || n < 1. {
        return Err("Unsupported project schema version.".into());
    }
    if n > 1. {
        return Err("This project was saved by a newer version of ScanRuler — please reload the app to open it.".into());
    }
    if !matches!(m.get("scan"), V::Null) && !matches!(m.get("scan").get("fileName"), V::String(_)) {
        return invalid("scan entry");
    }
    if ["deviation", "thickness", "flat"]
        .iter()
        .any(|k| matches!(m.get(k), V::Undefined | V::Null | V::Boolean(false)))
    {
        return invalid("missing parts");
    }
    object(m, "root")?;
    fn inspect(v: &V, d: usize) -> Result<(), String> {
        if d > 64 {
            return invalid("nesting depth");
        }
        match v {
            V::Number(n) if !n.is_finite() => return invalid("non-finite number"),
            V::Array(a) => {
                for v in a {
                    inspect(v, d + 1)?
                }
            }
            V::Object(f) => {
                for (k, v) in f {
                    if ["__proto__", "constructor", "prototype"]
                        .iter()
                        .any(|s| k.iter().copied().eq(s.encode_utf16()))
                    {
                        return invalid("reserved property");
                    }
                    inspect(v, d + 1)?
                }
            }
            _ => {}
        }
        Ok(())
    }
    inspect(m, 0)?;
    fields(m, "appVersion workspace", "string", "root")?;
    if !matches!(m.get("name"), V::Undefined) {
        fields(m, "name", "string", "root")?
    }
    let s = m.get("scan");
    if !matches!(s, V::Null) {
        object(s, "scan")?;
        file(s, "scan")?;
        if !matches!(s.get("companions"), V::Undefined) {
            for c in array(s.get("companions"), "scan.companions")? {
                file(c, "scan.companions")?
            }
        }
        fields(s, "nextId nextNumber nextDimensionId", "number", "scan")?;
        fields(s, "selectMode", "string", "scan")?;
        fields(s, "showOverlays", "boolean", "scan")?;
        object(s.get("nextOfKind"), "scan.nextOfKind")?;
        object(s.get("nextOfDimGroup"), "scan.nextOfDimGroup")?;
        object(s.get("settings"), "scan.settings")?;
        fields(s.get("settings"), "method", "string", "scan.settings")?;
        fields(s.get("settings"), "sigma", "number", "scan.settings")?;
        rigid(s.get("appliedAlignment"), "scan.appliedAlignment")?;
        entries(s.get("elements"), "scan.elements", |e| {
            fields(e, "kind name color status", "string", "element")?;
            fields(e, "visible", "boolean", "element")?;
            let src = e.get("source");
            object(src, "element.source")?;
            if !["picked", "fitted", "constructed"]
                .iter()
                .any(|s| src.get("type").is(s))
            {
                return invalid("element.source.type");
            }
            if src.get("type").is("fitted") {
                indices(src.get("seeds"), "element.seeds")?
            }
            if src.get("type").is("constructed") {
                indices(src.get("refs"), "element.refs")?;
                array(src.get("params"), "element.params")?;
            }
            if !matches!(src.get("selection"), V::Undefined) {
                indices(src.get("selection"), "element.selection")?
            }
            if ![
                "point", "line", "plane", "sphere", "cylinder", "cone", "circle", "torus",
            ]
            .iter()
            .any(|s| e.get("kind").is(s))
            {
                return invalid("element.kind");
            }
            if !matches!(e.get("fit"), V::Undefined) {
                let f = e.get("fit");
                object(f, "element.fit")?;
                if f.get("kind") != e.get("kind") {
                    return invalid("element.fit.kind");
                }
                fields(f, "sigma usedPoints regionSize", "number", "element.fit")?;
                vector(f.get("center"), 3, "element.fit.center")?;
                for k in ["axis", "normal", "dir", "basisU", "basisV"] {
                    if !matches!(f.get(k), V::Undefined) {
                        vector(f.get(k), 3, &format!("element.fit.{k}"))?
                    }
                }
            }
            Ok(())
        })?;
        entries(s.get("dimensions"), "scan.dimensions", |e| {
            fields(e, "type name", "string", "dimension")?;
            indices(e.get("refs"), "dimension.refs")
        })?;
        if !matches!(s.get("sections"), V::Undefined) {
            entries(s.get("sections"), "scan.sections", |e| {
                object(e.get("frame"), "section.frame")?;
                Ok(())
            })?
        }
    }
    let d = m.get("deviation");
    object(d, "deviation")?;
    fields(
        d,
        "localMaxDistance range maxDistance tolerance nextProbeId targetSide",
        "number",
        "deviation",
    )?;
    fields(
        d,
        "rangeAuto maxDistanceAuto showElement showHistogram showNominal showScan showMap split",
        "boolean",
        "deviation",
    )?;
    if !["reference", "element"]
        .iter()
        .any(|s| d.get("source").is(s))
        || !["all", "marked"].iter().any(|s| d.get("targetScope").is(s))
    {
        return invalid("deviation source or scope");
    }
    if !matches!(d.get("mapFacingDeg"), V::Undefined | V::Null | V::Number(_)) {
        return invalid("deviation.mapFacingDeg");
    }
    file(d.get("reference"), "deviation.reference")?;
    array(d.get("pairs"), "deviation.pairs")?;
    entries(d.get("probes"), "deviation.probes", |_| Ok(()))?;
    if !matches!(d.get("scope"), V::Null) {
        indices(d.get("scope"), "deviation.scope")?
    }
    for k in ["align", "globalAlign"] {
        if !matches!(d.get(k), V::Null) {
            object(d.get(k), &format!("deviation.{k}"))?;
            rigid(
                d.get(k).get("transform"),
                &format!("deviation.{k}.transform"),
            )?
        }
    }
    let t = m.get("thickness");
    object(t, "thickness")?;
    fields(
        t,
        "maxThickness coneRays coneAngleDeg low high limit nextProbeId",
        "number",
        "thickness",
    )?;
    fields(
        t,
        "measured maxThicknessAuto scaleAuto showHistogram",
        "boolean",
        "thickness",
    )?;
    fields(t, "method", "string", "thickness")?;
    entries(t.get("probes"), "thickness.probes", |_| Ok(()))?;
    let f = m.get("flat");
    object(f, "flat")?;
    file(f.get("image"), "flat.image")?;
    sheet(f, "flat")?;
    fields(f, "edgeSensitivity", "number", "flat")?;
    fields(
        f,
        "splitAxes showEdges snapToEdge showGrid",
        "boolean",
        "flat",
    )?;
    if !matches!(f.get("sheets"), V::Undefined) {
        for (k, s) in object(f.get("sheets"), "flat.sheets")? {
            sheet(s, &format!("flat.sheets.{}", String::from_utf16_lossy(k)))?
        }
    }
    Ok(())
}
