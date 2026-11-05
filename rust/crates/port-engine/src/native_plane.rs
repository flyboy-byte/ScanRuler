// SPDX-License-Identifier: AGPL-3.0-only
//! Real no-viewport marked-plane continuation. Unsupported family/finishing
//! paths are explicit; the public slice remains draft until all continuations land.
use super::*;
use port_core::marked_cylinder::{self, MarkedCylinder};
use port_core::marked_plane::{self, MarkedPlane, MarkedPlaneError};
#[path = "native_plane_definition.rs"]
mod definition;
#[path = "native_plane_finish.rs"]
mod finishing;
pub(super) fn cell(v: Value) -> C {
    match v {
        Value::Undefined => C::Undefined,
        Value::Null => C::Null,
        Value::Boolean(v) => C::Boolean(v),
        Value::Number(v) => C::Number(v),
        Value::String(v) => C::String(v),
        Value::Bytes(v) => C::Bytes(Arc::new(v)),
        Value::Array(v) => C::Array(Arc::new(v.into_iter().map(cell).collect())),
        Value::Object(v) => C::Object(Arc::new(v.into_iter().map(|(k, v)| (k, cell(v))).collect())),
    }
}
fn obj(v: Vec<(&str, C)>) -> C {
    C::Object(Arc::new(v.into_iter().map(|(k, v)| (text(k), v)).collect()))
}
fn st(v: &str) -> C {
    C::String(text(v))
}
fn num(v: usize) -> C {
    C::Number(v as f64)
}
fn arr(v: Vec<C>) -> C {
    C::Array(Arc::new(v))
}
fn field(v: &C, k: &str) -> C {
    if let C::Object(f) = v {
        f.iter()
            .find(|(key, _)| *key == text(k))
            .map_or(C::Undefined, |(_, v)| v.clone())
    } else {
        C::Undefined
    }
}
pub(super) fn put(v: &C, key: &str, value: C) -> C {
    let mut fields = match v {
        C::Object(f) => (**f).clone(),
        _ => vec![],
    };
    if let Some((_, v)) = fields.iter_mut().find(|(k, _)| *k == text(key)) {
        *v = value
    } else {
        fields.push((text(key), value))
    }
    C::Object(Arc::new(fields))
}
fn count(v: &C) -> usize {
    match v {
        C::Array(v) => v.len(),
        C::Words(v) => v.len(),
        _ => 0,
    }
}
fn truthy(v: &C) -> bool {
    match v {
        C::Null | C::Undefined => false,
        C::Boolean(b) => *b,
        C::Number(n) => *n != 0. && !n.is_nan(),
        C::String(s) => !s.is_empty(),
        _ => true,
    }
}
pub(super) fn measure_free(doc: &Session) -> Result<(), Fault> {
    let app = doc.store(StoreId::App);
    let draft = app.get("draft");
    let mut open = vec![];
    let refs = field(&draft, "refs");
    let refs = match refs {
        C::Array(v) => v.iter().map(|v| !matches!(v, C::Null)).collect::<Vec<_>>(),
        _ => vec![],
    };
    if crate::history_guards::draft_holds(if matches!(draft, C::Null) {
        None
    } else {
        Some(crate::history_guards::DraftWork {
            edit_present: !matches!(field(&draft, "editId"), C::Undefined),
            picks: count(&field(&draft, "picks")),
            selection: count(&field(&draft, "selection")),
            references: &refs,
        })
    }) {
        open.push(if !matches!(field(&draft, "editId"), C::Undefined) {
            "an element being edited".into()
        } else {
            format!(
                "a {} being made",
                match field(&draft, "kind") {
                    C::String(s) => String::from_utf16_lossy(&s),
                    _ => "undefined".into(),
                }
            )
        });
    }
    let dim = app.get("dimDraft");
    let refs = field(&dim, "refs");
    let refs = match refs {
        C::Array(v) => v.iter().map(|v| !matches!(v, C::Null)).collect::<Vec<_>>(),
        _ => vec![],
    };
    if crate::history_guards::dim_holds(if matches!(dim, C::Null) {
        None
    } else {
        Some(crate::history_guards::DraftWork {
            edit_present: !matches!(field(&dim, "editId"), C::Undefined),
            references: &refs,
            ..Default::default()
        })
    }) {
        open.push("a dimension being set up".into())
    }
    if truthy(&app.get("alignDraft")) {
        open.push("the alignment being set up".into())
    }
    if truthy(&app.get("sectionDraft")) {
        open.push("a section being made".into())
    }
    if open.is_empty() {
        Ok(())
    } else {
        Err(command_error(
            CommandErrorCode::InvalidState,
            &format!(
                "The 3D Measure panel has {} — finish or cancel it there first, or ask the person at the screen to.",
                open.join(" and ")
            ),
        ))
    }
}
/// The kinds the native marked fit carries so far.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Plane,
    Cylinder,
}
impl Kind {
    fn of(name: &str) -> Option<Self> {
        match name {
            "plane" => Some(Self::Plane),
            "cylinder" => Some(Self::Cylinder),
            _ => None,
        }
    }
    fn id(self) -> &'static str {
        match self {
            Self::Plane => "plane",
            Self::Cylinder => "cylinder",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Plane => "Plane",
            Self::Cylinder => "Cylinder",
        }
    }
    /// construct.ts creation-method hint for the kind's `fit` method.
    fn hint(self) -> &'static str {
        match self {
            Self::Plane => "Click a point on the flat surface in the 3D view.",
            Self::Cylinder => "Click a point on the cylindrical surface in the 3D view.",
        }
    }
}
/// A marked fit of either kind.
pub(super) enum Fitted {
    Plane(MarkedPlane),
    Cylinder(MarkedCylinder),
}
impl Fitted {
    fn cell(&self) -> C {
        match self {
            Self::Plane(f) => fit_cell(f),
            Self::Cylinder(f) => cylinder_cell(f),
        }
    }
    fn region(&self) -> Vec<u32> {
        match self {
            Self::Plane(f) => f.region.clone(),
            Self::Cylinder(f) => f.region.to_vec(),
        }
    }
    fn region_size(&self) -> usize {
        match self {
            Self::Plane(f) => f.region_size,
            Self::Cylinder(f) => f.region_size,
        }
    }
}
/// fitCylinderOnSelection's CylinderFit without its region, in source key order.
pub(super) fn cylinder_cell(f: &MarkedCylinder) -> C {
    let xyz = |v: [f64; 3]| arr(v.into_iter().map(C::Number).collect());
    obj(vec![
        ("kind", st("cylinder")),
        ("center", xyz(f.center)),
        ("axis", xyz(f.axis)),
        ("radius", C::Number(f.radius)),
        ("length", C::Number(f.length)),
        ("coverage", C::Number(f.coverage)),
        ("sigma", C::Number(f.sigma)),
        ("usedPoints", num(f.used_points)),
        ("regionSize", num(f.region_size)),
        ("formError", C::Number(f.form_error)),
        ("residualMin", C::Number(f.residual_min)),
        ("residualMax", C::Number(f.residual_max)),
    ])
}
/// `toLocaleString('en-US')` for a count.
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',')
        }
        out.push(c)
    }
    out
}
fn draft(kind: Kind, settings: C) -> C {
    obj(vec![
        ("kind", st(kind.id())),
        ("method", st("fit")),
        ("picks", arr(vec![])),
        ("pickPoints", arr(vec![])),
        ("settings", settings),
        ("refs", arr(vec![])),
        ("params", arr(vec![])),
        ("status", st("empty")),
        ("pickSlot", C::Null),
    ])
}
pub(super) fn fit_cell(f: &MarkedPlane) -> C {
    let xyz = |v: [f64; 3]| arr(v.into_iter().map(C::Number).collect());
    obj(vec![
        ("kind", st("plane")),
        ("center", xyz(f.center)),
        ("normal", xyz(f.normal)),
        ("basisU", xyz(f.basis_u)),
        ("basisV", xyz(f.basis_v)),
        ("extentU", C::Number(f.extent_u)),
        ("extentV", C::Number(f.extent_v)),
        ("sigma", C::Number(f.sigma)),
        ("usedPoints", num(f.used_points)),
        ("regionSize", num(f.region_size)),
        ("formError", C::Number(f.form_error)),
        ("residualMin", C::Number(f.residual_min)),
        ("residualMax", C::Number(f.residual_max)),
    ])
}
fn close(doc: &Session) -> Result<(), Fault> {
    let app = doc.store(StoreId::App);
    let d = app.get("draft");
    if matches!(d, C::Null) {
        return Ok(());
    }
    let n = count(&field(&d, "selection"));
    let noun = match field(&d, "kind") {
        C::String(k) => String::from_utf16_lossy(&k),
        _ => "plane".into(),
    };
    let kept = if n > 0 { d } else { C::Null };
    patch(
        doc,
        vec![
            ("draft", C::Null),
            ("discarded", kept),
            (
                "statusText",
                st(&if n > 0 {
                    format!(
                        "The {noun} was discarded with {} marked points on it — Restore in the panel brings it back.",
                        grouped(n)
                    )
                } else {
                    String::new()
                }),
            ),
        ],
    )?;
    doc.store(StoreId::Mark)
        .patch(vec![(text("count"), num(0))])
}
pub(super) fn register(runtime: &Runtime, state: &Rc<State>) -> Result<(), Fault> {
    let weak = Rc::downgrade(state);
    runtime.register(
        &text("element"),
        definition::definition(),
        Callbacks {
            label: Some(Rc::new(command_label)),
            discards: None,
            run: Rc::new(move |input, ctx| {
                let state = weak
                    .upgrade()
                    .ok_or_else(|| Fault::host("closed native session"))?;
                start(state, input, ctx)
            }),
        },
    )?;
    Ok(())
}
fn command_label(input: &Value) -> Result<Option<Text>, Fault> {
    let kind = match get(input, "kind") {
        Some(Value::String(s)) => String::from_utf16_lossy(s),
        _ => "undefined".into(),
    };
    Ok(Some(text(&format!("fit {kind} to a marked surface"))))
}
fn start(state: Rc<State>, input: &Value, ctx: CommandContext) -> Result<(), Fault> {
    let app = state.document.store(StoreId::App);
    if !truthy(&app.get("fileName")) || matches!(app.get("vertexCount"), C::Number(0.)) {
        return Err(command_error(
            CommandErrorCode::NoScan,
            "No scan is open — open one with scan.open first.",
        ));
    }
    measure_free(&state.document)?;
    let kind = match get(input, "kind") {
        Some(Value::String(s)) => String::from_utf16_lossy(s),
        _ => return Err(Fault::host("missing admitted kind")),
    };
    if get(input, "knownDiameter").is_some() && kind != "sphere" {
        return Err(command_error(
            CommandErrorCode::InvalidInput,
            &format!("knownDiameter is for a sphere, not a {kind}."),
        ));
    }
    let supplied = [
        get(input, "vertices").is_some(),
        matches!(get(input, "marked"), Some(Value::Boolean(true))),
        get(input, "box").is_some(),
    ]
    .into_iter()
    .filter(|v| *v)
    .count();
    if supplied != 1 {
        return Err(command_error(
            CommandErrorCode::InvalidInput,
            "Give vertices, marked: true or box — one of them.",
        ));
    }
    if get(input, "normal").is_some() && get(input, "box").is_none() {
        return Err(command_error(
            CommandErrorCode::InvalidInput,
            "normal goes with box.",
        ));
    }
    let graph = state
        .worker
        .borrow()
        .graph()
        .ok_or_else(|| Fault::host("missing owned scan graph"))?;
    let Some(kind) = Kind::of(&kind) else {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native sphere, cone and torus marked fits are not implemented.",
        ));
    };
    if let Some(bounds) = get(input, "box") {
        let work = selection::box_query(&state, graph.clone(), bounds, get(input, "normal"))?;
        let input = input.clone();
        let facing = get(&input, "normal").is_some();
        ctx.spawn(async move {
            let vertices = work.await.map_err(KernelError::fault)?;
            if vertices.len() < 3 {
                return Err(command_error(
                    CommandErrorCode::InvalidInput,
                    &format!(
                        "Only {} scan points lie in the box{} — a surface needs more.",
                        vertices.len(),
                        if facing { " facing that way" } else { "" }
                    ),
                ));
            }
            let (settings, scan, work) = begin(&state, kind, &input, graph, vertices)?;
            finish(state, input, settings, scan, work).await
        })?;
        return Ok(());
    }
    let vertices = match get(input, "vertices") {
        Some(Value::Array(a)) => {
            if a.len() > 1_000_000 {
                return Err(Fault::host("native marking budget"));
            }
            let mut v = Vec::new();
            v.try_reserve_exact(a.len()).map_err(Fault::host)?;
            for n in a {
                let Value::Number(n) = n else {
                    return Err(Fault::host("non-numeric admitted index"));
                };
                if *n >= graph.vertex_count as f64 {
                    return Err(command_error(
                        CommandErrorCode::InvalidInput,
                        &format!(
                            "The scan has no vertex {} — it has {}.",
                            port_core::js_number::number_string(*n),
                            graph.vertex_count
                        ),
                    ));
                }
                v.push(*n as u32)
            }
            v
        }
        _ => selection::require_marked(&state)?,
    };
    let (settings, scan, work) = begin(&state, kind, input, graph, vertices)?;
    ctx.spawn(finish(state, input.clone(), settings, scan, work))?;
    Ok(())
}
fn begin(
    state: &Rc<State>,
    kind: Kind,
    input: &Value,
    graph: Arc<MeshGraph>,
    vertices: Vec<u32>,
) -> Result<(C, u64, KernelWork<Fitted>), Fault> {
    let app = state.document.store(StoreId::App);
    let mut settings = app.get("settings");
    if let Some(v) = get(input, "sigma") {
        settings = put(&settings, "sigma", cell(v.clone()));
        let remembered = match &settings {
            C::Object(f) => C::Object(Arc::new(
                f.iter()
                    .filter(|(k, _)| *k != text("knownDiameter"))
                    .cloned()
                    .collect(),
            )),
            _ => return Err(Fault::host("settings owner")),
        };
        patch(&state.document, vec![("settings", remembered)])?
    }

    let sigma = match field(&settings, "sigma") {
        C::Number(n) => n,
        _ => 3.,
    };
    let mut d = draft(kind, settings.clone());
    d = put(&d, "selection", C::Words(Arc::new(vertices.clone())));
    d = put(&d, "fit", C::Undefined);
    d = put(&d, "message", C::Undefined);
    d = put(&d, "status", st("fitting"));
    state
        .document
        .store(StoreId::Mark)
        .patch(vec![(text("count"), num(0))])?;
    patch(
        &state.document,
        vec![
            ("draft", d),
            ("discarded", C::Null),
            ("dimDraft", C::Null),
            ("alignDraft", C::Null),
            ("sectionDraft", C::Null),
            ("errorText", C::Null),
            ("statusText", st(kind.hint())),
        ],
    )?;

    state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    let scan = state.worker.borrow().versions().scan;
    let work = start_job("scanruler-plane", move || match kind {
        Kind::Plane => marked_plane::fit_plane_on_selection(
            &graph.positions,
            &graph.normals,
            &vertices,
            sigma,
            1_000_000,
        )
        .map(Fitted::Plane)
        .map_err(|e| match e {
            MarkedPlaneError::Source(s) => KernelError::Source(s),
            _ => KernelError::Host(format!("{e:?}")),
        }),
        Kind::Cylinder => marked_cylinder::fit_cylinder_on_selection(
            &graph.positions,
            &graph.normals,
            vertices.into(),
            sigma,
            None,
            1_000_000,
        )
        .map(Fitted::Cylinder)
        .map_err(|e| match e {
            marked_cylinder::Error::Source(s) => KernelError::Source(s),
            _ => KernelError::Host(format!("{e:?}")),
        }),
    });
    let work = match work {
        Ok(work) => work,
        Err(error) => {
            let d = app.get("draft");
            let d = put(
                &put(&d, "status", st("failed")),
                "message",
                st("The native geometry worker could not be started."),
            );
            patch(&state.document, vec![("draft", d)])?;
            close(&state.document)?;
            return Err(error);
        }
    };

    Ok((settings, scan, work))
}

async fn finish(
    state: Rc<State>,
    input: Value,
    settings: C,
    scan: u64,
    work: KernelWork<Fitted>,
) -> Result<Value, Fault> {
    let boxed = get(&input, "box").is_some();
    let mut result = apply(state.clone(), input, settings, scan, work).await;
    if boxed && let Ok(Value::Object(fields)) = &mut result {
        let points = fields
            .iter()
            .find(|(k, _)| *k == text("element"))
            .and_then(|(_, element)| get(element, "source"))
            .and_then(|source| get(source, "markedPoints"))
            .cloned()
            .ok_or_else(|| Fault::host("missing fitted selection count"))?;
        fields.push((text("points"), points));
    }
    close(&state.document)?;
    result
}
async fn apply(
    state: Rc<State>,
    input: Value,
    settings: C,
    scan: u64,
    work: KernelWork<Fitted>,
) -> Result<Value, Fault> {
    let sigma = match field(&settings, "sigma") {
        C::Number(n) => n,
        _ => 3.,
    };

    let fitted = work.await;
    if state.worker.borrow().versions().scan != scan
        || matches!(state.document.store(StoreId::App).get("draft"), C::Null)
    {
        return Err(command_error(
            CommandErrorCode::Failed,
            "Nothing was measured there.",
        ));
    }
    let app = state.document.store(StoreId::App);
    let mut d = app.get("draft");
    let fitted = match fitted {
        Ok(f) => f,
        Err(KernelError::Source(message)) => {
            d = put(&d, "status", st("failed"));
            d = put(&d, "message", st(&message));
            patch(&state.document, vec![("draft", d)])?;
            return Err(command_error(CommandErrorCode::Failed, &message));
        }
        Err(e) => return Err(e.fault()),
    };

    let kind = if matches!(fitted, Fitted::Cylinder(_)) {
        Kind::Cylinder
    } else {
        Kind::Plane
    };
    let fit = fitted.cell();
    d = put(&d, "status", st("ready"));
    d = put(&d, "fit", fit.clone());
    d = put(&d, "message", C::Undefined);
    patch(&state.document, vec![("draft", d)])?;
    // finishAndCreate: setDraftAssumed, then the orientation, then the sides.
    let assumed = match (kind, get(&input, "assumed")) {
        (_, None) => C::Undefined,
        (Kind::Plane, Some(_)) => {
            return Err(command_error(
                CommandErrorCode::InvalidInput,
                "A plane has no diameter to assume.",
            ));
        }
        (Kind::Cylinder, Some(Value::Number(v))) => {
            if v.is_finite() && *v > 0. {
                let d = put(&app.get("draft"), "assumed", C::Number(*v));
                patch(&state.document, vec![("draft", d)])?;
                C::Number(*v)
            } else {
                C::Undefined
            }
        }
        (Kind::Cylinder, Some(_)) => return Err(Fault::host("admitted assumed number")),
    };
    let (oriented, orient) = finishing::orient(&state.document, get(&input, "orient"), &fit)?;
    let extend = finishing::extension(&state.document, get(&input, "extend"), &fit)?;
    let measured = if matches!(orient, C::Undefined) {
        C::Undefined
    } else {
        fit.clone()
    };
    let read = |key| match app.get(key) {
        C::Number(n) if (1. ..9_007_199_254_740_991.).contains(&n) => Ok(n as usize),
        _ => Err(Fault::host("native element counter limit")),
    };
    let id = read("nextId")?;
    let number = read("nextNumber")?;
    let counts = app.get("nextOfKind");
    let of_kind = match field(&counts, kind.id()) {
        C::Number(n) if (1. ..9_007_199_254_740_991.).contains(&n) => n as usize,
        _ => return Err(Fault::host("element kind counter limit")),
    };

    let default = text(&format!("{} {of_kind}", kind.label()));
    let name = match get(&input, "name") {
        Some(Value::String(s)) => {
            let whitespace = |v: &u16| matches!(*v,0x9..=0xd|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff);
            let first = s.iter().position(|v| !whitespace(v)).unwrap_or(s.len());
            let last = s
                .iter()
                .rposition(|v| !whitespace(v))
                .map_or(first, |i| i + 1);
            if first == last {
                default
            } else {
                s[first..last].to_vec()
            }
        }
        _ => default,
    };
    const PALETTE: [&str; 8] = [
        "#1877c0", "#e8590c", "#2e7d46", "#b5179e", "#c99a0a", "#0f9b9b", "#b3361c", "#6d4bbd",
    ];
    let source = obj(vec![
        ("type", st("fitted")),
        ("seeds", arr(vec![])),
        ("selection", C::Words(Arc::new(fitted.region()))),
        ("settings", settings.clone()),
    ]);
    let mut element = obj(vec![
        ("id", num(id)),
        ("kind", st(kind.id())),
        ("name", C::String(name.clone())),
        ("color", st(PALETTE[(number - 1) % 8])),
        ("source", source),
        ("status", st("done")),
        ("visible", C::Boolean(true)),
        ("fit", oriented.clone()),
        ("measured", measured.clone()),
        ("orient", orient.clone()),
        ("extend", extend.clone()),
        ("assumed", assumed.clone()),
    ]);

    let renamed = get(&input, "name").is_some();
    if renamed {
        element = put(&element, "message", C::Undefined)
    }
    let mut elements = match app.get("elements") {
        C::Array(v) => (*v).clone(),
        _ => return Err(Fault::host("invalid elements owner")),
    };
    elements.push(element);
    patch(
        &state.document,
        vec![
            ("nextId", num(id + 1)),
            ("nextNumber", num(number + 1)),
            ("nextOfKind", put(&counts, kind.id(), num(of_kind + 1))),
            ("draft", draft(kind, settings.clone())),
            ("elements", arr(elements)),
        ],
    )?;
    state
        .document
        .remember_surface(id as u64, Arc::new(fitted.region()))?;
    if renamed {
        patch(
            &state.document,
            vec![
                ("draft", C::Null),
                ("selectMode", st("paint")),
                (
                    "statusText",
                    C::String({
                        let mut status = name.clone();
                        status.extend(text(" updated."));
                        status
                    }),
                ),
            ],
        )?;
    }
    let cutoff = if sigma == 0. {
        "all points".into()
    } else {
        format!(
            "{} sigma cut-off",
            port_core::js_number::number_string(sigma)
        )
    };
    let mut description = vec![
        ("id", Value::Number(id as f64)),
        ("name", Value::String(name.clone())),
        ("kind", value_text(kind.id())),
        ("status", value_text("done")),
        ("visible", Value::Boolean(true)),
        ("fit", cell_value(&oriented)),
        ("measured", cell_value(&measured)),
        (
            "source",
            object(vec![
                ("type", value_text("fitted")),
                ("method", cell_value(&field(&settings, "method"))),
                ("sigmaCutoff", Value::Number(sigma)),
                ("cutoff", value_text(&cutoff)),
                ("markedPoints", Value::Number(fitted.region_size() as f64)),
            ]),
        ),
    ];
    if matches!(measured, C::Undefined) {
        description.retain(|(k, _)| *k != "measured")
    }
    if !matches!(assumed, C::Undefined) {
        description.push(("assumed", cell_value(&assumed)))
    }
    if !matches!(orient, C::Undefined) {
        description.push(("orient", cell_value(&orient)))
    }
    if !matches!(extend, C::Undefined) {
        description.push(("extend", cell_value(&extend)))
    }
    Ok(object(vec![("element", object(description))]))
}
