// SPDX-License-Identifier: AGPL-3.0-only
//! M3 project orchestration: prepare owned resources, then replace the real session.
use super::*;
use crate::project::{self as archive, Project};
#[path = "native_project_state.rs"]
mod readout;
const CORE: [&str; 9] = [
    "app",
    "schemaVersion",
    "appVersion",
    "name",
    "workspace",
    "scan",
    "deviation",
    "thickness",
    "flat",
];
fn vget(v: &Value, k: &str) -> Value {
    v.get(k).clone()
}
fn put(v: &mut Value, k: &str, x: Value) {
    if let Value::Object(f) = v {
        if let Some((_, v)) = f.iter_mut().find(|(n, _)| *n == text(k)) {
            *v = x
        } else {
            f.push((text(k), x))
        }
    }
}
fn remove(v: &mut Value, k: &str) {
    if let Value::Object(f) = v {
        f.retain(|(n, _)| *n != text(k))
    }
}
fn array(v: Vec<Value>) -> Value {
    Value::Array(v)
}
fn store_fields(doc: &Session, id: StoreId, keys: &str) -> Value {
    Value::Object(
        keys.split(' ')
            .map(|k| (text(k), cell_value(&doc.store(id).get(k))))
            .collect(),
    )
}
fn items(v: &Value) -> &[Value] {
    v.list().unwrap_or(&[])
}
fn name(v: &Value) -> String {
    v.text().map(String::from_utf16_lossy).unwrap_or_default()
}
fn element_json(e: &C) -> Value {
    let mut e = cell_value(e);
    let src = e.get("source");
    if src.get("type").is("fitted") {
        put(&mut e, "status", value_text("done"))
    }
    e
}
fn flat_blank() -> Result<Value, Fault> {
    archive::parse_json(br#"{"image":null,"pxPerMm":null,"calSource":"none","splitAxes":false,"edgeSensitivity":0.5,"showEdges":true,"snapToEdge":true,"showGrid":true,"elements":[],"nextId":1,"nameCounts":{},"dimensions":[],"nextDimId":1,"dimCounts":{},"datum":null,"counts":[],"nextCountId":1,"notes":[],"nextNoteId":1,"turns":0,"mirror":false,"subject":{"kind":"image"},"sheets":{}}"#).map_err(Fault::host)
}
fn flat_saved(mut f: Value) -> Value {
    for (k, v) in [
        ("notes", array(vec![])),
        ("nextNoteId", Value::Number(1.)),
        ("turns", Value::Number(0.)),
        ("mirror", Value::Boolean(false)),
        ("subject", object(vec![("kind", value_text("image"))])),
    ] {
        if get(&f, k).is_none() {
            put(&mut f, k, v);
        }
    }

    if let Some(active) = get(f.get("sheets"), "image").cloned() {
        for k in "pxPerMm calSource elements nextId nameCounts dimensions nextDimId dimCounts datum counts nextCountId notes nextNoteId turns mirror".split(' '){let default=match k{"turns"=>Value::Number(0.),"mirror"=>Value::Boolean(false),_=>Value::Undefined};put(&mut f,k,if matches!(k,"turns"|"mirror"){coalesce(get(&active,k),default)}else{vget(&active,k)});}
    }
    put(
        &mut f,
        "subject",
        object(vec![("kind", value_text("image"))]),
    );
    let mut sheet = object(vec![]);
    for k in "pxPerMm calSource elements nextId nameCounts dimensions nextDimId dimCounts datum counts nextCountId notes nextNoteId turns mirror".split(' '){let default=match k{"notes"=>array(vec![]),"nextNoteId"=>Value::Number(1.),"turns"=>Value::Number(0.),"mirror"=>Value::Boolean(false),_=>Value::Undefined};let x=get(&f,k).cloned().unwrap_or(default);put(&mut sheet,k,x)}
    let mut sheets = vget(&f, "sheets");
    if !matches!(sheets, Value::Object(_)) {
        sheets = object(vec![])
    }
    put(&mut sheets, "image", sheet);
    put(&mut f, "sheets", sheets);
    f
}
fn collect(state: &State) -> Result<Project, Fault> {
    let doc = &state.document;
    let sources = doc.sources();
    let app = doc.store(StoreId::App);
    let mut members = vec![];
    let mut scan = Value::Null;
    if let Some(src) = sources.scan {
        let member = format!("scan.{}", extension(&String::from_utf16_lossy(&src.name)));
        members.push((member.clone(), (*src.bytes).clone()));
        scan = store_fields(
            doc,
            StoreId::App,
            "appliedAlignment elements dimensions nextId nextNumber nextOfKind nextDimensionId nextOfDimGroup settings selectMode sections nextSectionNumber",
        );
        if let C::Array(elements) = app.get("elements") {
            put(
                &mut scan,
                "elements",
                array(elements.iter().map(element_json).collect()),
            )
        }
        if let Some(sections) = scan.get("sections").list() {
            let ss = sections
                .iter()
                .cloned()
                .map(|mut s| {
                    for k in ["cut", "cutKey", "message"] {
                        remove(&mut s, k)
                    }
                    s
                })
                .collect();
            put(&mut scan, "sections", array(ss))
        }
        let mut prefix = object(vec![
            ("fileName", cell_value(&app.get("fileName"))),
            ("member", value_text(&member)),
        ]);
        if let Some(u) = src.units
            && !matches!(u, MeshUnits::Mm)
        {
            put(&mut prefix, "units", value_text(u.id()))
        }
        if let Some(c) = &src.companions {
            let mut companions = vec![];
            for (i, c) in c.iter().enumerate() {
                let member = format!(
                    "scan-{}.{}",
                    i + 1,
                    extension(&String::from_utf16_lossy(&c.name))
                );
                members.push((member.clone(), (*c.bytes).clone()));
                companions.push(object(vec![
                    ("fileName", Value::String(c.name.clone())),
                    ("member", value_text(&member)),
                ]));
            }
            if !companions.is_empty() {
                put(&mut prefix, "companions", array(companions))
            }
        }
        if let (Value::Object(f), Value::Object(p)) = (&mut scan, prefix) {
            f.splice(0..0, p);
        }
        put(
            &mut scan,
            "showOverlays",
            cell_value(&app.get("showLabels")),
        );
        put(
            &mut scan,
            "showBackfaces",
            cell_value(&app.get("showBackfaces")),
        );
    }
    let mut deviation = store_fields(
        doc,
        StoreId::Deviation,
        "source align globalAlign pairs localMaxDistance targetId targetSide targetFacingDeg mapFacingDeg targetScope showElement range rangeAuto maxDistance maxDistanceAuto bands tolerance showHistogram showNominal showScan showMap split probes nextProbeId",
    );
    put(&mut deviation, "reference", Value::Null);
    put(&mut deviation, "align", Value::Null);
    put(&mut deviation, "globalAlign", Value::Null);
    let scope = doc
        .maps()
        .into_iter()
        .find(|(k, _)| *k == "elementScope")
        .and_then(|(_, v)| v);
    put(
        &mut deviation,
        "scope",
        match scope {
            Some(scan_session::MapValue::Words(v)) => {
                array(v.iter().map(|i| Value::Number(f64::from(*i))).collect())
            }
            _ => Value::Null,
        },
    );
    let mut thickness = store_fields(
        doc,
        StoreId::Thickness,
        "method maxThickness maxThicknessAuto coneRays coneAngleDeg normalDeviationDeg low high scaleAuto bands limit showHistogram probes nextProbeId",
    );
    put(&mut thickness, "measured", Value::Boolean(false));
    let saved = state.saved_project.borrow();
    let flat = saved
        .as_ref()
        .map(|p| vget(&p.manifest, "flat"))
        .map_or_else(flat_blank, Ok)?;
    let mut manifest = object(vec![
        ("app", value_text("ScanRuler")),
        ("schemaVersion", Value::Number(1.)),
        ("appVersion", value_text(port_core::REFERENCE_APP_VERSION)),
        ("workspace", value_text("elements")),
        ("scan", scan),
        ("deviation", deviation),
        ("thickness", thickness),
        ("flat", flat_saved(flat)),
    ]);
    if let C::String(n) = app.get("projectName")
        && !n.is_empty()
    {
        put(&mut manifest, "name", Value::String(n));
    }
    if let Some(saved) = saved.as_ref() {
        let mut unknown = false;
        if let Value::Object(f) = &saved.manifest {
            for (k, v) in f {
                if !CORE.iter().any(|n| *k == text(n)) {
                    unknown = true;
                    if let Value::Object(out) = &mut manifest {
                        out.push((k.clone(), v.clone()))
                    }
                }
            }
        }
        if unknown {
            let mut named = vec![];
            let s = saved.manifest.get("scan");
            for d in [
                s,
                saved.manifest.get("deviation").get("reference"),
                saved.manifest.get("flat").get("image"),
            ]
            .into_iter()
            .chain(items(s.get("companions")))
            {
                if let Some(n) = d.get("member").text() {
                    named.push(String::from_utf16_lossy(n))
                }
            }
            for (n, b) in &saved.members {
                if !named.contains(n) {
                    members.push((n.clone(), b.clone()))
                }
            }
        }
    }
    Ok(Project { manifest, members })
}
fn extension(s: &str) -> String {
    s.rsplit('.').next().unwrap_or("").to_lowercase()
}
fn stem(s: &str) -> String {
    s.rsplit_once('.')
        .map_or_else(|| s.into(), |(a, _)| a.into())
}
fn file_stem(doc: &Session) -> String {
    if let C::String(n) = doc.store(StoreId::App).get("projectName") {
        let n = String::from_utf16_lossy(&n)
            .chars()
            .map(|c| {
                if c <= '\u{1f}' || "\\/:*?\"<>|".contains(c) {
                    '-'
                } else {
                    c
                }
            })
            .collect::<String>();
        let n = n.trim_matches(|c: char| c == '.' || js_space(c));
        if !n.is_empty() {
            return n.into();
        }
    }
    match doc.store(StoreId::App).get("fileName") {
        C::String(n) if !n.is_empty() => stem(&String::from_utf16_lossy(&n)),
        _ => "project".into(),
    }
}
fn holds(doc: &Session) -> Value {
    let mut h = object(vec![]);
    for k in ["elements", "dimensions", "sections"] {
        if let C::Array(a) = doc.store(StoreId::App).get(k)
            && !a.is_empty()
        {
            put(&mut h, k, Value::Number(a.len() as f64))
        }
    }
    h
}
fn archive_fault(s: String) -> Fault {
    if s.starts_with("Native ") {
        if s.contains("not implemented") {
            command_error(CommandErrorCode::Unavailable, &s)
        } else {
            Fault::host(s)
        }
    } else {
        command_error(CommandErrorCode::Failed, &s)
    }
}
fn definition(load: bool) -> Value {
    let mut properties = vec![];
    let mut required = vec![];
    if load {
        properties = vec![
            (
                "bytes",
                object(vec![
                    ("type", value_text("string")),
                    ("format", value_text("binary")),
                    ("description", value_text("The project file’s contents.")),
                ]),
            ),
            (
                "name",
                object(vec![
                    ("type", value_text("string")),
                    ("minLength", Value::Number(1.)),
                    ("description", value_text("Its name, for the status line.")),
                ]),
            ),
            (
                "discard",
                object(vec![
                    ("type", value_text("boolean")),
                    (
                        "description",
                        value_text(
                            "true: replace the session though it holds measurements (lost unless saved as a project first). Refused otherwise when there is work to lose.",
                        ),
                    ),
                ]),
            ),
        ];
        required.push(value_text("bytes"))
    }
    let mut d = object(vec![
        ("name", value_text(if load { "load" } else { "save" })),
        (
            "title",
            value_text(if load {
                "Open a project"
            } else {
                "Save the project"
            }),
        ),
        (
            "description",
            value_text(if load {
                "Open a .scanruler project in place of the session — its models, then every measurement in it, remeasured, each fit as it was saved. Refuses while the session holds measurements unless discard is true. Needs the browser. Returns the session. Not undoable: an opened project starts a new history."
            } else {
                "The session as a .scanruler project file, as Save Project downloads it: the scan, the reference, the image and every measurement, each rebuilt when it is opened. Needs the browser. Returns the file’s bytes and its name, holds (what is in it, counted — read it before calling the file a revision: a page reset holds no model), aligned (whether the scan stands in an aligned frame) and says (what a workspace wants said of the work saved)."
            }),
        ),
        (
            "input",
            object(vec![
                ("type", value_text("object")),
                ("properties", object(properties)),
                ("required", array(required)),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
    ]);
    if !load
        && let Value::Object(f) = &mut d
        && let Some((_, v)) = f.iter_mut().find(|(k, _)| *k == text("input"))
    {
        remove(v, "required");
    }
    if load {
        put(&mut d, "history", Value::Boolean(false))
    } else {
        put(&mut d, "readOnly", Value::Boolean(true));
        put(&mut d, "returnsFile", Value::Boolean(true))
    }
    d
}
pub(super) fn register(runtime: &Runtime, state: &Rc<State>) -> Result<(), Fault> {
    for loading in [false, true] {
        let weak = Rc::downgrade(state);
        let rt = runtime.downgrade();
        runtime.register(
            &text("project"),
            definition(loading),
            Callbacks {
                label: None,
                discards: if loading {
                    Some(Rc::new(|v| {
                        Ok(Value::Boolean(matches!(
                            get(v, "discard"),
                            Some(Value::Boolean(true))
                        )))
                    }))
                } else {
                    None
                },
                run: Rc::new(move |input, ctx| {
                    let state = weak
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed project session"))?;
                    let runtime = rt
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed project runtime"))?;
                    if loading {
                        if !matches!(get(input, "discard"), Some(Value::Boolean(true)))
                            && let Err(Fault::Command(e)) =
                                require_discardable(&state.document, false)
                        {
                            return Err(command_error(
                                e.code(),
                                &String::from_utf16_lossy(e.message_units())
                                    .replace("opening another scan", "opening a project"),
                            ));
                        }
                        let Some(Value::Bytes(bytes)) = get(input, "bytes") else {
                            return Err(Fault::host("missing admitted project bytes"));
                        };
                        let bytes = bytes.clone();
                        let filename = get(input, "name")
                            .map(name)
                            .unwrap_or_else(|| "project.scanruler".into());
                        let copy = state.clone();
                        let abort = ctx.clone();
                        let job = state
                            .imports
                            .run(move || async move {
                                let previous=copy.document.store(StoreId::App).get("statusText");
                                patch(&copy.document,vec![("busy",C::Boolean(true)),("errorText",C::Null),("statusText",C::String(text("Reading project…")))])?;
                                let result=load_project(copy.clone(),runtime,bytes,filename,abort).await;
                                if let Err(e)=&result {patch(&copy.document,vec![("statusText",previous)])?;report_load_failure(&copy,e)?;}
                                result
                            })
                            .map_err(Fault::host)?;
                        ctx.spawn(async move {
                            match job.await {
                                Ok(v) => Ok(v),
                                Err(QueueError::Callback(e))=>Err(e),
                                Err(e) => Err(Fault::host(e)),
                            }
                        })?;
                    } else {
                        let copy = state.clone();
                        let abort = ctx.clone();
                        let job = state
                            .imports
                            .run(move || async move {
                                let p = collect(&copy)?;
                                if matches!(p.manifest.get("scan"), Value::Null)
                                    && !matches!(&p.manifest,Value::Object(f) if f.iter().any(|(k,_)|!CORE.iter().any(|n|*k==text(n))))
                                {
                                    return Err(command_error(
                                        CommandErrorCode::Failed,
                                        "There is nothing to save yet — no scan, image or work.",
                                    ));
                                }
                                let previous=copy.document.store(StoreId::App).get("statusText");
                                patch(&copy.document,vec![("errorText",C::Null),("busy",C::Boolean(true)),("statusText",C::String(text("Saving project…")))])?;
                                let pending=start_job("scanruler-project-save",move||archive::pack(&p).map_err(|e|kernel_fault(archive_fault(e))));
                                let packed=match pending{Ok(work)=>work.await.map_err(KernelError::fault),Err(e)=>Err(e)};
                                let bytes=match packed{Ok(b)=>b,Err(e)=>{let message=match &e{Fault::Command(e)=>String::from_utf16_lossy(e.message_units()),_=>format!("{e:?}")};let message=format!("The project could not be saved — {message}");patch(&copy.document,vec![("busy",C::Boolean(false)),("statusText",C::String(vec![])),("errorText",C::String(text(&message)))])?;return Err(if matches!(e,Fault::Host(_)){e}else{command_error(CommandErrorCode::Failed,&message)});}};
                                if let Some(e)=abort.abort_reason(){patch(&copy.document,vec![("busy",C::Boolean(false)),("statusText",previous)])?;return Err(Fault::Command(e));}
                                let status = format!("Project saved — {} MB.", size_text(bytes.len()));
                                patch(
                                    &copy.document,
                                    vec![("statusText", C::String(text(&status))),("busy",C::Boolean(false))],
                                )?;
                                Ok(object(vec![
                                    (
                                        "file",
                                        object(vec![
                                            (
                                                "name",
                                                value_text(&format!(
                                                    "{}.scanruler",
                                                    file_stem(&copy.document)
                                                )),
                                            ),
                                            ("mimeType", value_text("application/zip")),
                                            ("bytes", Value::Bytes(bytes)),
                                        ]),
                                    ),
                                    ("status", value_text(&status)),
                                    ("holds", holds(&copy.document)),
                                    (
                                        "aligned",
                                        Value::Boolean(!matches!(
                                            copy.document
                                                .store(StoreId::App)
                                                .get("appliedAlignment"),
                                            C::Null
                                        )),
                                    ),
                                ]))
                            })
                            .map_err(Fault::host)?;
                        ctx.spawn(async move {
                            match job.await {
                                Ok(v) => Ok(v),
                                Err(QueueError::Callback(e)) => Err(e),
                                Err(e) => Err(Fault::host(e)),
                            }
                        })?;
                    }
                    Ok(())
                }),
            },
        )?;
    }
    Ok(())
}
struct Candidate {
    project: Project,
    loaded: Option<Loaded>,
    elements: Value,
    regions: Vec<(u64, Vec<u32>)>,
}
fn prepare(bytes: Vec<u8>, mode: Mode, angle: f64) -> Result<Candidate, Fault> {
    let p = archive::unpack(&bytes).map_err(archive_fault)?;
    let m = &p.manifest;
    let s = m.get("scan");
    if !matches!(m.get("deviation").get("reference"), Value::Null)
        || !matches!(m.get("flat").get("image"), Value::Null)
        || matches!(m.get("thickness").get("measured"), Value::Boolean(true))
    {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native project reference/image/map reconstruction is not implemented.",
        ));
    }
    if ["dimensions", "sections"]
        .iter()
        .any(|k| !items(s.get(k)).is_empty())
        || ["elements", "dimensions", "counts", "notes"]
            .iter()
            .any(|k| !items(m.get("flat").get(k)).is_empty())
        || !matches!(m.get("flat").get("datum"), Value::Null)
    {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native project dimension/section/2D reconstruction is not implemented.",
        ));
    }
    if !items(s.get("companions")).is_empty() {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native project companion image reconstruction is not implemented.",
        ));
    }
    if ["deviation", "thickness", "flat"]
        .iter()
        .any(|w| m.get("workspace").is(w))
    {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native project workspaces beyond Measure are not implemented.",
        ));
    }
    if let Value::Object(sheets) = m.get("flat").get("sheets") {
        for (_, sheet) in sheets {
            if ["elements", "dimensions", "counts", "notes"]
                .iter()
                .any(|k| !items(sheet.get(k)).is_empty())
                || !matches!(sheet.get("datum"), Value::Null)
            {
                return Err(command_error(
                    CommandErrorCode::Unavailable,
                    "Native project 2D sheet reconstruction is not implemented.",
                ));
            }
        }
    }
    let loaded = if matches!(s, Value::Null) {
        None
    } else {
        let member = name(s.get("member"));
        let original_name = name(s.get("fileName"));
        let ext = extension(&member);
        let read_as = if extension(&original_name) == ext {
            original_name
        } else {
            format!("{}.{ext}", stem(&original_name))
        };
        let data = p
            .members
            .iter()
            .find(|(n, _)| *n == member)
            .ok_or_else(|| archive_fault(format!("Project is missing {member}.")))?
            .1
            .clone();
        let units = match name(s.get("units")).as_str() {
            "cm" => MeshUnits::Cm,
            "m" => MeshUnits::M,
            "in" => MeshUnits::In,
            _ => MeshUnits::Mm,
        };
        let a = s.get("appliedAlignment");
        let alignment = if matches!(a, Value::Null) {
            None
        } else {
            let mut r = [0.; 9];
            let mut t = [0.; 3];
            for (i, v) in items(a.get("r")).iter().enumerate() {
                r[i] = v.number().unwrap_or(0.)
            }
            for (i, v) in items(a.get("t")).iter().enumerate() {
                t[i] = v.number().unwrap_or(0.)
            }
            Some((r, t))
        };
        let src = Arc::new(SourceFile {
            name: text(&read_as),
            bytes: Arc::new(data),
            units: Some(units),
            companions: None,
        });
        let l = load_aligned(src, mode, angle, alignment).map_err(KernelError::fault)?;
        let bounds = port_core::bounds::bounds_of(&l.render.positions, l.graph.vertex_count)
            .map_err(Fault::host)?;
        if l.render.positions.iter().any(|v| !v.is_finite())
            || !bounds.radius.is_finite()
            || bounds.radius < 1e-4
        {
            return Err(Fault::host(
                "project scene metrics outside owned bounds contract",
            ));
        }
        Some(l)
    };
    let mut elements = vec![];
    let mut regions = vec![];
    for e in items(s.get("elements")) {
        let mut e = e.clone();
        let mut source = vget(&e, "source");
        if !e.get("kind").is("plane")
            || !source.get("type").is("fitted")
            || source.get("selection").list().is_none()
        {
            return Err(command_error(
                CommandErrorCode::Unavailable,
                "Native project restore currently supports marked-plane recipes only.",
            ));
        }
        if matches!(e.get("fit"), Value::Undefined)
            && ["orient", "extend", "assumed"]
                .iter()
                .any(|k| get(&e, k).is_some())
        {
            return Err(command_error(
                CommandErrorCode::Unavailable,
                "Native project finishing rebuild without a saved fit is not implemented.",
            ));
        }
        let settings = get(&source, "settings")
            .filter(|v| !matches!(v, Value::Null))
            .cloned()
            .unwrap_or_else(|| vget(s, "settings"));
        put(&mut source, "settings", settings.clone());
        let vertices = items(source.get("selection"))
            .iter()
            .map(|v| v.number().unwrap_or(0.) as u64 as u32)
            .collect::<Vec<_>>();
        let l = loaded
            .as_ref()
            .ok_or_else(|| archive_fault("Malformed project: scan entry.".into()))?;
        let measured = port_core::marked_plane::fit_plane_on_selection(
            &l.graph.positions,
            &l.graph.normals,
            &vertices,
            settings.get("sigma").number().unwrap_or(3.),
            1_000_000,
        );
        match measured {
            Ok(f) => {
                regions.push((e.get("id").number().unwrap_or(0.) as u64, f.region.clone()));
                if matches!(e.get("fit"), Value::Undefined) {
                    put(&mut e, "fit", cell_value(&plane::fit_cell(&f)));
                    put(&mut e, "status", value_text("done"));
                    remove(&mut e, "message")
                }
            }
            Err(port_core::marked_plane::MarkedPlaneError::Source(msg)) => {
                put(&mut e, "status", value_text("failed"));
                put(&mut e, "message", value_text(&msg))
            }
            Err(e) => return Err(Fault::host(format!("{e:?}"))),
        }
        put(&mut e, "source", source);
        elements.push(e);
    }
    Ok(Candidate {
        project: p,
        loaded,
        elements: array(elements),
        regions,
    })
}
async fn load_project(
    state: Rc<State>,
    runtime: Runtime,
    bytes: Vec<u8>,
    filename: String,
    ctx: CommandContext,
) -> Result<Value, Fault> {
    let app = state.document.store(StoreId::App);
    let mode = match app.get("creaseMode") {
        C::String(s) if s == text("off") => Mode::Off,
        C::String(s) if s == text("auto") => Mode::Auto,
        _ => Mode::On,
    };
    let angle = match app.get("creaseAngle") {
        C::Number(n) if n.is_finite() => n.clamp(5., 120.),
        _ => 30.,
    };
    #[cfg(test)]
    let test_gate = LOAD_TEST_GATE.lock().map_err(Fault::host)?.take();
    let c = start_job("scanruler-project", move || {
        #[cfg(test)]
        if let Some(gate) = test_gate {
            gate.wait();
        }

        prepare(bytes, mode, angle).map_err(|e| match e {
            Fault::Command(e) => KernelError::Project(e.code(), e.message_units().to_vec()),
            Fault::Host(e) => KernelError::Host(format!("{e:?}")),
            Fault::Error { message, .. } | Fault::Text(message) => {
                KernelError::Source(String::from_utf16_lossy(&message))
            }
            _ => KernelError::Host("native project preparation failed".into()),
        })
    })?
    .await
    .map_err(KernelError::fault)?;
    if let Some(e) = ctx.abort_reason() {
        return Err(Fault::Command(e));
    }
    let m = &c.project.manifest;
    let s = m.get("scan");
    let mut patches = vec![];
    if !matches!(s, Value::Null) {
        let mut patch_fields = vec![];
        for k in "nextId nextNumber nextOfKind nextDimensionId nextOfDimGroup settings selectMode appliedAlignment".split(' '){patch_fields.push((text(k),plane::cell(vget(s,k))))}
        patch_fields.extend([
            (text("elements"), restore_elements(c.elements.clone())),
            (text("dimensions"), C::Array(Arc::new(vec![]))),
            (text("sections"), C::Array(Arc::new(vec![]))),
            (text("showLabels"), plane::cell(vget(s, "showOverlays"))),
            (
                text("showBackfaces"),
                plane::cell(coalesce(get(s, "showBackfaces"), Value::Boolean(true))),
            ),
            (
                text("nextSectionNumber"),
                plane::cell(coalesce(get(s, "nextSectionNumber"), Value::Number(1.))),
            ),
            (text("fileName"), plane::cell(vget(s, "fileName"))),
        ]);
        patches.push((StoreId::App, patch_fields));
    }
    for (id, part, keys) in [
        (
            StoreId::Deviation,
            "deviation",
            "source align globalAlign pairs localMaxDistance targetId targetSide targetFacingDeg targetScope showElement range rangeAuto maxDistance maxDistanceAuto bands tolerance showHistogram showNominal showScan showMap split probes nextProbeId",
        ),
        (
            StoreId::Thickness,
            "thickness",
            "method maxThickness maxThicknessAuto coneRays coneAngleDeg normalDeviationDeg low high scaleAuto bands limit showHistogram probes nextProbeId",
        ),
    ] {
        patches.push((
            id,
            keys.split(' ')
                .map(|k| (text(k), plane::cell(vget(m.get(part), k))))
                .collect(),
        ));
    }
    let project_name = name(m.get("name"))
        .trim_matches(|c: char| js_space(c))
        .to_string();
    patches.push((
        StoreId::App,
        vec![(
            text("projectName"),
            if project_name.is_empty() {
                C::Null
            } else {
                C::String(text(&project_name))
            },
        )],
    ));
    for (store, fields) in &patches {
        state.document.store(*store).preflight(fields)?;
    }
    let id = state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    let prepared = if let Some(l) = c.loaded {
        state
            .worker
            .borrow_mut()
            .stage_scan(id, l.graph.clone())
            .map_err(Fault::host)?;
        state.components.set(l.components);
        Some(PreparedScan {
            asset: Arc::new(ScanAsset {
                source: l.source,
                positions: Arc::new(l.render.positions),
                indices: Arc::new(l.render.indices),
                vertex_count: l.graph.vertex_count,
                triangle_count: l.graph.triangle_count,
                colors: None,
            }),
            view: state.no_view.clone(),
        })
    } else {
        state.components.set(0);
        None
    };
    state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    state
        .worker
        .borrow_mut()
        .commit(
            if prepared.is_some() {
                Change::Prepared(id)
            } else {
                Change::Clear
            },
            Change::Clear,
            |_| Ok::<(), ()>(()),
        )
        .map_err(Fault::host)?;
    state.document.commit_scan(prepared.as_ref())?;
    runtime.clear_history()?;
    for (store, fields) in patches {
        state.document.store(store).patch(fields)?;
    }
    for (id, region) in c.regions {
        state.document.remember_surface(id, Arc::new(region))?
    }
    let d = m.get("deviation");
    let scope = if matches!(d.get("scope"), Value::Null) {
        None
    } else {
        Some(Arc::new(
            items(d.get("scope"))
                .iter()
                .map(|v| v.number().unwrap_or(0.) as u64 as u32)
                .collect::<Vec<_>>(),
        ))
    };
    state.document.set_map(
        "elementScope",
        scope.clone().map(scan_session::MapValue::Words),
    )?;
    let scope_version = match state.document.store(StoreId::Deviation).get("scopeVersion") {
        C::Number(v) => v,
        _ => 0.,
    };
    state.document.store(StoreId::Deviation).patch(vec![
        (text("scopeVersion"), C::Number(scope_version + 1.)),
        (
            text("scopeCount"),
            C::Number(scope.as_ref().map_or(0, |s| s.len()) as f64),
        ),
        (
            text("mapFacingDeg"),
            plane::cell(
                get(d, "mapFacingDeg")
                    .cloned()
                    .unwrap_or(Value::Number(90.)),
            ),
        ),
    ])?;
    let project_name = name(m.get("name"))
        .trim_matches(|c: char| js_space(c))
        .to_string();
    let unknown = if let Value::Object(f) = m {
        f.iter().any(|(k, _)| !CORE.iter().any(|n| *k == text(n)))
    } else {
        false
    };
    patch(
        &state.document,
        vec![
            (
                "projectName",
                if project_name.is_empty() {
                    C::Null
                } else {
                    C::String(text(&project_name))
                },
            ),
            (
                "appliedAlignment",
                plane::cell(if matches!(s, Value::Null) {
                    Value::Null
                } else {
                    vget(s, "appliedAlignment")
                }),
            ),
            ("busy", C::Boolean(false)),
            (
                "statusText",
                C::String(text(&format!(
                    "Project opened — {filename}.{}",
                    if unknown {
                        " It also holds work this build has no workspace for; that is kept, and saved with the project."
                    } else {
                        ""
                    }
                ))),
            ),
        ],
    )?;
    *state.saved_project.borrow_mut() = Some(c.project);
    state
        .worker
        .borrow_mut()
        .reserve_request()
        .map_err(Fault::host)?;
    state
        .worker
        .borrow_mut()
        .discard(&[id])
        .map_err(Fault::host)?;
    readout::state(&state, &runtime)
}

fn restore_elements(v: Value) -> C {
    C::Array(Arc::new(
        items(&v)
            .iter()
            .cloned()
            .map(|v| {
                let src = vget(&v, "source");
                let selection = items(src.get("selection"))
                    .iter()
                    .map(|v| v.number().unwrap_or(0.) as u64 as u32)
                    .collect();
                let e = plane::cell(v);
                let source = plane::put(
                    &plane::cell(src),
                    "selection",
                    C::Words(Arc::new(selection)),
                );
                plane::put(&e, "source", source)
            })
            .collect(),
    ))
}

fn kernel_fault(e: Fault) -> KernelError {
    match e {
        Fault::Command(e) => KernelError::Project(e.code(), e.message_units().to_vec()),
        Fault::Host(e) => KernelError::Host(format!("{e:?}")),
        Fault::Error { message, .. } | Fault::Text(message) => {
            KernelError::Source(String::from_utf16_lossy(&message))
        }
        _ => KernelError::Host("native project worker failed".into()),
    }
}

fn report_load_failure(state: &State, e: &Fault) -> Result<(), Fault> {
    let message = match e {
        Fault::Command(e) => e.message_units().to_vec(),
        Fault::Error { message, .. } | Fault::Text(message) => message.clone(),
        _ => text(&format!("{e:?}")),
    };
    patch(
        &state.document,
        vec![
            ("busy", C::Boolean(false)),
            ("errorText", C::String(message)),
        ],
    )
}

// The source's toFixed(1): integer rounding of the exact binary64 input,
// choosing the larger integer on a tie (Rust formatting chooses even).
fn size_text(bytes: usize) -> String {
    let v = bytes as f64 / 1e6;
    if v == 0. {
        return "0.0".into();
    }
    let bits = v.to_bits();
    let m = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let power = ((bits >> 52) & 0x7ff) as i32 - 1023 - 52 + 1;
    let n = u128::from(m) * 5;
    let r = if power >= 0 {
        n << (power as u32)
    } else {
        let shift = (-power) as u32;
        if shift >= 128 {
            0
        } else {
            (n >> shift) + u128::from((n & ((1u128 << shift) - 1)) >= (1u128 << (shift - 1)))
        }
    };
    format!("{}.{}", r / 10, r % 10)
}
fn js_space(c: char) -> bool {
    matches!(c as u32,9..=13|32|160|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn coalesce(v: Option<&Value>, fallback: Value) -> Value {
    v.filter(|v| !matches!(v, Value::Null))
        .cloned()
        .unwrap_or(fallback)
}

#[cfg(test)]
static LOAD_TEST_GATE: std::sync::Mutex<Option<Arc<std::sync::Barrier>>> =
    std::sync::Mutex::new(None);
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::*;
    #[test]
    fn expired_load_never_commits_the_late_prepared_scan() {
        let e = Engine::new().unwrap();
        let scan = include_bytes!("../../../../docs/port-plan/fixtures/native-project/scan.ply");
        e.execute(
            "scan.open",
            object(vec![
                ("bytes", Value::Bytes(scan.to_vec())),
                ("name", value_text("scan.ply")),
            ]),
        )
        .unwrap();
        e.execute(
            "element.fit_marked",
            object(vec![
                ("kind", value_text("plane")),
                (
                    "vertices",
                    array((0..25).map(|n| Value::Number(n as f64)).collect()),
                ),
                ("sigma", Value::Number(0.)),
            ]),
        )
        .unwrap();
        let bytes = archive::pack(&collect(&e.state).unwrap()).unwrap();
        let source = e.document().sources().scan.unwrap();
        let graph = e.logical_graph().unwrap();
        let elements = e.document().store(StoreId::App).get("elements");
        let history = e.history().past.len();
        let gate = Arc::new(std::sync::Barrier::new(2));
        *LOAD_TEST_GATE.lock().unwrap() = Some(gate.clone());
        let result = Rc::new(RefCell::new(None));
        let receive = result.clone();
        let ticket = e
            .submit(
                "project.load",
                object(vec![
                    ("bytes", Value::Bytes(bytes)),
                    ("discard", Value::Boolean(true)),
                ]),
                move |v| *receive.borrow_mut() = Some(v),
            )
            .unwrap();
        // Advance the test host's timer, leaving production deadlines unchanged.
        e.poll().unwrap();
        for timer in e.host.timers.borrow_mut().iter_mut() {
            timer.at = Instant::now();
        }
        e.poll().unwrap();
        gate.wait();
        let until = Instant::now() + Duration::from_secs(5);
        while !ticket.complete() || e.state.imports.busy() {
            assert!(Instant::now() < until);
            e.poll().unwrap();
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            matches!(result.borrow().as_ref(),Some(Err(Fault::Command(e))) if e.code()==CommandErrorCode::Timeout)
        );
        assert!(Arc::ptr_eq(&source, &e.document().sources().scan.unwrap()));
        assert!(Arc::ptr_eq(&graph, &e.logical_graph().unwrap()));
        assert_eq!(
            cell_value(&elements),
            cell_value(&e.document().store(StoreId::App).get("elements"))
        );
        assert_eq!(history, e.history().past.len());
    }
}
