// SPDX-License-Identifier: AGPL-3.0-only
//! Native no-viewport command integration. Supported standalone imports: STL/PLY/OBJ.
//! The shared runtime, import queue, actual document and staged graph owner are reused.
//! Other format/measurement/project commands remain explicit unported capabilities.
use crate::{
    error::{CommandError, CommandErrorCode},
    guarded::{self, Callbacks, Context as CommandContext, Fault, Host, Runtime, WeakRuntime},
    history_record::{Cell as C, Compare, Section, Snapshot},
    import_queue::{ImportQueue, QueueError, QueueLimits},
    scan_session::{
        self, MeshUnits, PreparedScan, PreparedView, ScanAsset, Session, SourceFile, StoreId,
    },
    scan_worker::{Change, Worker},
    schema_value::{Text, Value},
    tasks::{Owner, Ticket},
};
use port_core::{
    crease::{self, Mode, Split},
    mesh_graph::{self, GraphLimits, MeshGraph, ParsedMesh},
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[path = "native_history.rs"]
mod history;
#[path = "native_plane.rs"]
mod plane;
#[path = "native_project.rs"]
mod project;
#[path = "native_scan_definition.rs"]
mod scan_definition;
#[path = "native_selection.rs"]
mod selection;
/// Bounded filesystem/engine input policy; parser-specific limits remain independent.
pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn value_text(s: &str) -> Value {
    Value::String(text(s))
}
fn object(v: Vec<(&str, Value)>) -> Value {
    Value::Object(v.into_iter().map(|(k, v)| (text(k), v)).collect())
}
// Optional JS arguments use undefined as absence, even when the own key exists.
fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(fields) => fields
            .iter()
            .find(|(k, _)| k.iter().copied().eq(key.encode_utf16()))
            .map(|(_, v)| v)
            .filter(|value| !matches!(value, Value::Undefined)),
        _ => None,
    }
}
fn command_error(code: CommandErrorCode, message: &str) -> Fault {
    match CommandError::from_utf8(code, message, 1_000_000) {
        Ok(e) => Fault::Command(Arc::new(e)),
        Err(e) => Fault::host(e),
    }
}
fn cell_value(c: &C) -> Value {
    match c {
        C::Undefined => Value::Undefined,
        C::Null => Value::Null,
        C::Boolean(v) => Value::Boolean(*v),
        C::Number(v) => Value::Number(*v),
        C::String(v) => Value::String(v.clone()),
        C::Bytes(v) => Value::Bytes((**v).clone()),
        C::Words(v) => Value::Array(v.iter().map(|v| Value::Number(f64::from(*v))).collect()),
        C::Array(v) => Value::Array(v.iter().map(cell_value).collect()),
        C::Object(v) => Value::Object(v.iter().map(|(k, v)| (k.clone(), cell_value(v))).collect()),
    }
}
fn patch(document: &Session, fields: Vec<(&str, C)>) -> Result<(), Fault> {
    document
        .store(StoreId::App)
        .patch(fields.into_iter().map(|(k, v)| (text(k), v)).collect())
}
// Native supported document families. Flat sheets/plugin sections and the saved-project
// probe must be connected when those owners are implemented; they are not synthesized.
fn require_discardable(document: &Session, discard: bool) -> Result<(), Fault> {
    if discard {
        return Ok(());
    }
    let app = document.store(StoreId::App);
    let mut held = Vec::new();
    for (key, singular) in [
        ("elements", "element"),
        ("dimensions", "dimension"),
        ("sections", "section"),
    ] {
        if let C::Array(values) = app.get(key)
            && !values.is_empty()
        {
            held.push(format!(
                "{} {}{}",
                values.len(),
                singular,
                if values.len() == 1 { "" } else { "s" }
            ));
        }
    }
    if held.is_empty() {
        return Ok(());
    }
    let list = if held.len() > 1 {
        format!(
            "{} and {}",
            held[..held.len() - 1].join(", "),
            held[held.len() - 1]
        )
    } else {
        held[0].clone()
    };
    let project = match app.get("projectName") {
        C::String(s) if !s.is_empty() => {
            format!(" in the project “{}”", String::from_utf16_lossy(&s))
        }
        _ => String::new(),
    };
    Err(command_error(
        CommandErrorCode::InvalidState,
        &format!(
            "The session holds {list}{project} that opening another scan would replace. Save it with project.save first, or pass discard: true to let it go."
        ),
    ))
}
fn raw_now() -> Result<f64, Fault> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(Fault::host)?
        .as_millis() as f64)
}
struct Timer {
    id: u64,
    at: Instant,
    callback: Rc<dyn Fn() -> Result<(), Fault>>,
}
struct NativeHost {
    state: Weak<State>,
    timers: RefCell<Vec<Timer>>,
    next_timer: Cell<u64>,
    observer_errors: RefCell<Vec<Fault>>,
}
impl Host for NativeHost {
    fn busy(&self) -> Result<Option<Text>, Fault> {
        let s = self
            .state
            .upgrade()
            .ok_or_else(|| Fault::host("closed native session"))?;
        Ok(if s.imports.busy() {
            Some(text("An import is still running."))
        } else {
            None
        })
    }
    fn workspace(&self) -> Result<Text, Fault> {
        Ok(text("elements"))
    }
    fn capture(&self) -> Result<Snapshot, Fault> {
        let s = self
            .state
            .upgrade()
            .ok_or_else(|| Fault::host("closed native session"))?;
        let app = s.document.store(StoreId::App);
        let fields = [
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
        ]
        .into_iter()
        .map(|k| (text(k), app.get(k)))
        .collect();
        Ok(Snapshot {
            sections: vec![Section {
                key: text("measure"),
                value: C::Object(Arc::new(fields)),
                compare: Compare::Fields,
            }],
        })
    }
    fn now(&self) -> Result<f64, Fault> {
        raw_now()
    }
    fn stale_now(&self) -> Result<f64, Fault> {
        raw_now()
    }
    fn timer(&self, ms: f64, callback: Rc<dyn Fn() -> Result<(), Fault>>) -> u64 {
        let id = self.next_timer.get();
        self.next_timer.set(id + 1);
        self.timers.borrow_mut().push(Timer {
            id,
            at: Instant::now() + Duration::from_secs_f64(ms.max(0.) / 1000.),
            callback,
        });
        id
    }
    fn clear_timer(&self, id: u64) {
        self.timers.borrow_mut().retain(|t| t.id != id)
    }
    fn settle(&self, _: &[u16]) -> Result<crate::settle::Work<'static, Fault>, Fault> {
        Ok(Box::pin(std::future::ready(Ok(()))))
    }
    fn settle_readonly(
        &self,
        _: &[u16],
    ) -> Result<Option<crate::settle::Work<'static, Fault>>, Fault> {
        Ok(None)
    }
    fn observer_fault(&self, error: Fault) {
        let mut errors = self.observer_errors.borrow_mut();
        if errors.len() < 128 {
            errors.push(error)
        }
    }
    fn stop(&self, _: crate::execute::Done) -> crate::settle::Work<'static, Fault> {
        Box::pin(std::future::ready(Ok(())))
    }
}
impl NativeHost {
    fn fire_timers(&self) -> Result<(), Fault> {
        let now = Instant::now();
        let mut due = vec![];
        {
            let mut timers = self.timers.borrow_mut();
            let mut i = 0;
            while i < timers.len() {
                if timers[i].at <= now {
                    due.push(timers.remove(i).callback)
                } else {
                    i += 1
                }
            }
        }
        for callback in due {
            callback()?
        }
        Ok(())
    }
}
struct NoView;
impl PreparedView for NoView {
    fn commit(&self) -> Result<(), Fault> {
        Ok(())
    }
    fn dispose(&self) -> Result<(), Fault> {
        Ok(())
    }
}
struct State {
    document: Session,
    worker: RefCell<Worker<MeshGraph, Vec<u8>>>,
    imports: ImportQueue<Fault>,
    components: Cell<usize>,
    no_view: Rc<dyn PreparedView>,
    command_marked: RefCell<Option<selection::Marked>>,
    saved_project: RefCell<Option<crate::project::Project>>,
}
#[derive(Debug)]
enum KernelError {
    Source(String),
    Host(String),
    Unavailable(String),
    Project(CommandErrorCode, Text),
}
impl KernelError {
    fn fault(self) -> Fault {
        match self {
            Self::Project(code, message) => CommandError::from_utf16(code, &message, 1_000_000)
                .map(|e| Fault::Command(Arc::new(e)))
                .unwrap_or_else(Fault::host),
            Self::Source(s) => Fault::error(&s),
            Self::Host(s) => Fault::host(s),
            Self::Unavailable(s) => command_error(CommandErrorCode::Unavailable, &s),
        }
    }
}
struct Loaded {
    graph: Arc<MeshGraph>,
    render: Split,
    report: crease::Report,
    source: Arc<SourceFile>,
    components: usize,
    file_note: String,
}
fn load(source: Arc<SourceFile>, mode: Mode, angle: f64) -> Result<Loaded, KernelError> {
    load_aligned(source, mode, angle, None)
}
fn load_aligned(
    source: Arc<SourceFile>,
    mode: Mode,
    angle: f64,
    alignment: Option<([f64; 9], [f64; 3])>,
) -> Result<Loaded, KernelError> {
    let name = String::from_utf16_lossy(&source.name).to_ascii_lowercase();
    let bytes = &*source.bytes;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(KernelError::Host("native import byte limit".into()));
    }
    let mut file_note = String::new();
    let mut parsed = if name.ends_with(".stl") {
        let positions = port_io::stl_dispatch::read_stl(
            bytes,
            port_io::stl_dispatch::StlLimits {
                max_bytes: MAX_INPUT_BYTES,
                max_triangles: 1_000_000,
            },
            |_| {},
        )
        .map_err(|e| match e {
            port_io::stl_dispatch::StlError::InvalidFile
            | port_io::stl_dispatch::StlError::InvalidAscii => KernelError::Source(e.to_string()),
            _ => KernelError::Host(e.to_string()),
        })?;
        ParsedMesh {
            positions,
            soup: true,
            indices: None,
            keep_winding: false,
        }
    } else if name.ends_with(".ply") {
        let p = port_io::ply::read_ply(
            bytes,
            port_io::ply::PlyLimits {
                max_bytes: MAX_INPUT_BYTES,
                max_vertices: 1_000_000,
                max_triangles: 1_000_000,
                max_work: 100_000_000,
            },
            |_| {},
        )
        .map_err(|e| match e {
            port_io::ply::PlyError::ResourceLimit | port_io::ply::PlyError::Allocation => {
                KernelError::Host(e.to_string())
            }
            _ => KernelError::Source(e.to_string()),
        })?;
        ParsedMesh {
            positions: p.positions,
            soup: false,
            indices: Some(p.indices),
            keep_winding: p.keep_winding,
        }
    } else if name.ends_with(".obj") {
        let p = port_io::obj::read_obj(
            bytes,
            port_io::obj::ObjLimits {
                max_bytes: MAX_INPUT_BYTES,
                max_vertices: 1_000_000,
                max_uvs: 1_000_000,
                max_indices: 3_000_000,
                max_face_vertices: 1_000_000,
                max_materials: 100_000,
                max_libraries: 4096,
                max_work: 100_000_000,
                texture: port_io::obj_texture::TextureLimits {
                    max_bytes: MAX_INPUT_BYTES,
                    max_vertices: 1_000_000,
                    max_files: 4096,
                    max_materials: 100_000,
                    max_name_bytes: 4096,
                    max_work: 100_000_000,
                },
            },
            |_| {},
            None,
        )
        .map_err(|e| match e {
            port_io::obj::ObjError::Limit
            | port_io::obj::ObjError::Allocation
            | port_io::obj::ObjError::Texture(_) => KernelError::Host(e.to_string()),
            _ => KernelError::Source(e.to_string()),
        })?;
        // scan.open has no companion-file input. Preserve the source's missing
        // texture notice rather than silently claiming a coloured import.
        if let Some(port_io::obj_texture::TextureNote::Missing(files)) = p.texture {
            file_note = format!(
                "This OBJ has a texture ({}) that did not come with it — open the OBJ, its .mtl and the image together (select all three, or drop their folder) to see the scan in colour. ",
                files.join(", ")
            );
        }
        ParsedMesh {
            positions: p.positions,
            soup: false,
            indices: Some(p.indices),
            keep_winding: false,
        }
    } else {
        return Err(KernelError::Unavailable("This native engine currently opens STL, PLY and OBJ; other import commands are not implemented.".into()));
    };
    let scale = match source.units.unwrap_or(MeshUnits::Mm) {
        MeshUnits::Mm => 1.,
        MeshUnits::Cm => 10.,
        MeshUnits::M => 1000.,
        MeshUnits::In => 25.4,
    };
    if scale != 1. {
        for value in &mut parsed.positions {
            *value = (f64::from(*value) * scale) as f32
        }
    }
    let mut graph = mesh_graph::build_mesh_graph(
        parsed,
        GraphLimits {
            vertices: 3_000_000,
            indices: 3_000_000,
            neighbors: 6_000_000,
        },
        |_| {},
    )
    .map_err(|e| match e {
        mesh_graph::GraphError::Source(s) => KernelError::Source(s.into()),
        _ => KernelError::Host(format!("{e:?}")),
    })?;
    if let Some((r, t)) = alignment {
        for (values, translation) in [(&mut graph.positions, t), (&mut graph.normals, [0.; 3])] {
            for p in values.as_chunks_mut::<3>().0 {
                let [x, y, z] = [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
                for i in 0..3 {
                    p[i] = (r[i * 3] * x + r[i * 3 + 1] * y + r[i * 3 + 2] * z + translation[i])
                        as f32;
                }
            }
        }
    }
    let groups =
        port_core::components::vertex_components(&graph.indices, graph.vertex_count, 1_000_000)
            .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    let components = groups
        .parent()
        .iter()
        .enumerate()
        .filter(|(i, p)| **p as usize == *i)
        .count();
    let (render, report) = crease::scan_render(
        &graph,
        mode,
        angle,
        crease::Limits {
            vertices: 1_000_000,
            indices: 3_000_000,
            render_vertices: 2_000_000,
        },
    )
    .map_err(|e| KernelError::Host(format!("{e:?}")))?;
    Ok(Loaded {
        graph: Arc::new(graph),
        render,
        report,
        source,
        components,
        file_note,
    })
}
struct Reply<T> {
    value: Option<Result<T, KernelError>>,
    waker: Option<Waker>,
}
struct KernelWork<T>(Arc<Mutex<Reply<T>>>);
impl<T> Future for KernelWork<T> {
    type Output = Result<T, KernelError>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.lock() {
            Ok(mut reply) => {
                if let Some(value) = reply.value.take() {
                    Poll::Ready(value)
                } else {
                    reply.waker = Some(ctx.waker().clone());
                    Poll::Pending
                }
            }
            Err(_) => Poll::Ready(Err(KernelError::Host(
                "native kernel reply lock failed".into(),
            ))),
        }
    }
}
fn start_kernel(
    source: Arc<SourceFile>,
    mode: Mode,
    angle: f64,
) -> Result<KernelWork<Loaded>, Fault> {
    start_job("scanruler-import", move || load(source, mode, angle))
}
const MAX_KERNEL_JOBS: usize = 8;
static ACTIVE_KERNEL_JOBS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct KernelPermit;
impl KernelPermit {
    fn acquire() -> Result<Self, Fault> {
        let mut current = ACTIVE_KERNEL_JOBS.load(std::sync::atomic::Ordering::Acquire);
        loop {
            if current >= MAX_KERNEL_JOBS {
                return Err(Fault::host("native kernel concurrency budget"));
            }
            match ACTIVE_KERNEL_JOBS.compare_exchange_weak(
                current,
                current + 1,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
        Ok(Self)
    }
}
impl Drop for KernelPermit {
    fn drop(&mut self) {
        ACTIVE_KERNEL_JOBS.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}
fn start_job<T: Send + 'static>(
    name: &str,
    work: impl FnOnce() -> Result<T, KernelError> + Send + 'static,
) -> Result<KernelWork<T>, Fault> {
    let permit = KernelPermit::acquire()?;
    let shared = Arc::new(Mutex::new(Reply {
        value: None,
        waker: None,
    }));
    let output = shared.clone();
    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            let _permit = permit;
            let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
                .unwrap_or_else(|_| Err(KernelError::Host("native mesh worker failed".into())));
            let wake = match output.lock() {
                Ok(mut reply) => {
                    reply.value = Some(value);
                    reply.waker.take()
                }
                Err(_) => None,
            };
            if let Some(wake) = wake {
                wake.wake()
            }
        })
        .map_err(Fault::host)?;
    Ok(KernelWork(shared))
}
impl State {
    fn summary(&self) -> Result<Value, Fault> {
        self.worker
            .borrow_mut()
            .reserve_request()
            .map_err(Fault::host)?;
        let graph = self
            .worker
            .borrow()
            .graph()
            .ok_or_else(|| command_error(CommandErrorCode::NoScan, "Open a scan first."))?;
        let bounds = port_core::bounds::extent_of(&graph.positions, graph.vertex_count)
            .map_err(Fault::host)?;
        let app = self.document.store(StoreId::App);
        let sources = self.document.sources();
        let units = sources
            .scan
            .as_ref()
            .and_then(|s| s.units)
            .unwrap_or(MeshUnits::Mm);
        let size = match app.get("modelSize") {
            C::Number(n) => 2. * n,
            _ => return Err(Fault::host("invalid model size")),
        };
        Ok(object(vec![
            ("fileName", cell_value(&app.get("fileName"))),
            ("units", value_text(units.id())),
            ("vertices", cell_value(&app.get("vertexCount"))),
            ("triangles", cell_value(&app.get("triangleCount"))),
            (
                "bounds",
                object(vec![
                    (
                        "min",
                        Value::Array(bounds.min.into_iter().map(Value::Number).collect()),
                    ),
                    (
                        "max",
                        Value::Array(bounds.max.into_iter().map(Value::Number).collect()),
                    ),
                ]),
            ),
            ("center", cell_value(&app.get("modelCenter"))),
            ("size", Value::Number(size)),
        ]))
    }
    async fn import(&self, source: Arc<SourceFile>) -> Result<(), Fault> {
        let app = self.document.store(StoreId::App);
        let previous = app.get("statusText");
        patch(
            &self.document,
            vec![
                ("busy", C::Boolean(true)),
                ("errorText", C::Null),
                ("statusText", C::String(text("Reading file…"))),
            ],
        )?;
        let result = async {
            let id = self.worker.borrow_mut().reserve_request().map_err(Fault::host)?;
            let mode = match app.get("creaseMode") {
                C::String(s) if s == text("off") => Mode::Off,
                C::String(s) if s == text("auto") => Mode::Auto,
                _ => Mode::On,
            };
            let angle = match app.get("creaseAngle") {
                C::Number(n) if n.is_finite() => n.clamp(5.,120.),
                _ => 30.,
            };
            let loaded = start_kernel(source,mode,angle)?.await.map_err(KernelError::fault)?;
            self.worker.borrow_mut().stage_scan(id,loaded.graph.clone()).map_err(Fault::host)?;
            let prepared = PreparedScan {
                asset: Arc::new(ScanAsset {
                    source: loaded.source,
                    positions: Arc::new(loaded.render.positions),
                    indices: Arc::new(loaded.render.indices),
                    vertex_count: loaded.graph.vertex_count,
                    triangle_count: loaded.graph.triangle_count,
                    colors: None,
                }),
                view: self.no_view.clone(),
            };
            let operation = (|| {
                let clear = matches!(self.document.store(StoreId::Deviation).get("nominalFromSession"),C::Boolean(true));
                self.worker.borrow_mut().reserve_request().map_err(Fault::host)?;
                self.worker.borrow_mut().commit(
                    Change::Prepared(id),
                    if clear { Change::Clear } else { Change::Unchanged },
                    |_| Ok::<(),()>(()),
                ).map_err(Fault::host)?;
                self.components.set(loaded.components);
                self.document.commit_scan(Some(&prepared))?;
                *self.saved_project.borrow_mut() = None;
                let units = prepared.asset.source.units.unwrap_or(MeshUnits::Mm);
                let prefix = match units {
                    MeshUnits::Mm => "",
                    MeshUnits::Cm => "Read in centimetres and converted to millimetres. ",
                    MeshUnits::M => "Read in metres and converted to millimetres. ",
                    MeshUnits::In => "Read in inches and converted to millimetres. ",
                };
                let note = if loaded.report.skipped == Some(crease::Skipped::Budget) {
                    format!(" Sharp edges are shaded smooth: drawing every edge from {}° on sharp would add more vertices than the scan has — try a larger angle.",port_core::js_number::number_string(angle))
                } else { String::new() };
                patch(&self.document,vec![("statusText",C::String(text(&format!("{prefix}{}Pick an element type in the panel to start measuring.{note}",loaded.file_note))))])
            })();
            prepared.view.dispose()?;
            self.worker.borrow_mut().reserve_request().map_err(Fault::host)?;
            self.worker.borrow_mut().discard(&[id]).map_err(Fault::host)?;
            operation
        }.await;
        let result = if let Err(error) = result {
            let message = match &error {
                Fault::Error { message, .. } => message.clone(),
                Fault::Command(e) => e.message_units().to_vec(),
                _ => text(&format!("{error:?}")),
            };
            patch(
                &self.document,
                vec![("statusText", previous), ("errorText", C::String(message))],
            )?;
            Err(error)
        } else {
            Ok(())
        };
        patch(&self.document, vec![("busy", C::Boolean(false))])?;
        result
    }
}
/// Native engine using the reviewed command/task owner and sole actual scan document.
pub struct Engine {
    state: Rc<State>,
    runtime: Runtime,
    owner: Owner<'static>,
    host: Rc<NativeHost>,
}
impl Engine {
    /// Create a native no-viewport session with the supported original command definitions.
    pub fn new() -> Result<Self, Fault> {
        let link: Rc<RefCell<Option<WeakRuntime>>> = Rc::new(RefCell::new(None));
        let clear = link.clone();
        let document = Session::new(
            None,
            Rc::new(move || {
                clear
                    .borrow()
                    .as_ref()
                    .and_then(WeakRuntime::upgrade)
                    .ok_or_else(|| Fault::host("closed history runtime"))?
                    .clear_history()
            }),
            scan_session::Limits::default(),
        )?;
        let state = Rc::new(State {
            document,
            worker: RefCell::new(Worker::new(crate::scan_worker::Limits::default())),
            imports: ImportQueue::new(QueueLimits::default()),
            components: Cell::new(0),
            no_view: Rc::new(NoView),
            command_marked: RefCell::new(None),
            saved_project: RefCell::new(None),
        });
        let host = Rc::new(NativeHost {
            state: Rc::downgrade(&state),
            timers: RefCell::new(vec![]),
            next_timer: Cell::new(1),
            observer_errors: RefCell::new(vec![]),
        });
        let owner = Owner::new(crate::tasks::Limits {
            tasks: 1024,
            depth: 64,
            polls_per_turn: 4096,
        })
        .map_err(Fault::host)?;
        let mut limits = guarded::Limits::default();
        limits.registry.schema.units = MAX_INPUT_BYTES + 1_000_000;
        limits.registry.schema.work = MAX_INPUT_BYTES * 2 + 2_000_000;
        let runtime = Runtime::new(owner.clone(), host.clone(), limits);
        *link.borrow_mut() = Some(runtime.downgrade());
        let weak = Rc::downgrade(&state);
        runtime.register(
            &text("scan"),
            scan_definition::definition(),
            Callbacks {
                run: Rc::new(move |input, ctx: CommandContext| {
                    let state = weak
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed native document"))?;
                    let Some(Value::Bytes(bytes)) = get(input, "bytes") else {
                        return Err(Fault::host("missing admitted bytes"));
                    };
                    let Some(Value::String(name)) = get(input, "name") else {
                        return Err(Fault::host("missing admitted name"));
                    };
                    let name = String::from_utf16_lossy(name);
                    if name.encode_utf16().count() > 4096 {
                        return Err(Fault::host("native filename limit"));
                    }
                    let units = match get(input, "units") {
                        Some(Value::String(u)) if *u == text("cm") => MeshUnits::Cm,
                        Some(Value::String(u)) if *u == text("m") => MeshUnits::M,
                        Some(Value::String(u)) if *u == text("in") => MeshUnits::In,
                        _ => MeshUnits::Mm,
                    };
                    let discard = matches!(get(input, "discard"), Some(Value::Boolean(true)));
                    require_discardable(&state.document, discard)?;
                    let source = Arc::new(SourceFile {
                        name: text(&name),
                        bytes: Arc::new(bytes.clone()),
                        units: Some(units),
                        companions: None,
                    });
                    let before = state.worker.borrow().versions().scan;
                    let reference = state.document.store(StoreId::Deviation).get("nominalName");
                    let job_state = state.clone();
                    let run = state
                        .imports
                        .run(move || async move { job_state.import(source).await })
                        .map_err(Fault::host)?;
                    ctx.spawn(async move {
                        let result = run.await;
                        if state.worker.borrow().versions().scan == before {
                            match result {
                                Err(QueueError::Callback(Fault::Host(error))) => {
                                    return Err(Fault::Host(error));
                                }
                                Err(QueueError::Callback(Fault::Command(e)))
                                    if e.code() == CommandErrorCode::Unavailable =>
                                {
                                    return Err(Fault::Command(e));
                                }
                                _ => {}
                            }
                            let message = match state.document.store(StoreId::App).get("errorText")
                            {
                                C::String(s) => String::from_utf16_lossy(&s),
                                _ => "The scan could not be opened.".into(),
                            };
                            return Err(command_error(CommandErrorCode::Failed, &message));
                        }
                        let kept = state.document.store(StoreId::Deviation).get("nominalName");
                        let mut out = vec![("scan", state.summary()?)];
                        if matches!(&kept,C::String(s) if !s.is_empty()) {
                            out.push(("referenceKept", cell_value(&kept)));
                        } else if matches!(&reference,C::String(s) if !s.is_empty()) {
                            out.push(("referenceDropped", cell_value(&reference)));
                        }
                        Ok(object(out))
                    })?;
                    Ok(())
                }),
                label: None,
                discards: Some(Rc::new(|input| {
                    Ok(get(input, "discard").cloned().unwrap_or(Value::Undefined))
                })),
            },
        )?;

        selection::register(&runtime, &state)?;
        plane::register(&runtime, &state)?;
        history::register(&runtime, &state, &host)?;
        project::register(&runtime, &state)?;
        Ok(Self {
            state,
            runtime,
            owner,
            host,
        })
    }
    /// Submit to the real registered command owner; responses stay with this receiver.
    pub fn submit(
        &self,
        name: &str,
        input: Value,
        receive: impl FnOnce(Result<Value, Fault>) + 'static,
    ) -> Result<Ticket, Fault> {
        self.runtime
            .submit(text(name), input, None, false, receive)
            .map_err(Fault::host)
    }
    /// Drive ready work/timers on the owning thread. Kernel producers wake it safely.
    pub fn poll(&self) -> Result<(), Fault> {
        self.host.fire_timers()?;
        self.owner.turn().map_err(Fault::host)?;
        Ok(())
    }
    /// Blocking native front-end adapter over submit/poll; geometry remains in the engine.
    pub fn execute(&self, name: &str, input: Value) -> Result<Value, Fault> {
        let result = Rc::new(RefCell::new(None));
        let capture = result.clone();
        let _ticket = self.submit(name, input, move |value| {
            *capture.borrow_mut() = Some(value)
        })?;
        loop {
            self.poll()?;
            if let Some(value) = result.borrow_mut().take() {
                return value;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    /// The actual immutable-view/mutable-owner document, not a mirrored CLI store.
    pub fn document(&self) -> Session {
        self.state.document.clone()
    }
    /// The actual immutable logical mesh retained by the worker owner.
    pub fn logical_graph(&self) -> Option<Arc<MeshGraph>> {
        self.state.worker.borrow().graph()
    }
    /// Inspect the actual worker-owned resources without granting mutation access.
    pub fn inspect_worker<R>(&self, inspect: impl FnOnce(&Worker<MeshGraph, Vec<u8>>) -> R) -> R {
        inspect(&self.state.worker.borrow())
    }
    /// Actual next worker-client request identity, including failed/discard requests.
    pub fn next_request_id(&self) -> u64 {
        self.state.worker.borrow().next_request()
    }
    /// Actual runtime history state; scan import clears this owner directly.
    pub fn history(&self) -> crate::history_record::State {
        self.runtime.history()
    }
    /// Already-computed connected components of the current logical graph.
    pub fn components(&self) -> usize {
        self.state.components.get()
    }
    /// Current source client generations.
    pub fn versions(&self) -> crate::scan_worker::Versions {
        self.state.worker.borrow().versions()
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.host.timers.borrow_mut().clear();
    }
}

#[cfg(test)]
mod native_work_budget_tests {
    use super::*;
    #[test]
    fn dropping_waiters_keeps_work_and_assets_charged_until_producers_finish() {
        let engine = match Engine::new() {
            Ok(engine) => engine,
            Err(e) => panic!("native engine init: {e:?}"),
        };
        let bytes=b"solid budget\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid\n".to_vec();
        assert!(
            engine
                .execute(
                    "scan.open",
                    object(vec![
                        ("bytes", Value::Bytes(bytes)),
                        ("name", value_text("budget.stl"))
                    ])
                )
                .is_ok()
        );
        let original = engine.logical_graph();
        let mut waits = Vec::new();
        let mut release = Vec::new();
        let mut assets = Vec::new();
        for _ in 0..MAX_KERNEL_JOBS {
            let (send, receive) = std::sync::mpsc::channel::<()>();
            let asset = Arc::new(vec![7_u8; 1024]);
            assets.push(Arc::downgrade(&asset));
            let task = start_job("scanruler-budget-check", move || {
                let _asset = asset;
                receive
                    .recv_timeout(Duration::from_secs(5))
                    .map_err(|e| KernelError::Host(format!("{e:?}")))?;
                Ok(())
            });
            let task = match task {
                Ok(task) => task,
                Err(e) => panic!("budget admission failed: {e:?}"),
            };
            waits.push(task);
            release.push(send);
        }
        let refused_before = start_job("scanruler-budget-overflow", || Ok(())).is_err();
        drop(waits);
        let refused_after = start_job("scanruler-dropped-waiter", || Ok(())).is_err();
        let held = assets.iter().all(|a| a.upgrade().is_some());
        let import_refused = matches!(
            engine.execute(
                "scan.open",
                object(vec![
                    ("bytes", Value::Bytes(b"solid budget\nendsolid\n".to_vec())),
                    ("name", value_text("replacement.stl"))
                ])
            ),
            Err(Fault::Host(_))
        );
        let plane_refused = matches!(
            engine.execute(
                "element.fit_marked",
                object(vec![
                    ("kind", value_text("plane")),
                    (
                        "vertices",
                        Value::Array(vec![
                            Value::Number(0.),
                            Value::Number(1.),
                            Value::Number(2.)
                        ])
                    )
                ])
            ),
            Err(Fault::Host(_))
        );
        let closed = matches!(engine.document().store(StoreId::App).get("draft"), C::Null);
        let unchanged = original
            .as_ref()
            .zip(engine.logical_graph().as_ref())
            .is_some_and(|(a, b)| Arc::ptr_eq(a, b));
        let no_history = engine.history().past.is_empty();

        for send in release {
            let _ = send.send(());
        }
        let start = Instant::now();
        while ACTIVE_KERNEL_JOBS.load(std::sync::atomic::Ordering::Acquire) != 0
            && start.elapsed() < Duration::from_secs(5)
        {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            refused_before
                && refused_after
                && held
                && import_refused
                && plane_refused
                && closed
                && unchanged
                && no_history
        );
        assert_eq!(
            ACTIVE_KERNEL_JOBS.load(std::sync::atomic::Ordering::Acquire),
            0
        );
        assert!(assets.iter().all(|a| a.upgrade().is_none()));
    }
}
