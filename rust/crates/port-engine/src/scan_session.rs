// SPDX-License-Identifier: AGPL-3.0-only
//! Actual prepared-scan document ownership and the complete scanImport.commitScan consumer.
//! Parsing, worker staging/render preparation, commands and platform ingress are continuations.
use crate::{guarded::Fault, history_record::Cell as C, schema_value::Text};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};
/// Immutable business fields. History captures share the same composite Cell assets.
pub type Fields = Vec<(Text, C)>;
fn txt(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn s(v: &str) -> C {
    C::String(txt(v))
}
fn n(v: f64) -> C {
    C::Number(v)
}
fn b(v: bool) -> C {
    C::Boolean(v)
}
fn a(v: Vec<C>) -> C {
    C::Array(Arc::new(v))
}
fn o(v: Fields) -> C {
    C::Object(Arc::new(v))
}
fn xyz(v: [f64; 3]) -> C {
    a(v.into_iter().map(n).collect())
}
macro_rules! fields {($($k:literal=>$v:expr),* $(,)?)=>{vec![$((txt($k),$v)),*]};}
/// Bounded native metadata/storage policy, distinct from source behavior.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Fields in each store and each patch.
    pub fields: usize,
    /// Lifetime listener slots in each registry.
    pub listeners: usize,
    /// Listener visits within one reentrant publication chain.
    pub visits: usize,
    /// Reentrant publication/commit depth.
    pub depth: usize,
    /// Maximum render position count.
    pub vertices: usize,
    /// Maximum index count.
    pub indices: usize,
    /// Bytes retained in one source file or companion set.
    pub source_bytes: usize,
    /// Examined owned state values per store/report admission.
    pub cell_nodes: usize,
    /// Owned state text/byte/word units per store/report admission.
    pub cell_units: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            fields: 256,
            listeners: 512,
            visits: 100_000,
            depth: 32,
            vertices: 10_000_000,
            indices: 30_000_000,
            source_bytes: 64 * 1024 * 1024,
            cell_nodes: 1_000_000,
            cell_units: 16 * 1024 * 1024,
        }
    }
}
/// Live source store callback. The previous snapshot is fixed; current may change on reentry.
pub type Listener = Rc<dyn Fn(&Store, Arc<Fields>, Arc<Fields>) -> Result<(), Fault>>;
struct StoreInner {
    current: RefCell<Arc<Fields>>,
    listeners: RefCell<Vec<Option<Listener>>>,
    depth: Cell<usize>,
    visits: Cell<usize>,
    limits: Limits,
}
/// One actual mutable business store; previous roots and composite assets remain immutable.
#[derive(Clone)]
pub struct Store {
    inner: Rc<StoreInner>,
}
/// Non-owning store handle for callback/adapter captures.
#[derive(Clone)]
pub struct WeakStore {
    inner: Weak<StoreInner>,
}
impl WeakStore {
    /// Upgrade while the document or a caller retains the actual store.
    pub fn upgrade(&self) -> Option<Store> {
        self.inner.upgrade().map(|inner| Store { inner })
    }
}
/// Explicit source unsubscribe, also effective against a later add of the same function.
pub struct Release {
    inner: Weak<StoreInner>,
    callback: Listener,
}
impl Release {
    /// Remove this function from the live Set; dropping the handle does not unsubscribe.
    pub fn unsubscribe(&self) {
        if let Some(inner) = self.inner.upgrade() {
            for slot in inner.listeners.borrow_mut().iter_mut() {
                if slot.as_ref().is_some_and(|f| Rc::ptr_eq(f, &self.callback)) {
                    *slot = None;
                    break;
                }
            }
        }
    }
}
struct StoreDepth<'a>(&'a StoreInner);
impl Drop for StoreDepth<'_> {
    fn drop(&mut self) {
        let depth = self.0.depth.get() - 1;
        self.0.depth.set(depth);
        if depth == 0 {
            self.0.visits.set(0)
        }
    }
}
impl Store {
    fn new(data: Fields, limits: Limits) -> Result<Self, Fault> {
        check_fields(&data, limits)?;
        Ok(Self {
            inner: Rc::new(StoreInner {
                current: RefCell::new(Arc::new(data)),
                listeners: RefCell::new(vec![]),
                depth: Cell::new(0),
                visits: Cell::new(0),
                limits,
            }),
        })
    }
    /// A callback-safe non-owning handle to this same store.
    pub fn downgrade(&self) -> WeakStore {
        WeakStore {
            inner: Rc::downgrade(&self.inner),
        }
    }
    /// Current root identity, shared by every observer; never another mutable business document.
    pub fn snapshot(&self) -> Arc<Fields> {
        self.inner.current.borrow().clone()
    }
    /// Read a field, retaining actual composite identity. Missing fields remain undefined.
    pub fn get(&self, key: &str) -> C {
        self.inner
            .current
            .borrow()
            .iter()
            .find(|(k, _)| k.iter().copied().eq(key.encode_utf16()))
            .map_or(C::Undefined, |(_, v)| v.clone())
    }
    /// Check a candidate patch without publication, for prepare-before-commit loads.
    pub(crate) fn preflight(&self, patch: &Fields) -> Result<(), Fault> {
        check_fields(patch, self.inner.limits)?;
        let mut next = (*self.snapshot()).clone();
        for (key, value) in patch {
            if let Some((_, old)) = next.iter_mut().find(|(k, _)| k == key) {
                *old = value.clone()
            } else {
                next.push((key.clone(), value.clone()))
            }
        }
        check_fields(&next, self.inner.limits)
    }
    /// Source partial-object publication. Even an empty partial creates a root and notifies.
    pub fn patch(&self, patch: Fields) -> Result<(), Fault> {
        check_fields(&patch, self.inner.limits)?;
        if self.inner.depth.get() >= self.inner.limits.depth {
            return Err(Fault::host("store depth"));
        }
        let previous = self.snapshot();
        let mut next = Vec::new();
        next.try_reserve(previous.len() + patch.len())
            .map_err(Fault::host)?;
        next.extend(previous.iter().cloned());
        for (key, value) in patch {
            if let Some((_, old)) = next.iter_mut().find(|(k, _)| *k == key) {
                *old = value
            } else {
                next.push((key, value))
            }
        }
        check_fields(&next, self.inner.limits)?;
        *self.inner.current.borrow_mut() = Arc::new(next);
        self.inner.depth.set(self.inner.depth.get() + 1);
        let _depth = StoreDepth(&self.inner);
        let mut index = 0;
        loop {
            let callback = self.inner.listeners.borrow().get(index).cloned();
            let Some(callback) = callback else { break };
            index += 1;
            if let Some(callback) = callback {
                let visits = self.inner.visits.get() + 1;
                if visits > self.inner.limits.visits {
                    return Err(Fault::host("store visits"));
                }
                self.inner.visits.set(visits);
                callback(self, self.snapshot(), previous.clone())?;
            }
        }
        Ok(())
    }
    /// Add by function identity, preserving live insertion/removal iteration.
    pub fn subscribe(&self, callback: Listener) -> Result<Release, Fault> {
        let mut listeners = self.inner.listeners.borrow_mut();
        if !listeners.iter().flatten().any(|f| Rc::ptr_eq(f, &callback)) {
            if listeners.len() >= self.inner.limits.listeners {
                return Err(Fault::host("store listener slots"));
            }
            listeners.try_reserve(1).map_err(Fault::host)?;
            listeners.push(Some(callback.clone()));
        }
        Ok(Release {
            inner: Rc::downgrade(&self.inner),
            callback,
        })
    }
}
fn check_fields(fields: &Fields, limits: Limits) -> Result<(), Fault> {
    if fields.len() > limits.fields {
        return Err(Fault::host("store field budget"));
    }
    let (mut nodes, mut units) = (0usize, 0usize);
    for (i, (key, value)) in fields.iter().enumerate() {
        if fields[..i].iter().any(|(k, _)| k == key) {
            return Err(Fault::host("duplicate owned store key"));
        }
        units = units
            .checked_add(key.len())
            .ok_or_else(|| Fault::host("state text overflow"))?;
        check_cell(value, 0, limits, &mut nodes, &mut units)?;
    }
    Ok(())
}
fn check_cell(
    value: &C,
    depth: usize,
    limits: Limits,
    nodes: &mut usize,
    units: &mut usize,
) -> Result<(), Fault> {
    *nodes = nodes
        .checked_add(1)
        .ok_or_else(|| Fault::host("state nodes overflow"))?;
    if *nodes > limits.cell_nodes || depth > limits.depth {
        return Err(Fault::host("state node/depth budget"));
    }
    let count = match value {
        C::String(v) => v.len(),
        C::Bytes(v) => v.len(),
        C::Words(v) => v
            .len()
            .checked_mul(4)
            .ok_or_else(|| Fault::host("state words overflow"))?,
        C::Array(v) => {
            if v.len() > limits.cell_nodes {
                return Err(Fault::host("state array budget"));
            }
            for c in v.iter() {
                check_cell(c, depth + 1, limits, nodes, units)?;
            }
            0
        }
        C::Object(v) => {
            if v.len() > limits.fields {
                return Err(Fault::host("state object budget"));
            }
            for (i, (k, c)) in v.iter().enumerate() {
                if v[..i].iter().any(|(key, _)| key == k) {
                    return Err(Fault::host("duplicate owned object key"));
                }
                *units = units
                    .checked_add(k.len())
                    .ok_or_else(|| Fault::host("state keys overflow"))?;
                check_cell(c, depth + 1, limits, nodes, units)?;
            }
            0
        }
        _ => 0,
    };
    *units = units
        .checked_add(count)
        .ok_or_else(|| Fault::host("state units overflow"))?;
    if *units > limits.cell_units {
        return Err(Fault::host("state text/bytes budget"));
    }
    Ok(())
}

fn counters() -> C {
    o(
        fields!["point"=>n(1.),"line"=>n(1.),"plane"=>n(1.),"sphere"=>n(1.),"cylinder"=>n(1.),"cone"=>n(1.),"circle"=>n(1.),"torus"=>n(1.)],
    )
}
fn fresh_scan() -> Fields {
    fields!["elements"=>a(vec![]),"draft"=>C::Null,"discarded"=>C::Null,"dimensions"=>a(vec![]),"dimDraft"=>C::Null,"alignDraft"=>C::Null,"sections"=>a(vec![]),"sectionDraft"=>C::Null,"nextSectionNumber"=>n(1.),"appliedAlignment"=>C::Null,"nextNumber"=>n(1.),"nextOfKind"=>counters(),"nextOfDimGroup"=>o(fields!["distance"=>n(1.),"angle"=>n(1.)]),"vertexCount"=>n(0.),"triangleCount"=>n(0.),"scanHidden"=>b(false),"elementsHidden"=>b(false)]
}
fn app_initial() -> Fields {
    let mut v = fields!["projectName"=>C::Null,"fileName"=>C::Null];
    v.extend(fresh_scan());
    v.extend(fields!["modelSize"=>n(1.),"modelCenter"=>xyz([0.;3]),"busy"=>b(false),"working"=>C::Null,"statusText"=>s(""),"progress"=>C::Null,"errorText"=>C::Null,"nextId"=>n(1.),"nextDimensionId"=>n(1.),"settings"=>o(fields!["method"=>s("gaussian"),"sigma"=>n(3.)]),"selectMode"=>s("auto"),"showLabels"=>b(true),"showBackfaces"=>b(true),"translucent"=>b(false),"wireframe"=>b(false),"scanColors"=>b(false),"showScanColors"=>b(true),"navScheme"=>s("cnckitchen"),"viewTheme"=>s("scanner"),"stepStyle"=>s("solids"),"cloudFormat"=>s("ply"),"creaseMode"=>s("on"),"creaseAngle"=>n(30.),"imprintOpen"=>b(false)]);
    v
}
fn no_volume() -> Fields {
    fields!["volume"=>C::Null,"volumeStatus"=>s("idle"),"volumeMessage"=>C::Null,"volumeHover"=>C::Null]
}
fn cleared_deviation() -> Fields {
    fields!["alignStatus"=>s("idle"),"align"=>C::Null,"alignMessage"=>C::Null,"alignStopping"=>b(false),"globalAlign"=>C::Null,"mapStatus"=>s("idle"),"referenceStatus"=>s("idle"),"volume"=>C::Null,"volumeStatus"=>s("idle"),"volumeMessage"=>C::Null,"volumeHover"=>C::Null,"stats"=>C::Null,"histogram"=>C::Null,"probes"=>a(vec![]),"picking"=>b(false),"pairs"=>a(vec![]),"pendingScan"=>C::Null,"marking"=>b(false),"maxDistanceAuto"=>b(true),"animate"=>b(false)]
}
fn no_target() -> Fields {
    fields!["targetId"=>C::Null,"targetSide"=>n(1.),"elementStatus"=>s("idle")]
}
fn deviation_initial() -> Fields {
    let mut v = fields!["source"=>s("reference"),"nominalName"=>C::Null,"nominalTriangles"=>n(0.),"nominalVertices"=>n(0.),"nominalBusy"=>b(false),"nominalStep"=>C::Null,"nominalFromSession"=>b(false),"probes"=>a(vec![]),"nextProbeId"=>n(1.)];
    // Object spread replaces probes in place rather than moving its own-key position.
    for (k, c) in cleared_deviation() {
        if let Some((_, old)) = v.iter_mut().find(|(key, _)| *key == k) {
            *old = c
        } else {
            v.push((k, c))
        }
    }
    v.extend(no_target());
    v.extend(fields!["mapVersion"=>n(0.),"mapOn"=>s("scan"),"referenceVersion"=>n(0.),"showVolume"=>b(true),"elementVersion"=>n(0.),"targetScope"=>s("all"),"scopeCount"=>n(0.),"scopeVersion"=>n(0.),"targetFacingDeg"=>n(60.),"mapFacingDeg"=>n(90.),"showElement"=>b(true),"localMaxDistance"=>n(1.),"range"=>n(1.),"rangeAuto"=>b(true),"maxDistance"=>n(3.),"bands"=>C::Null,"tolerance"=>n(0.1),"showHistogram"=>b(false),"showNominal"=>b(true),"showScan"=>b(true),"nominalSeeThrough"=>b(false),"nominalMesh"=>b(false),"nominalBackfaces"=>b(true),"showMap"=>b(true),"split"=>b(false),"animScale"=>n(10.),"animScaleAuto"=>b(true)]);
    v
}
fn cleared_thickness() -> Fields {
    fields!["status"=>s("idle"),"message"=>C::Null,"stats"=>C::Null,"histogram"=>C::Null,"probes"=>a(vec![])]
}
fn thickness_initial() -> Fields {
    let mut v = fields!["probes"=>a(vec![]),"nextProbeId"=>n(1.)];
    for (k, c) in cleared_thickness() {
        if let Some((_, old)) = v.iter_mut().find(|(key, _)| *key == k) {
            *old = c
        } else {
            v.push((k, c))
        }
    }
    v.extend(fields!["mapVersion"=>n(0.),"method"=>s("ray"),"maxThickness"=>n(10.),"maxThicknessAuto"=>b(true),"coneRays"=>n(0.),"coneAngleDeg"=>n(30.),"normalDeviationDeg"=>n(60.),"low"=>n(0.),"high"=>n(3.),"scaleAuto"=>b(true),"bands"=>C::Null,"limit"=>n(1.),"showHistogram"=>b(true)]);
    v
}
/// Mesh-file units retained with original bytes, independently from millimetre geometry.
#[derive(Clone, Copy, Debug)]
pub enum MeshUnits {
    /// Millimetres.
    Mm,
    /// Centimetres.
    Cm,
    /// Metres.
    M,
    /// Inches.
    In,
}
impl MeshUnits {
    /// Original identifier.
    pub fn id(self) -> &'static str {
        match self {
            Self::Mm => "mm",
            Self::Cm => "cm",
            Self::M => "m",
            Self::In => "in",
        }
    }
}
/// Original companion file, retained for original-compatible project writing.
#[derive(Clone)]
pub struct Companion {
    /// Lossless original filename.
    pub name: Text,
    /// Immutable original bytes.
    pub bytes: Arc<Vec<u8>>,
}
/// Actual immutable model source, never regenerated from render geometry.
#[derive(Clone)]
pub struct SourceFile {
    /// Lossless original filename.
    pub name: Text,
    /// Original retained bytes.
    pub bytes: Arc<Vec<u8>>,
    /// Absence remains distinct from explicit millimetres.
    pub units: Option<MeshUnits>,
    /// Optional companions, including a distinct present empty list.
    pub companions: Option<Vec<Companion>>,
}
/// The session's original files; replacing a scan preserves the image and file reference.
#[derive(Clone, Default)]
pub struct Sources {
    /// Scan source.
    pub scan: Option<Arc<SourceFile>>,
    /// Reference source.
    pub reference: Option<Arc<SourceFile>>,
    /// Image source.
    pub image: Option<Arc<SourceFile>>,
}
/// Real prepared render assets. Worker logical graph/staging and render construction are separate.
#[derive(Clone)]
pub struct ScanAsset {
    /// Immutable source bytes/metadata shared with history/project callers.
    pub source: Arc<SourceFile>,
    /// Owned render positions, including any render copies.
    pub positions: Arc<Vec<f32>>,
    /// Owned render triangle indices.
    pub indices: Arc<Vec<u32>>,
    /// Source logical vertex count, used by no-view bounds.
    pub vertex_count: usize,
    /// Source render triangle count.
    pub triangle_count: usize,
    /// Present empty colours remain truthy in commitScan.
    pub colors: Option<Arc<Vec<u8>>>,
}
/// Owned prepared-view service. commitScan never calls dispose; the import finally owns it.
pub trait PreparedView {
    /// Install the prepared view.
    fn commit(&self) -> Result<(), Fault>;
    /// Release the preparation handle.
    fn dispose(&self) -> Result<(), Fault>;
}
/// A prepared asset and its original view lifetime provider.
#[derive(Clone)]
pub struct PreparedScan {
    /// Immutable real geometry/source data.
    pub asset: Arc<ScanAsset>,
    /// Owned provider, including the explicit no-viewport view.
    pub view: Rc<dyn PreparedView>,
}
/// Optional viewport service. Its metrics have the real scene's finite radius floor of 1e-4.
pub trait Scene {
    /// Clear the preview region after its generation advances.
    fn clear_preview_region(&self) -> Result<(), Fault>;
    /// Clear the preview geometry.
    fn clear_preview_shape(&self) -> Result<(), Fault>;
    /// Remove a session-derived nominal.
    fn clear_nominal(&self) -> Result<(), Fault>;
    /// Remove the displayed scan.
    fn clear_scan(&self) -> Result<(), Fault>;
    /// Real viewport model size.
    fn model_size(&self) -> Result<f64, Fault>;
    /// Real viewport model centre.
    fn model_center(&self) -> Result<[f64; 3], Fault>;
}
/// Native cache payloads: actual immutable arrays, not duplicate mutable measurement state.
#[derive(Clone)]
pub enum MapValue {
    /// Float field buffer.
    Floats(Arc<Vec<f32>>),
    /// Colour buffer.
    Bytes(Arc<Vec<u8>>),
    /// Region buffer.
    Words(Arc<Vec<u32>>),
    /// Owned result metadata from another measurement family.
    Report(C),
}
const MAP_KEYS: [&str; 9] = [
    "deviation",
    "deviationRgb",
    "elementField",
    "elementRgb",
    "elementScope",
    "thickness",
    "thicknessRgb",
    "referenceMap",
    "referenceRgb",
];
/// Source scanLoaded payload: the prepared buffer identities and actual size.
#[derive(Clone)]
pub struct Loaded {
    /// Actual position asset.
    pub positions: Arc<Vec<f32>>,
    /// Actual index asset.
    pub indices: Arc<Vec<u32>>,
    /// Model size used by this commit.
    pub model_size: f64,
}
/// Source listener identity. Emission snapshots the listener array, not its mutable business data.
pub type LoadedListener = Rc<dyn Fn(&Session, Option<Loaded>) -> Result<(), Fault>>;
struct Core {
    app: Store,
    deviation: Store,
    thickness: Store,
    mark: Store,
    surfaces: Store,
    sources: RefCell<Sources>,
    asset: RefCell<Option<Arc<ScanAsset>>>,
    maps: RefCell<Vec<Option<MapValue>>>,
    regions: RefCell<Vec<(u64, Arc<Vec<u32>>)>>,
    preview_seq: Cell<f64>,
    preview_region: RefCell<Option<Arc<Vec<u32>>>>,
    scene: Option<Rc<dyn Scene>>,
    history_clear: Rc<dyn Fn() -> Result<(), Fault>>,
    loaded: RefCell<Vec<Option<LoadedListener>>>,
    limits: Limits,
    depth: Cell<usize>,
}
/// Single owner of actual scan business data and immutable assets. Runtime metadata stays external.
#[derive(Clone)]
pub struct Session {
    core: Rc<Core>,
}
/// Non-owning document handle for viewport/import/listener adapters.
#[derive(Clone)]
pub struct WeakSession {
    core: Weak<Core>,
}
impl WeakSession {
    /// Upgrade only while an external owner retains the document.
    pub fn upgrade(&self) -> Option<Session> {
        self.core.upgrade().map(|core| Session { core })
    }
}
/// Exact scan-listener removal. Duplicate registrations of one identity remain separate entries.
pub struct LoadedRelease {
    core: Weak<Core>,
    callback: LoadedListener,
}
impl LoadedRelease {
    /// Remove the first remaining registration of this identity.
    pub fn unsubscribe(&self) {
        if let Some(core) = self.core.upgrade() {
            for slot in core.loaded.borrow_mut().iter_mut() {
                if slot.as_ref().is_some_and(|f| Rc::ptr_eq(f, &self.callback)) {
                    *slot = None;
                    break;
                }
            }
        }
    }
}
/// Actual store identifiers within the scan document.
#[derive(Clone, Copy)]
pub enum StoreId {
    /// Measurement/session state.
    App,
    /// Deviation/reference state.
    Deviation,
    /// Thickness state.
    Thickness,
    /// Shared marking tools.
    Mark,
    /// Surface version state.
    Surfaces,
}
struct CommitDepth<'a>(&'a Cell<usize>);
impl Drop for CommitDepth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl Session {
    /// Fresh real document. history_clear must call the session runtime's actual clear_history;
    /// capture/apply/restore and worker command preparation remain explicit later integrations.
    pub fn new(
        scene: Option<Rc<dyn Scene>>,
        history_clear: Rc<dyn Fn() -> Result<(), Fault>>,
        limits: Limits,
    ) -> Result<Self, Fault> {
        Ok(Self {
            core: Rc::new(Core {
                app: Store::new(app_initial(), limits)?,
                deviation: Store::new(deviation_initial(), limits)?,
                thickness: Store::new(thickness_initial(), limits)?,
                mark: Store::new(
                    fields!["gesture"=>C::Null,"erase"=>b(false),"backfaces"=>b(false),"diameter"=>n(2.),"count"=>n(0.),"inverts"=>n(0.)],
                    limits,
                )?,
                surfaces: Store::new(fields!["version"=>n(0.)], limits)?,
                sources: RefCell::new(Sources::default()),
                asset: RefCell::new(None),
                maps: RefCell::new(vec![None; 9]),
                regions: RefCell::new(vec![]),
                preview_seq: Cell::new(0.),
                preview_region: RefCell::new(None),
                scene,
                history_clear,
                loaded: RefCell::new(vec![]),
                limits,
                depth: Cell::new(0),
            }),
        })
    }
    /// A non-owning handle for services installed in this document itself.
    pub fn downgrade(&self) -> WeakSession {
        WeakSession {
            core: Rc::downgrade(&self.core),
        }
    }
    /// Access the sole actual store, with immutable snapshot/asset identity.
    pub fn store(&self, id: StoreId) -> Store {
        match id {
            StoreId::App => &self.core.app,
            StoreId::Deviation => &self.core.deviation,
            StoreId::Thickness => &self.core.thickness,
            StoreId::Mark => &self.core.mark,
            StoreId::Surfaces => &self.core.surfaces,
        }
        .clone()
    }
    /// Current immutable source references.
    pub fn sources(&self) -> Sources {
        self.core.sources.borrow().clone()
    }
    /// Set owned source references during project/import orchestration.
    pub fn set_sources(&self, sources: Sources) -> Result<(), Fault> {
        for f in [&sources.scan, &sources.reference, &sources.image]
            .into_iter()
            .flatten()
        {
            self.check_source(f)?
        }
        *self.core.sources.borrow_mut() = sources;
        Ok(())
    }
    /// Current prepared asset retained by the document, without copying its arrays.
    pub fn asset(&self) -> Option<Arc<ScanAsset>> {
        self.core.asset.borrow().clone()
    }
    /// All actual per-vertex caches, in original session insertion order.
    pub fn maps(&self) -> Vec<(&'static str, Option<MapValue>)> {
        MAP_KEYS
            .into_iter()
            .zip(self.core.maps.borrow().iter().cloned())
            .collect()
    }
    /// Assign a bounded owned cache produced by a measurement family.
    pub fn set_map(&self, key: &str, value: Option<MapValue>) -> Result<(), Fault> {
        let Some(i) = MAP_KEYS.iter().position(|k| *k == key) else {
            return Err(Fault::host("unknown scan map"));
        };
        if let Some(value) = &value {
            match value {
                MapValue::Floats(v) if v.len() > self.core.limits.vertices => {
                    return Err(Fault::host("float map budget"));
                }
                MapValue::Bytes(v) if v.len() > self.core.limits.vertices.saturating_mul(3) => {
                    return Err(Fault::host("colour map budget"));
                }
                MapValue::Words(v) if v.len() > self.core.limits.vertices => {
                    return Err(Fault::host("region map budget"));
                }
                MapValue::Report(v) => {
                    let (mut nodes, mut units) = (0, 0);
                    check_cell(v, 0, self.core.limits, &mut nodes, &mut units)?;
                }
                _ => {}
            }
        }
        *self
            .core
            .maps
            .borrow_mut()
            .get_mut(i)
            .ok_or_else(|| Fault::host("map contract"))? = value;
        Ok(())
    }
    /// Retained fitted surface regions, preserving original Map insertion order.
    pub fn regions(&self) -> Vec<(u64, Arc<Vec<u32>>)> {
        self.core.regions.borrow().clone()
    }
    /// Store an actual element's immutable surface and advance its shared version.
    pub fn remember_surface(&self, id: u64, vertices: Arc<Vec<u32>>) -> Result<(), Fault> {
        if id > 9_007_199_254_740_991 || vertices.len() > self.core.limits.vertices {
            return Err(Fault::host("surface bounds"));
        }
        let mut regions = self.core.regions.borrow_mut();
        if let Some((_, v)) = regions.iter_mut().find(|(i, _)| *i == id) {
            *v = vertices
        } else {
            if regions.len() >= self.core.limits.fields {
                return Err(Fault::host("surface slots"));
            }
            regions.try_reserve(1).map_err(Fault::host)?;
            regions.push((id, vertices))
        }
        drop(regions);
        self.bump_surfaces()
    }
    fn bump_surfaces(&self) -> Result<(), Fault> {
        let C::Number(version) = self.core.surfaces.get("version") else {
            return Err(Fault::host("surface version contract"));
        };
        self.core.surfaces.patch(fields!["version"=>n(version+1.)])
    }
    /// Actual preview generation/region, used to reject a result from an earlier draft.
    pub fn preview(&self) -> (f64, Option<Arc<Vec<u32>>>) {
        (
            self.core.preview_seq.get(),
            self.core.preview_region.borrow().clone(),
        )
    }
    /// Keep an actual preview region without copying its immutable vertex asset.
    pub fn set_preview_region(&self, region: Option<Arc<Vec<u32>>>) -> Result<(), Fault> {
        if region
            .as_ref()
            .is_some_and(|v| v.len() > self.core.limits.vertices)
        {
            return Err(Fault::host("preview vertices"));
        }
        *self.core.preview_region.borrow_mut() = region;
        Ok(())
    }
    /// Complete measureWorkspace.clearPreview at the owned provider boundary.
    pub fn clear_preview(&self) -> Result<(), Fault> {
        self.core.preview_seq.set(self.core.preview_seq.get() + 1.);
        *self.core.preview_region.borrow_mut() = None;
        if let Some(scene) = &self.core.scene {
            scene.clear_preview_region()?;
            scene.clear_preview_shape()?;
        }
        Ok(())
    }
    /// Register a real scan-loaded provider; dropping its release is not unsubscribe.
    pub fn on_loaded(&self, callback: LoadedListener) -> Result<LoadedRelease, Fault> {
        let mut listeners = self.core.loaded.borrow_mut();
        if listeners.len() >= self.core.limits.listeners {
            return Err(Fault::host("loaded listener slots"));
        }
        listeners.try_reserve(1).map_err(Fault::host)?;
        listeners.push(Some(callback.clone()));
        Ok(LoadedRelease {
            core: Rc::downgrade(&self.core),
            callback,
        })
    }
    fn loaded(&self, scan: Option<Loaded>) -> Result<(), Fault> {
        let listeners = self
            .core
            .loaded
            .borrow()
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        for callback in listeners {
            callback(self, scan.clone())?
        }
        Ok(())
    }
    fn clear_maps(&self, keys: &[&str]) {
        let mut maps = self.core.maps.borrow_mut();
        for key in keys {
            if let Some(i) = MAP_KEYS.iter().position(|k| k == key) {
                maps[i] = None
            }
        }
    }
    fn check_source(&self, source: &SourceFile) -> Result<(), Fault> {
        let mut bytes = source.bytes.len();
        let mut names = source.name.len();
        if let Some(companions) = &source.companions {
            if companions.len() > self.core.limits.fields {
                return Err(Fault::host("source companion slots"));
            }
            for f in companions {
                names = names
                    .checked_add(f.name.len())
                    .ok_or_else(|| Fault::host("source names overflow"))?;
                bytes = bytes
                    .checked_add(f.bytes.len())
                    .ok_or_else(|| Fault::host("source bytes overflow"))?;
            }
        }
        if bytes > self.core.limits.source_bytes || names > self.core.limits.cell_units {
            return Err(Fault::host("source budget"));
        }
        Ok(())
    }
    /// Full source commitScan body. Pre-admitted assets are immutable; failures retain source
    /// publication/clear/view/listener order rather than rolling back already-published state.
    pub fn commit_scan(&self, prepared: Option<&PreparedScan>) -> Result<(), Fault> {
        if self.core.depth.get() >= self.core.limits.depth {
            return Err(Fault::host("commit depth"));
        }
        if let Some(p) = prepared {
            self.check_source(&p.asset.source)?;
            if p.asset.positions.len() / 3 > self.core.limits.vertices
                || p.asset.vertex_count > p.asset.positions.len() / 3
                || !p.asset.positions.len().is_multiple_of(3)
                || p.asset.indices.len() > self.core.limits.indices
                || p.asset.positions.iter().any(|v| !v.is_finite())
                || p.asset
                    .indices
                    .iter()
                    .any(|&v| v as usize >= p.asset.positions.len() / 3)
                || p.asset.triangle_count != p.asset.indices.len() / 3
                || !p.asset.indices.len().is_multiple_of(3)
                || p.asset
                    .colors
                    .as_ref()
                    .is_some_and(|v| v.len() > self.core.limits.vertices.saturating_mul(3))
            {
                return Err(Fault::host("prepared asset bounds"));
            }
        }
        self.core.depth.set(self.core.depth.get() + 1);
        let _depth = CommitDepth(&self.core.depth);
        let app = self.core.app.clone();
        self.clear_preview()?;
        if matches!(
            self.core.deviation.get("nominalFromSession"),
            C::Boolean(true)
        ) {
            if let Some(scene) = &self.core.scene {
                scene.clear_nominal()?
            }
            self.core.sources.borrow_mut().reference = None;
            self.core.deviation.patch(fields!["nominalBusy"=>b(false),"nominalName"=>C::Null,"nominalVertices"=>n(0.),"nominalTriangles"=>n(0.),"nominalStep"=>C::Null,"nominalFromSession"=>b(false)])?;
        }
        self.clear_maps(&["deviation", "deviationRgb", "referenceMap", "referenceRgb"]);
        let mut reference = fields!["referenceStatus"=>s("idle")];
        reference.extend(no_volume());
        self.core.deviation.patch(reference)?;
        self.clear_maps(&["elementField", "elementRgb", "elementScope"]);
        self.core.regions.borrow_mut().clear();
        self.bump_surfaces()?;
        self.clear_maps(&["thickness", "thicknessRgb"]);
        self.core.deviation.patch(cleared_deviation())?;
        let mut target = no_target();
        if matches!(self.core.deviation.get("source"),C::String(v) if v==txt("element")) {
            target.extend(fields!["stats"=>C::Null,"histogram"=>C::Null,"probes"=>a(vec![])])
        }
        self.core.deviation.patch(target)?;
        let C::Number(scope_version) = self.core.deviation.get("scopeVersion") else {
            return Err(Fault::host("scope version contract"));
        };
        self.core.deviation.patch(fields!["targetScope"=>s("all"),"scopeCount"=>n(0.),"scopeVersion"=>n(scope_version+1.)])?;
        self.core.thickness.patch(cleared_thickness())?;
        self.core
            .mark
            .patch(fields!["gesture"=>C::Null,"erase"=>b(false),"count"=>n(0.)])?;
        self.core.sources.borrow_mut().scan = prepared.map(|p| p.asset.source.clone());
        *self.core.asset.borrow_mut() = prepared.map(|p| p.asset.clone());
        let mut begin = fields!["busy"=>b(true),"fileName"=>C::String(prepared.map_or_else(Vec::new,|p|p.asset.source.name.clone())),"statusText"=>s("Reading file…"),"errorText"=>C::Null];
        begin.extend(fresh_scan());
        app.patch(begin)?;
        app.patch(fields!["scanColors"=>b(prepared.is_some_and(|p|p.asset.colors.is_some()))])?;
        if let Some(prepared) = prepared {
            prepared.view.commit()?;
            let asset = &prepared.asset;
            let (size, center) = if let Some(scene) = &self.core.scene {
                let size = scene.model_size()?;
                let center = scene.model_center()?;
                (size, center)
            } else {
                let bounds = port_core::bounds::bounds_of(&asset.positions, asset.vertex_count)
                    .map_err(Fault::host)?;
                (bounds.radius, bounds.center)
            };
            if !size.is_finite() || size < 1e-4 {
                return Err(Fault::host("scene metrics outside owned bounds contract"));
            }
            app.patch(fields!["busy"=>b(false),"vertexCount"=>n(asset.vertex_count as f64),"triangleCount"=>n(asset.triangle_count as f64),"modelSize"=>n(size),"modelCenter"=>xyz(center)])?;
            self.core
                .mark
                .patch(fields!["diameter"=>n((size*0.08).max(2e-3))])?;
            let thickness = if matches!(
                self.core.thickness.get("maxThicknessAuto"),
                C::Boolean(true)
            ) {
                fields!["maxThickness"=>n(nice_ceil(2.*size*0.2)?)]
            } else {
                vec![]
            };
            self.core.thickness.patch(thickness)?;
            self.loaded(Some(Loaded {
                positions: asset.positions.clone(),
                indices: asset.indices.clone(),
                model_size: size,
            }))?;
        } else {
            if let Some(scene) = &self.core.scene {
                scene.clear_scan()?
            }
            app.patch(fields!["fileName"=>C::Null,"modelSize"=>n(1.),"modelCenter"=>xyz([0.;3])])?;
            self.loaded(None)?;
        }
        app.patch(fields!["busy"=>b(true)])?;
        (self.core.history_clear)()?;
        Ok(())
    }
}
// Complete niceCeil/scaleByDecade for the actual model-size domain: size>=1e-4,
// so fractional counts are 0..6. Preserve toFixed's exact rational tie decision;
// Rust's ties-to-even formatting is not the JavaScript operation.
fn nice_ceil(x: f64) -> Result<f64, Fault> {
    if x <= 0. || x.is_nan() {
        return Ok(0.1);
    }
    let exp = x.log10().floor();
    let m = x / 10_f64.powf(exp);
    let step = [1., 1.2, 1.5, 2., 2.5, 3., 4., 5., 6., 8., 10.]
        .into_iter()
        .find(|step| m <= *step + 1e-9)
        .unwrap_or(10.);
    let v = if exp >= 0. {
        step * 10_f64.powf(exp)
    } else {
        step / 10_f64.powf(-exp)
    };
    let digits = (1. - exp).clamp(0., 100.) as u32;
    if !v.is_finite() || v >= 1e21 {
        return Ok(v);
    }
    if digits > 6 {
        return Err(Fault::host("model scale outside bounded fixed domain"));
    }
    let bits = v.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let (mantissa, power) = if exponent == 0 {
        (bits & ((1u64 << 52) - 1), -1074 + digits as i32)
    } else {
        (
            (bits & ((1u64 << 52) - 1)) | (1u64 << 52),
            exponent - 1023 - 52 + digits as i32,
        )
    };
    let numerator = u128::from(mantissa) * 5u128.pow(digits);
    let rounded = if power >= 0 {
        numerator
            .checked_shl(power as u32)
            .ok_or_else(|| Fault::host("fixed scale overflow"))?
    } else {
        let shift = (-power) as u32;
        if shift >= 128 {
            0
        } else {
            let whole = numerator >> shift;
            let rem = numerator & ((1u128 << shift) - 1);
            whole + u128::from(rem >= (1u128 << (shift - 1)))
        }
    };
    let mut text = rounded.to_string();
    if digits > 0 {
        let digits = digits as usize;
        if text.len() <= digits {
            text = format!("0.{}{text}", "0".repeat(digits - text.len()))
        } else {
            text.insert(text.len() - digits, '.')
        }
    }
    text.parse::<f64>().map_err(Fault::host)
}
