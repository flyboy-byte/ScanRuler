// SPDX-License-Identifier: AGPL-3.0-only
//! Source sessionState readout for the supported M3 Measure/project state.
use super::*;
fn xyz(v: [f64; 3]) -> Value {
    array(v.into_iter().map(Value::Number).collect())
}
fn pick(v: &Value, keys: &str) -> Value {
    Value::Object(keys.split(' ').map(|k| (text(k), vget(v, k))).collect())
}
fn count(v: &Value) -> usize {
    v.list().map_or(0, <[Value]>::len)
}
fn rigid(v: &Value) -> Value {
    if matches!(v, Value::Null) {
        return Value::Null;
    }
    let r = items(v.get("r"))
        .iter()
        .map(|v| v.number().unwrap_or(0.))
        .collect::<Vec<_>>();
    let t = items(v.get("t"))
        .iter()
        .map(|v| v.number().unwrap_or(0.))
        .collect::<Vec<_>>();
    if r.len() != 9 || t.len() != 3 {
        return Value::Null;
    }
    let sin = port_core::hypot3([r[7] - r[5], r[2] - r[6], r[3] - r[1]]) / 2.;
    let cos = (r[0] + r[4] + r[8] - 1.) / 2.;
    object(vec![
        ("rotation", vget(v, "r")),
        ("translation", vget(v, "t")),
        (
            "rotationDeg",
            Value::Number(sin.atan2(cos) * 180. / std::f64::consts::PI),
        ),
        (
            "distanceMm",
            Value::Number(port_core::hypot3([t[0], t[1], t[2]])),
        ),
    ])
}
fn element(e: &Value) -> Value {
    let src = e.get("source");
    let settings = src.get("settings");
    let mut out = pick(e, "id name kind status visible");
    put(
        &mut out,
        "status",
        value_text(if e.get("status").is("fitting") {
            "fitting"
        } else if !matches!(e.get("fit"), Value::Undefined) {
            "done"
        } else {
            "failed"
        }),
    );
    put(
        &mut out,
        "fit",
        get(e, "fit").cloned().unwrap_or(Value::Null),
    );
    if let Some(m) = get(e, "measured") {
        put(&mut out, "measured", m.clone())
    }
    let sigma = settings.get("sigma").number().unwrap_or(3.);
    let mut source = object(vec![
        ("type", value_text("fitted")),
        ("method", vget(settings, "method")),
        ("sigmaCutoff", Value::Number(sigma)),
        (
            "cutoff",
            value_text(&if sigma == 0. {
                "all points".into()
            } else {
                format!(
                    "{} sigma cut-off",
                    port_core::js_number::number_string(sigma)
                )
            }),
        ),
    ]);
    if let Some(v) = get(settings, "knownDiameter") {
        put(&mut source, "knownDiameter", v.clone())
    }
    if let Some(v) = get(src, "selection") {
        put(&mut source, "markedPoints", Value::Number(count(v) as f64))
    } else {
        put(&mut source, "seeds", vget(src, "seeds"))
    }
    put(&mut out, "source", source);
    for k in ["assumed", "orient", "extend", "message"] {
        if let Some(v) = get(e, k) {
            put(&mut out, k, v.clone())
        }
    }
    out
}
fn iso(since: f64) -> String {
    let ms = since.max(0.) as u64;
    let mut days = ms / 86_400_000;
    let mut y = 1970u64;
    let leap = |y: u64| y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    while days >= if leap(y) { 366 } else { 365 } {
        days -= if leap(y) { 366 } else { 365 };
        y += 1
    }
    let months = [
        31,
        if leap(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while days >= months[month] {
        days -= months[month];
        month += 1
    }
    format!(
        "{y:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        month + 1,
        days + 1,
        ms / 3_600_000 % 24,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}
pub(super) fn state(state: &State, runtime: &Runtime) -> Result<Value, Fault> {
    let d = &state.document;
    let app = d.store(StoreId::App);
    let s = store_fields(
        d,
        StoreId::App,
        "fileName appliedAlignment elements dimensions sections projectName draft dimDraft alignDraft sectionDraft busy",
    );
    let dev = cell_value(&C::Object(d.store(StoreId::Deviation).snapshot()));
    let t = cell_value(&C::Object(d.store(StoreId::Thickness).snapshot()));
    let saved = state.saved_project.borrow();
    let flat = saved
        .as_ref()
        .map(|p| vget(&p.manifest, "flat"))
        .map_or_else(flat_blank, Ok)?;
    let scan = if matches!(s.get("fileName"), Value::Null)
        || s.get("fileName").text().is_some_and(<[u16]>::is_empty)
    {
        Value::Null
    } else {
        let mut scan = state.summary()?;
        put(
            &mut scan,
            "appliedAlignment",
            rigid(s.get("appliedAlignment")),
        );
        scan
    };
    let range = dev.get("range").number().unwrap_or(0.);
    let colormap = object(vec![
        ("id", value_text("jet")),
        (
            "low",
            object(vec![
                ("mm", Value::Number(-range)),
                ("rgb", xyz([0., 0., 255.])),
            ]),
        ),
        (
            "zero",
            object(vec![
                ("mm", Value::Number(0.)),
                ("rgb", xyz([0., 200., 0.])),
            ]),
        ),
        (
            "high",
            object(vec![
                ("mm", Value::Number(range)),
                ("rgb", xyz([255., 0., 0.])),
            ]),
        ),
        (
            "under",
            object(vec![
                (
                    "mm",
                    value_text(&format!(
                        "< {}",
                        port_core::js_number::number_string(-range)
                    )),
                ),
                ("rgb", xyz([0., 0., 110.])),
            ]),
        ),
        (
            "over",
            object(vec![
                (
                    "mm",
                    value_text(&format!("> {}", port_core::js_number::number_string(range))),
                ),
                ("rgb", xyz([130., 0., 0.])),
            ]),
        ),
        ("unmeasured", object(vec![("rgb", xyz([126., 131., 138.]))])),
    ]);
    let mut ds = pick(&dev, "range maxDistance bands tolerance");
    put(&mut ds, "facingDeg", vget(&dev, "mapFacingDeg"));
    put(&mut ds, "colormap", colormap);
    put(&mut ds, "show", vget(&dev, "showMap"));
    let mut deviation = object(vec![
        ("source", vget(&dev, "source")),
        ("reference", Value::Null),
        ("align", Value::Null),
        ("alignStatus", vget(&dev, "alignStatus")),
        (
            "map",
            vget(
                &dev,
                if dev.get("source").is("element") {
                    "elementStatus"
                } else {
                    "mapStatus"
                },
            ),
        ),
        ("stats", Value::Null),
    ]);
    if dev.get("source").is("reference") {
        put(&mut deviation, "on", vget(&dev, "mapOn"));
        put(
            &mut deviation,
            "onReference",
            object(vec![("map", vget(&dev, "referenceStatus"))]),
        )
    }
    let target = if matches!(dev.get("targetId"), Value::Null) {
        Value::Null
    } else {
        let mut v = object(vec![
            ("element", vget(&dev, "targetId")),
            (
                "side",
                value_text(if dev.get("targetSide").number() == Some(1.) {
                    "outward"
                } else {
                    "inward"
                }),
            ),
            ("facingDeg", vget(&dev, "targetFacingDeg")),
            ("scope", vget(&dev, "targetScope")),
        ]);
        if dev.get("targetScope").is("marked") {
            put(&mut v, "markedPoints", vget(&dev, "scopeCount"))
        }
        v
    };
    put(&mut deviation, "target", target);
    put(&mut deviation, "settings", ds);
    let thickness = object(vec![
        ("status", vget(&t, "status")),
        ("stats", Value::Null),
        (
            "settings",
            pick(
                &t,
                "method maxThickness coneRays coneAngleDeg normalDeviationDeg low high bands limit",
            ),
        ),
    ]);
    let flat_read = object(vec![
        ("subject", object(vec![("kind", value_text("image"))])),
        ("image", Value::Null),
        (
            "units",
            value_text(if matches!(flat.get("pxPerMm"), Value::Null) {
                "px"
            } else {
                "mm"
            }),
        ),
        ("sheet", Value::Null),
        (
            "calibration",
            object(vec![
                ("source", vget(&flat, "calSource")),
                ("pxPerMm", vget(&flat, "pxPerMm")),
            ]),
        ),
        ("alignment", Value::Null),
        (
            "edges",
            object(vec![
                ("status", value_text("idle")),
                ("chains", Value::Number(0.)),
                ("sensitivity", vget(&flat, "edgeSensitivity")),
                ("shown", vget(&flat, "showEdges")),
                ("snap", vget(&flat, "snapToEdge")),
            ]),
        ),
        ("elements", array(vec![])),
        ("dimensions", array(vec![])),
        ("notes", array(vec![])),
        ("counts", array(vec![])),
    ]);
    let elements = items(s.get("elements"));
    let fitted = elements.iter().filter(|e| get(e, "fit").is_some()).count();
    let hint = if matches!(scan, Value::Null) {
        object(vec![
            ("target", value_text("open-scan")),
            ("text", value_text("Open the scan you want to measure")),
        ])
    } else if !matches!(s.get("draft"), Value::Null) {
        object(vec![
            ("target", value_text("create-element")),
            (
                "text",
                value_text("Create the element once the fit looks right"),
            ),
        ])
    } else {
        object(vec![
            (
                "target",
                value_text(if fitted < 2 {
                    "kindrow"
                } else {
                    "new-dimension"
                }),
            ),
            (
                "text",
                value_text(match fitted {
                    0 => "Pick a shape to fit on the part — then click that feature on the scan",
                    1 => "One element measured — fit a second one to measure between them",
                    _ => "Two elements measured — now measure between them",
                }),
            ),
        ])
    };
    let h = runtime.history();
    let label = |entries: &[crate::history_record::Entry]| {
        entries
            .last()
            .map_or(Value::Null, |e| Value::String(e.label.clone()))
    };
    let mut out = object(vec![
        (
            "app",
            object(vec![
                ("name", value_text("ScanRuler")),
                ("version", value_text(port_core::REFERENCE_APP_VERSION)),
                ("plugins", array(vec![])),
                ("viewport", Value::Boolean(false)),
            ]),
        ),
        (
            "project",
            object(vec![
                ("name", cell_value(&app.get("projectName"))),
                ("unsaved", Value::Null),
                ("holds", holds(d)),
            ]),
        ),
        ("workspace", value_text("measure")),
        ("busy", Value::Null),
        ("running", Value::Null),
        ("scan", scan),
        ("elements", array(elements.iter().map(element).collect())),
        ("dimensions", array(vec![])),
        ("sections", array(vec![])),
        (
            "open",
            object(vec![
                ("element", Value::Null),
                ("dimension", Value::Null),
                ("alignment", Value::Boolean(false)),
                ("section", Value::Boolean(false)),
            ]),
        ),
        ("deviation", deviation),
        ("thickness", thickness),
        ("flat", flat_read),
        ("hint", hint),
        (
            "guide",
            value_text(
                "Before a task’s first step, read its guide: guide.read lists them — how to go about each task with this build.",
            ),
        ),
        (
            "history",
            object(vec![
                ("undo", label(&h.past)),
                ("redo", label(&h.future)),
                ("blocked", Value::Boolean(h.restoring)),
            ]),
        ),
        ("view", Value::Null),
        ("ui", object(vec![("units", value_text("mm"))])),
        ("plugins", object(vec![])),
    ]);
    if let Some(r) = runtime.activity().current().running.as_ref() {
        put(&mut out, "running", Value::String(r.name.to_vec()));
        put(&mut out, "runningSince", value_text(&iso(r.since)));
        put(&mut out, "runningMs", Value::Number(raw_now()? - r.since));
    }
    Ok(out)
}
