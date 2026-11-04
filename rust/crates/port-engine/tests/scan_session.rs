// SPDX-License-Identifier: AGPL-3.0-only
//! Independent real scan document assets/stores, paired with the frozen complete consumer.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_engine::scan_session;
use port_engine::{
    guarded::{self, Callbacks, Fault, Host, Runtime, WeakRuntime},
    history_record::{Cell as C, Compare, Section, Snapshot, State},
    schema_value::Value as V,
    tasks::{Limits as TaskLimits, Owner},
};
#[path = "support/scan_worker_profiles.rs"]
mod worker_profiles;
use scan_session::{
    MapValue, MeshUnits, PreparedScan, PreparedView, ScanAsset, Scene, Session, SourceFile,
    Sources, StoreId,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};
const INPUT: &str =
    include_str!("../../../../docs/port-plan/fixtures/scan-session-commit/cases.tsv");
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn st(s: &str) -> V {
    V::String(text(s))
}
fn num(n: f64) -> V {
    V::Number(n)
}
fn boolv(v: bool) -> V {
    V::Boolean(v)
}
fn array(v: Vec<V>) -> V {
    V::Array(v)
}
fn obj(v: Vec<(&str, V)>) -> V {
    V::Object(v.into_iter().map(|(k, v)| (text(k), v)).collect())
}
fn xyz(v: [f64; 3]) -> V {
    array(v.into_iter().map(num).collect())
}
macro_rules! fields {($($k:literal=>$v:expr),* $(,)?)=>{vec![$((text($k),cell($v))),*]};}
fn cell(v: V) -> C {
    match v {
        V::Undefined => C::Undefined,
        V::Null => C::Null,
        V::Boolean(v) => C::Boolean(v),
        V::Number(v) => C::Number(v),
        V::String(v) => C::String(v),
        V::Array(v) => C::Array(Arc::new(v.into_iter().map(cell).collect())),
        V::Object(v) => C::Object(Arc::new(v.into_iter().map(|(k, v)| (k, cell(v))).collect())),
        V::Bytes(v) => C::Bytes(Arc::new(v)),
    }
}
fn value(v: &C) -> V {
    match v {
        C::Undefined => V::Undefined,
        C::Null => V::Null,
        C::Boolean(v) => boolv(*v),
        C::Number(v) => num(*v),
        C::String(v) => V::String(v.clone()),
        C::Array(v) => array(v.iter().map(value).collect()),
        C::Object(v) => field_value(v),
        C::Bytes(v) => V::Bytes(v.as_ref().clone()),
        C::Words(v) => array(v.iter().map(|v| num(f64::from(*v))).collect()),
    }
}
fn field_value(v: &[(Vec<u16>, C)]) -> V {
    V::Object(v.iter().map(|(k, v)| (k.clone(), value(v))).collect())
}
fn wire(value: &V) -> String {
    fn count(out: &mut Vec<u8>, n: usize) {
        out.extend_from_slice(&u32::try_from(n).unwrap().to_le_bytes())
    }
    fn txt(out: &mut Vec<u8>, s: &[u16]) {
        count(out, s.len());
        for c in s {
            out.extend_from_slice(&c.to_le_bytes())
        }
    }
    fn walk(out: &mut Vec<u8>, v: &V) {
        match v {
            V::Undefined => out.push(0),
            V::Null => out.push(1),
            V::Boolean(b) => out.push(if *b { 3 } else { 2 }),
            V::Number(n) => {
                out.push(4);
                out.extend_from_slice(&n.to_bits().to_le_bytes())
            }
            V::String(s) => {
                out.push(5);
                txt(out, s)
            }
            V::Array(items) => {
                out.push(6);
                count(out, items.len());
                for v in items {
                    walk(out, v)
                }
            }
            V::Object(fields) => {
                out.push(7);
                count(out, fields.len());
                for (k, v) in fields {
                    txt(out, k);
                    walk(out, v)
                }
            }
            V::Bytes(b) => {
                out.push(8);
                count(out, b.len());
                out.extend_from_slice(b)
            }
        }
    }
    let mut out = Vec::new();
    walk(&mut out, value);
    out.iter().map(|n| format!("{n:02x}")).collect()
}

fn source(file: &SourceFile) -> V {
    let mut f = vec![
        (text("name"), V::String(file.name.clone())),
        (text("bytes"), V::Bytes(file.bytes.as_ref().clone())),
    ];
    if let Some(u) = file.units {
        f.push((text("units"), st(u.id())))
    }
    if let Some(files) = &file.companions {
        f.push((
            text("companions"),
            array(
                files
                    .iter()
                    .map(|f| {
                        obj(vec![
                            ("name", V::String(f.name.clone())),
                            ("bytes", V::Bytes(f.bytes.as_ref().clone())),
                        ])
                    })
                    .collect(),
            ),
        ))
    }
    V::Object(f)
}
fn sources(session: &Session) -> V {
    let f = session.sources();
    obj(vec![
        ("scan", f.scan.as_ref().map_or(V::Null, |f| source(f))),
        (
            "reference",
            f.reference.as_ref().map_or(V::Null, |f| source(f)),
        ),
        ("image", f.image.as_ref().map_or(V::Null, |f| source(f))),
    ])
}
fn maps(session: &Session) -> V {
    obj(session
        .maps()
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                match v {
                    None => V::Null,
                    Some(MapValue::Floats(v)) => {
                        array(v.iter().map(|v| num(f64::from(*v))).collect())
                    }
                    Some(MapValue::Bytes(v)) => V::Bytes(v.as_ref().clone()),
                    Some(MapValue::Words(v)) => {
                        array(v.iter().map(|v| num(f64::from(*v))).collect())
                    }
                    Some(MapValue::Report(v)) => value(&v),
                },
            )
        })
        .collect())
}
fn regions(session: &Session) -> V {
    array(
        session
            .regions()
            .iter()
            .map(|(id, ids)| {
                array(vec![
                    num(*id as f64),
                    array(ids.iter().map(|v| num(f64::from(*v))).collect()),
                ])
            })
            .collect(),
    )
}
fn history(state: &State) -> V {
    let entries = |entries: &[port_engine::history_record::Entry]| {
        array(
            entries
                .iter()
                .map(|e| {
                    obj(vec![
                        ("before", field_value(&e.before)),
                        ("after", field_value(&e.after)),
                        ("label", V::String(e.label.clone())),
                        ("workspace", V::String(e.workspace.clone())),
                        ("group", num(e.group as f64)),
                    ])
                })
                .collect(),
        )
    };
    obj(vec![
        ("past", entries(&state.past)),
        ("future", entries(&state.future)),
        ("restoring", boolv(state.restoring)),
        ("importing", boolv(state.importing)),
        ("revision", num(state.revision as f64)),
    ])
}
fn file(name: &str, bytes: Vec<u8>, units: Option<MeshUnits>) -> Arc<SourceFile> {
    Arc::new(SourceFile {
        name: text(name),
        bytes: Arc::new(bytes),
        units,
        companions: None,
    })
}
struct Env {
    profile: [usize; 9],
    enabled: Cell<bool>,
    failed: Cell<bool>,
    reentered: Cell<bool>,
    mutated: Cell<bool>,
    secondary: RefCell<Option<scan_session::Release>>,
    loaded_release: RefCell<Option<scan_session::LoadedRelease>>,
    events: RefCell<Vec<V>>,
    model: RefCell<Option<Session>>,
    runtime: Rc<RefCell<Option<WeakRuntime>>>,
    initial_app: RefCell<Option<Arc<scan_session::Fields>>>,
}
impl Env {
    fn model(&self) -> Session {
        self.model.borrow().as_ref().unwrap().clone()
    }
    fn log(&self, label: &str) {
        self.events.borrow_mut().push(array(vec![st(label)]))
    }
    fn fail(&self, n: usize, label: &str) -> Result<(), Fault> {
        if self.enabled.get() && self.profile[6] == n && !self.failed.replace(true) {
            Err(Fault::error(&format!("{label} fault")))
        } else {
            Ok(())
        }
    }
    fn history(&self) -> V {
        history(
            &self
                .runtime
                .borrow()
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .history(),
        )
    }
    fn frame(&self, error: &V) -> String {
        let m = self.model();
        let preview = m.preview();
        wire(&array(vec![
            error.clone(),
            field_value(&m.store(StoreId::App).snapshot()),
            field_value(&m.store(StoreId::Deviation).snapshot()),
            field_value(&m.store(StoreId::Thickness).snapshot()),
            field_value(&m.store(StoreId::Mark).snapshot()),
            field_value(&m.store(StoreId::Surfaces).snapshot()),
            sources(&m),
            maps(&m),
            regions(&m),
            self.history(),
            num(preview.0),
            preview.1.map_or(V::Null, |v| {
                array(v.iter().map(|v| num(f64::from(*v))).collect())
            }),
            array(self.events.borrow().clone()),
        ]))
    }
}
struct SceneEnv(Weak<Env>);
impl Scene for SceneEnv {
    fn clear_preview_region(&self) -> Result<(), Fault> {
        let e = self.0.upgrade().unwrap();
        e.log("preview-region");
        e.fail(1, "preview")
    }
    fn clear_preview_shape(&self) -> Result<(), Fault> {
        self.0.upgrade().unwrap().log("preview");
        Ok(())
    }
    fn clear_nominal(&self) -> Result<(), Fault> {
        let e = self.0.upgrade().unwrap();
        e.log("clear-nominal");
        e.fail(2, "nominal")
    }
    fn clear_scan(&self) -> Result<(), Fault> {
        self.0.upgrade().unwrap().log("clear-scan");
        Ok(())
    }
    fn model_size(&self) -> Result<f64, Fault> {
        let e = self.0.upgrade().unwrap();
        e.log("model-size");
        e.fail(4, "size")?;
        Ok(7. + e.profile[0] as f64)
    }
    fn model_center(&self) -> Result<[f64; 3], Fault> {
        let e = self.0.upgrade().unwrap();
        e.log("model-center");
        e.fail(5, "center")?;
        Ok([4., 5., 6.])
    }
}
struct ViewEnv(Weak<Env>);
impl PreparedView for ViewEnv {
    fn commit(&self) -> Result<(), Fault> {
        let e = self.0.upgrade().unwrap();
        e.log("view-commit");
        e.fail(3, "view")
    }
    fn dispose(&self) -> Result<(), Fault> {
        self.0.upgrade().unwrap().log("view-dispose");
        Ok(())
    }
}
struct CaptureEnv(Weak<Env>);
impl Host for CaptureEnv {
    fn busy(&self) -> Result<Option<Vec<u16>>, Fault> {
        Ok(None)
    }
    fn workspace(&self) -> Result<Vec<u16>, Fault> {
        Ok(text("elements"))
    }
    fn capture(&self) -> Result<Snapshot, Fault> {
        let m = self.0.upgrade().unwrap().model();
        let app = m.store(StoreId::App);
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
        Ok(0.)
    }
    fn stale_now(&self) -> Result<f64, Fault> {
        Ok(0.)
    }
    fn timer(&self, _: f64, _: Rc<dyn Fn() -> Result<(), Fault>>) -> u64 {
        1
    }
    fn clear_timer(&self, _: u64) {}
    fn settle(&self, _: &[u16]) -> Result<port_engine::settle::Work<'static, Fault>, Fault> {
        Ok(Box::pin(std::future::ready(Ok(()))))
    }
    fn settle_readonly(
        &self,
        _: &[u16],
    ) -> Result<Option<port_engine::settle::Work<'static, Fault>>, Fault> {
        Ok(None)
    }
    fn observer_fault(&self, e: Fault) {
        panic!("unexpected observer {e:?}")
    }
    fn stop(&self, _: port_engine::execute::Done) -> port_engine::settle::Work<'static, Fault> {
        Box::pin(std::future::ready(Ok(())))
    }
}
fn point() -> V {
    obj(vec![
        ("id", num(41.)),
        ("kind", st("point")),
        ("name", st("Held point")),
        ("color", st("#123456")),
        ("source", obj(vec![("type", st("picked"))])),
        ("status", st("done")),
        ("visible", boolv(true)),
        (
            "fit",
            obj(vec![
                ("kind", st("point")),
                ("center", xyz([1., 2., 3.])),
                ("sigma", num(0.)),
                ("usedPoints", num(1.)),
                ("regionSize", num(1.)),
            ]),
        ),
    ])
}
fn extra_app(weak: Weak<Env>, label: &'static str) -> scan_session::Listener {
    Rc::new(move |store, current, previous| {
        let e = weak.upgrade().unwrap();
        e.events.borrow_mut().push(array(vec![
            st("app-extra"),
            st(label),
            field_value(&current),
            field_value(&previous),
            boolv(Arc::ptr_eq(&current, &store.snapshot())),
        ]));
        Ok(())
    })
}
fn prepared(env: &Rc<Env>, mode: usize) -> Option<PreparedScan> {
    if mode == 0 {
        return None;
    }
    let mut p = match mode {
        1 => vec![0., 0., 0., 2., 0., 0., 0., 3., 0.],
        2 => vec![-1., -2., -3., 1., -2., -3., 1., 2., -3., -1., 2., 3.],
        _ => vec![0., 0., 0.],
    };
    let seed = env.profile[8];
    let mut state = seed as u32;
    if seed != 0 {
        for p in &mut p {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *p = ((f64::from(state % 2001) - 1000.) / 7.) as f32;
        }
    }
    let indices = match mode {
        1 => vec![0, 1, 2],
        2 => vec![0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2],
        _ => vec![],
    };
    let vc = p.len() / 3;
    let tc = indices.len() / 3;
    let vertices = p
        .as_chunks::<3>()
        .0
        .iter()
        .map(|xyz| {
            xyz.iter()
                .map(|v| port_core::js_number::number_string(f64::from(*v)))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let faces = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|v| format!("3 {} {} {}", v[0], v[1], v[2]))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "ply\nformat ascii 1.0\nelement vertex {vc}\nproperty float x\nproperty float y\nproperty float z\nelement face {tc}\nproperty list uchar int vertex_indices\nend_header\n{vertices}\n{faces}\n"
    );
    let mut name = text(&format!("scan{mode}"));
    name.push(0xd800);
    name.extend(text(".ply"));
    let source = Arc::new(SourceFile {
        name,
        bytes: Arc::new(body.into_bytes()),
        units: Some(MeshUnits::Mm),
        companions: None,
    });
    Some(PreparedScan {
        asset: Arc::new(ScanAsset {
            source,
            positions: Arc::new(p),
            indices: Arc::new(indices),
            vertex_count: vc,
            triangle_count: tc,
            colors: match env.profile[5] {
                0 => None,
                1 => Some(Arc::new(vec![])),
                _ => Some(Arc::new(vec![23; vc * 3])),
            },
        }),
        view: Rc::new(ViewEnv(Rc::downgrade(env))),
    })
}
fn observe(line: &str) -> String {
    let mut cols = line.split('\t');
    let id = cols.next().unwrap();
    let f = cols
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let profile: [usize; 9] = f.try_into().unwrap();
    let [mesh, nominal, dirty, manual, scene, _, _, sequence, _] = profile;
    let binding = Rc::new(RefCell::new(None));
    let env = Rc::new(Env {
        profile,
        enabled: Cell::new(false),
        failed: Cell::new(false),
        reentered: Cell::new(false),
        mutated: Cell::new(false),
        secondary: RefCell::new(None),
        loaded_release: RefCell::new(None),
        events: RefCell::new(vec![]),
        model: RefCell::new(None),
        runtime: binding.clone(),
        initial_app: RefCell::new(None),
    });
    let weak = Rc::downgrade(&env);
    let history_clear = Rc::new(move || {
        let e = weak.upgrade().unwrap();
        let runtime = e.runtime.borrow().as_ref().unwrap().upgrade().unwrap();
        runtime.clear_history()?;
        e.events
            .borrow_mut()
            .push(array(vec![st("history"), e.history()]));
        e.fail(7, "history")
    });
    let model = Session::new(
        (scene != 0).then(|| Rc::new(SceneEnv(Rc::downgrade(&env))) as Rc<dyn Scene>),
        history_clear,
        scan_session::Limits::default(),
    )
    .unwrap();
    *env.model.borrow_mut() = Some(model.clone());
    let owner = Owner::new(TaskLimits {
        tasks: 128,
        depth: 32,
        polls_per_turn: 10000,
    })
    .unwrap();
    let runtime = Runtime::new(
        owner.clone(),
        Rc::new(CaptureEnv(Rc::downgrade(&env))),
        guarded::Limits::default(),
    );
    *binding.borrow_mut() = Some(runtime.downgrade());
    if dirty != 0 {
        model.store(StoreId::App).patch(fields!["projectName"=>st("kept project"),"fileName"=>st("old.ply"),"modelSize"=>num(42.),"modelCenter"=>xyz([1.,2.,3.]),"statusText"=>st("old status"),"progress"=>st("old progress"),"errorText"=>st("old error"),"working"=>st("old work"),"nextId"=>num(101.),"nextDimensionId"=>num(202.),"nextNumber"=>num(8.),"nextOfKind"=>obj(vec![("point",num(3.)),("line",num(3.)),("plane",num(3.)),("sphere",num(3.)),("cylinder",num(3.)),("cone",num(3.)),("circle",num(3.)),("torus",num(3.))]),"nextOfDimGroup"=>obj(vec![("distance",num(6.)),("angle",num(7.))]),"nextSectionNumber"=>num(9.),"scanHidden"=>boolv(true),"elementsHidden"=>boolv(true),"settings"=>obj(vec![("method",st("gaussian")),("sigma",num(2.))]),"showLabels"=>boolv(false),"showBackfaces"=>boolv(false),"translucent"=>boolv(true),"wireframe"=>boolv(true),"scanColors"=>boolv(true),"showScanColors"=>boolv(false),"imprintOpen"=>boolv(true)]).unwrap();
        let app = model.store(StoreId::App);
        runtime
            .register(
                &text("unit"),
                obj(vec![
                    ("name", st("seed")),
                    ("title", st("seed")),
                    ("description", st("fixture setup")),
                    (
                        "input",
                        obj(vec![("type", st("object")), ("properties", obj(vec![]))]),
                    ),
                ]),
                Callbacks {
                    run: Rc::new(move |_, ctx| {
                        app.patch(fields!["elements"=>array(vec![point()])])?;
                        ctx.complete(Ok(V::Undefined));
                        Ok(())
                    }),
                    label: None,
                    discards: None,
                },
            )
            .unwrap();
        drop(
            runtime
                .submit(text("unit.seed"), obj(vec![]), None, false, |result| {
                    result.unwrap();
                })
                .unwrap(),
        );
        assert_eq!(owner.turn().unwrap().ready, 0);
        let probe = array(vec![obj(vec![
            ("id", num(9.)),
            ("point", xyz([1., 2., 3.])),
            ("value", num(0.25)),
        ])]);
        let held = || obj(vec![("held", boolv(true))]);
        model.store(StoreId::Deviation).patch(fields!["source"=>st("element"),"alignStatus"=>st("done"),"align"=>obj(vec![("rotation",array([1.,0.,0.,0.,1.,0.,0.,0.,1.].into_iter().map(num).collect())),("translation",xyz([1.,2.,3.]))]),"alignMessage"=>st("held"),"alignStopping"=>boolv(true),"globalAlign"=>held(),"mapStatus"=>st("ready"),"referenceStatus"=>st("ready"),"volume"=>held(),"volumeStatus"=>st("ready"),"volumeMessage"=>st("held"),"volumeHover"=>num(7.),"stats"=>held(),"histogram"=>held(),"probes"=>probe.clone(),"nextProbeId"=>num(99.),"picking"=>boolv(true),"pairs"=>array(vec![obj(vec![("scan",xyz([1.,2.,3.])),("nominal",xyz([4.,5.,6.]))])]),"pendingScan"=>xyz([1.,2.,3.]),"marking"=>boolv(true),"maxDistanceAuto"=>boolv(false),"animate"=>boolv(true),"targetId"=>num(41.),"targetSide"=>num(-1.),"elementStatus"=>st("ready"),"mapVersion"=>num(11.),"referenceVersion"=>num(12.),"elementVersion"=>num(13.),"targetScope"=>st("marked"),"scopeCount"=>num(8.),"scopeVersion"=>num(4.),"range"=>num(9.),"rangeAuto"=>boolv(false),"maxDistance"=>num(7.),"tolerance"=>num(0.4),"showHistogram"=>boolv(true),"showNominal"=>boolv(false),"showScan"=>boolv(false),"showMap"=>boolv(false),"split"=>boolv(true),"localMaxDistance"=>num(0.5)]).unwrap();
        model.store(StoreId::Thickness).patch(fields!["status"=>st("ready"),"message"=>st("held"),"stats"=>held(),"histogram"=>held(),"probes"=>probe,"nextProbeId"=>num(99.),"mapVersion"=>num(7.),"low"=>num(0.2),"high"=>num(4.),"scaleAuto"=>boolv(false),"limit"=>num(0.75),"showHistogram"=>boolv(false)]).unwrap();
        model.store(StoreId::Mark).patch(fields!["gesture"=>st("brush"),"erase"=>boolv(true),"backfaces"=>boolv(true),"diameter"=>num(5.),"count"=>num(11.),"inverts"=>num(6.)]).unwrap();
        model.remember_surface(41, Arc::new(vec![0, 1])).unwrap();
        model.clear_preview().unwrap();
        model
            .set_sources(Sources {
                scan: Some(file("old.ply", vec![79, 76, 68], Some(MeshUnits::In))),
                reference: None,
                image: Some(file("image.png", vec![1, 2, 3], None)),
            })
            .unwrap();
        for (k, _) in model.maps() {
            model
                .set_map(
                    k,
                    Some(if k == "elementScope" {
                        MapValue::Words(Arc::new(vec![1, 2]))
                    } else if k.ends_with("Rgb") {
                        MapValue::Bytes(Arc::new(vec![7, 8]))
                    } else if k == "referenceMap" {
                        MapValue::Report(cell(obj(vec![(
                            "held",
                            array(vec![num(1.25), num(2.5)]),
                        )])))
                    } else {
                        MapValue::Floats(Arc::new(vec![1.25, 2.5]))
                    }),
                )
                .unwrap();
        }
    }
    model.store(StoreId::Deviation).patch(fields!["nominalName"=>if nominal!=0{st("reference.ply")}else{V::Null},"nominalVertices"=>num(if nominal!=0{4.}else{0.}),"nominalTriangles"=>num(if nominal!=0{4.}else{0.}),"nominalFromSession"=>boolv(nominal==2),"nominalBusy"=>boolv(nominal==2),"nominalStep"=>if nominal!=0{obj(vec![("held",boolv(true))])}else{V::Null}]).unwrap();
    let mut current = model.sources();
    current.reference = (nominal != 0).then(|| {
        file(
            "reference.ply",
            vec![82, nominal as u8],
            Some(MeshUnits::Mm),
        )
    });
    model.set_sources(current).unwrap();
    model.store(StoreId::Thickness).patch(fields!["maxThicknessAuto"=>boolv(manual==0),"maxThickness"=>num(if manual!=0{17.}else{10.})]).unwrap();
    *env.initial_app.borrow_mut() = Some(model.store(StoreId::App).snapshot());
    env.events.borrow_mut().clear();
    for (name, which, fault) in [
        ("app", StoreId::App, 8),
        ("deviation", StoreId::Deviation, 9),
        ("thickness", StoreId::Thickness, 10),
        ("mark", StoreId::Mark, 11),
        ("surfaces", StoreId::Surfaces, 12),
    ] {
        let weak = Rc::downgrade(&env);
        model
            .store(which)
            .subscribe(Rc::new(move |store, current, previous| {
                let e = weak.upgrade().unwrap();
                let m = e.model();
                let identity = if name == "app" {
                    let field = |fields: &scan_session::Fields, key: &str| {
                        fields
                            .iter()
                            .find(|(k, _)| *k == text(key))
                            .unwrap()
                            .1
                            .clone()
                    };
                    array(vec![
                        boolv(
                            field(&current, "elements").strict_equal(&field(&previous, "elements")),
                        ),
                        boolv(
                            field(&current, "settings").strict_equal(&field(&previous, "settings")),
                        ),
                        boolv(Arc::ptr_eq(
                            &current,
                            e.initial_app.borrow().as_ref().unwrap(),
                        )),
                    ])
                } else {
                    V::Null
                };
                e.events.borrow_mut().push(array(vec![
                    st("state"),
                    st(name),
                    field_value(&current),
                    field_value(&previous),
                    boolv(Arc::ptr_eq(&current, &store.snapshot())),
                    identity,
                    sources(&m),
                    maps(&m),
                    num(m.preview().0),
                    regions(&m),
                ]));
                if name == "app" && e.profile[7] == 4 && !e.reentered.replace(true) {
                    e.secondary.borrow().as_ref().unwrap().unsubscribe();
                    e.model()
                        .store(StoreId::App)
                        .subscribe(extra_app(Rc::downgrade(&e), "C"))?;
                    e.model()
                        .store(StoreId::App)
                        .patch(fields!["progress"=>st("listener")])?;
                }
                e.fail(fault, name)
            }))
            .unwrap();
    }
    if sequence == 4 {
        *env.secondary.borrow_mut() = Some(
            model
                .store(StoreId::App)
                .subscribe(extra_app(Rc::downgrade(&env), "B"))
                .unwrap(),
        );
    }
    let weak = Rc::downgrade(&env);
    model
        .on_loaded(Rc::new(move |_, loaded| {
            let e = weak.upgrade().unwrap();
            e.events.borrow_mut().push(array(vec![
                st("loaded"),
                loaded.map_or(V::Null, |p| {
                    obj(vec![
                        (
                            "positions",
                            array(p.positions.iter().map(|v| num(f64::from(*v))).collect()),
                        ),
                        (
                            "indices",
                            array(p.indices.iter().map(|v| num(f64::from(*v))).collect()),
                        ),
                        ("modelSize", num(p.model_size)),
                    ])
                }),
            ]));
            e.fail(6, "loaded")
        }))
        .unwrap();
    if sequence == 3 {
        let weak = Rc::downgrade(&env);
        model
            .on_loaded(Rc::new(move |_, _| {
                let e = weak.upgrade().unwrap();
                e.events
                    .borrow_mut()
                    .push(array(vec![st("loaded-extra"), st("L1")]));
                if !e.mutated.replace(true) {
                    e.loaded_release.borrow().as_ref().unwrap().unsubscribe();
                    let late = Rc::downgrade(&e);
                    e.model().on_loaded(Rc::new(move |_, _| {
                        late.upgrade()
                            .unwrap()
                            .events
                            .borrow_mut()
                            .push(array(vec![st("loaded-extra"), st("L3")]));
                        Ok(())
                    }))?;
                }
                Ok(())
            }))
            .unwrap();
        let weak = Rc::downgrade(&env);
        let duplicate: scan_session::LoadedListener = Rc::new(move |_, _| {
            weak.upgrade()
                .unwrap()
                .events
                .borrow_mut()
                .push(array(vec![st("loaded-extra"), st("duplicate")]));
            Ok(())
        });
        *env.loaded_release.borrow_mut() = Some(model.on_loaded(duplicate.clone()).unwrap());
        model.on_loaded(duplicate).unwrap();
    }
    let mut frames = vec![env.frame(&V::Null)];
    env.enabled.set(true);
    let mut modes = vec![mesh];
    if sequence == 1 {
        modes.push(0)
    }
    if sequence == 2 {
        modes.extend([0, mesh])
    }
    if sequence == 3 {
        modes.push(mesh);
    }
    for mode in modes {
        let p = prepared(&env, mode);
        let result = model.commit_scan(p.as_ref());
        let error = match result {
            Ok(()) => V::Null,
            Err(Fault::Error { name, message }) => array(vec![V::String(name), V::String(message)]),
            Err(e) => panic!("unexpected native commit {e:?}"),
        };
        if std::env::var_os("SCANRULER_SCAN_SESSION_SABOTAGE").is_some() {
            let C::Number(id) = model.store(StoreId::App).get("nextId") else {
                panic!("id")
            };
            model
                .store(StoreId::App)
                .patch(fields!["nextId"=>num(id+1.)])
                .ok();
        }
        frames.push(env.frame(&error));
    }
    format!("{id}\t{}\n", frames.join(";"))
}
#[test]
fn native_scan_session_profiles_finish() {
    assert_eq!(INPUT.lines().count(), 800);
    let output = std::env::var_os("SCANRULER_SCAN_SESSION_DUMP").map(std::path::PathBuf::from);
    let mut dump = output.map(|p| std::fs::File::create(p).unwrap());
    for line in INPUT.lines() {
        let result = observe(line);
        if let Some(dump) = &mut dump {
            use std::io::Write;
            dump.write_all(result.as_bytes()).unwrap();
        }
    }
}

#[derive(Clone, Copy)]
enum Rule {
    Frame,
    App,
    Thickness,
    Mark,
    Events,
    Event,
    Loaded,
    Strict,
}
fn scalar_key(rule: Rule, key: &[u16]) -> bool {
    match rule {
        Rule::App => key == text("modelSize"),
        Rule::Thickness => key == text("maxThickness"),
        Rule::Mark => key == text("diameter"),
        Rule::Loaded => key == text("modelSize"),
        _ => false,
    }
}
fn event_rule(value: &V, index: usize) -> Rule {
    let V::Array(v) = value else {
        return Rule::Strict;
    };
    let tag = v.first();
    if tag == Some(&st("state")) && (index == 2 || index == 3) {
        match v.get(1) {
            Some(V::String(s)) if *s == text("app") => Rule::App,
            Some(V::String(s)) if *s == text("thickness") => Rule::Thickness,
            Some(V::String(s)) if *s == text("mark") => Rule::Mark,
            _ => Rule::Strict,
        }
    } else if tag == Some(&st("loaded")) && index == 1 {
        Rule::Loaded
    } else if tag == Some(&st("app-extra")) && (index == 2 || index == 3) {
        Rule::App
    } else {
        Rule::Strict
    }
}
fn equivalent(reference: &V, native: &V, rule: Rule) -> bool {
    match (reference, native) {
        (V::Array(a), V::Array(b)) => {
            a.len() == b.len()
                && a.iter().zip(b).enumerate().all(|(i, (a, b))| {
                    equivalent(
                        a,
                        b,
                        match rule {
                            Rule::Frame => match i {
                                1 => Rule::App,
                                3 => Rule::Thickness,
                                4 => Rule::Mark,
                                12 => Rule::Events,
                                _ => Rule::Strict,
                            },
                            Rule::Events => Rule::Event,
                            Rule::Event => event_rule(reference, i),
                            _ => Rule::Strict,
                        },
                    )
                })
        }
        (V::Object(a), V::Object(b)) => {
            a.len() == b.len()
                && a.iter().zip(b).all(|((ak, av), (bk, bv))| {
                    ak == bk
                        && if scalar_key(rule, ak) {
                            match (av, bv) {
                                (V::Number(x), V::Number(y)) if x.is_finite() && y.is_finite() => {
                                    (*x - *y).abs() <= 1e-12 * (1. + x.abs())
                                }
                                _ => equivalent(av, bv, Rule::Strict),
                            }
                        } else {
                            equivalent(av, bv, Rule::Strict)
                        }
                })
        }
        (V::Number(a), V::Number(b)) => a.to_bits() == b.to_bits(),
        _ => reference == native,
    }
}
fn decode_wire(encoded: &str) -> Result<V, &'static str> {
    if !encoded.len().is_multiple_of(2) || encoded.len() > 32 * 1024 * 1024 {
        return Err("wire length");
    }
    let bytes = encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| {
            u8::from_str_radix(std::str::from_utf8(v).map_err(|_| "hex utf8")?, 16)
                .map_err(|_| "hex")
        })
        .collect::<Result<Vec<_>, _>>()?;
    struct Reader<'a> {
        bytes: &'a [u8],
        nodes: usize,
    }
    impl Reader<'_> {
        fn take(&mut self, len: usize) -> Result<&[u8], &'static str> {
            if self.bytes.len() < len {
                return Err("truncated");
            };
            let (a, b) = self.bytes.split_at(len);
            self.bytes = b;
            Ok(a)
        }
        fn count(&mut self) -> Result<usize, &'static str> {
            Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| "count")?) as usize)
        }
        fn text(&mut self) -> Result<Vec<u16>, &'static str> {
            let n = self.count()?;
            if n > 1_000_000 {
                return Err("text budget");
            };
            Ok(self
                .take(n * 2)?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| u16::from_le_bytes(*v))
                .collect())
        }
        fn value(&mut self, depth: usize) -> Result<V, &'static str> {
            self.nodes += 1;
            if self.nodes > 1_000_000 || depth > 64 {
                return Err("value budget");
            };
            Ok(match self.take(1)?[0] {
                0 => V::Undefined,
                1 => V::Null,
                2 => boolv(false),
                3 => boolv(true),
                4 => num(f64::from_le_bytes(
                    self.take(8)?.try_into().map_err(|_| "number")?,
                )),
                5 => V::String(self.text()?),
                6 => {
                    let n = self.count()?;
                    if n > 1_000_000 {
                        return Err("array budget");
                    };
                    array(
                        (0..n)
                            .map(|_| self.value(depth + 1))
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                }
                7 => {
                    let n = self.count()?;
                    if n > 1_000_000 {
                        return Err("object budget");
                    };
                    V::Object(
                        (0..n)
                            .map(|_| Ok((self.text()?, self.value(depth + 1)?)))
                            .collect::<Result<Vec<_>, &str>>()?,
                    )
                }
                8 => {
                    let n = self.count()?;
                    if n > 16 * 1024 * 1024 {
                        return Err("bytes budget");
                    };
                    V::Bytes(self.take(n)?.to_vec())
                }
                _ => return Err("tag"),
            })
        }
    }
    let mut reader = Reader {
        bytes: &bytes,
        nodes: 0,
    };
    let v = reader.value(0)?;
    if !reader.bytes.is_empty() {
        return Err("trailing bytes");
    };
    Ok(v)
}
fn frames(line: &str) -> Result<(&str, Vec<V>), &'static str> {
    let (id, frames) = line
        .trim_end_matches('\n')
        .split_once('\t')
        .ok_or("missing case id")?;
    let frames = frames
        .split(';')
        .map(decode_wire)
        .collect::<Result<Vec<_>, _>>()?;
    if frames.len() < 2 {
        return Err("missing frames");
    };
    Ok((id, frames))
}
fn same_trace(original: &str, native: &str) -> bool {
    let (Ok((a, x)), Ok((b, y))) = (frames(original), frames(native)) else {
        return false;
    };
    a == b && x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, &b, Rule::Frame))
}
#[test]
fn comparison_rejects_missing_and_wrong_owned_values() {
    let v = array(vec![V::Null, obj(vec![("nextId", num(1.))])]);
    let mut wrong = v.clone();
    if let V::Array(ref mut fields) = wrong {
        fields[1] = obj(vec![("nextId", num(2.))])
    }
    assert!(!equivalent(&v, &wrong, Rule::Frame));
    assert!(decode_wire("ff").is_err());
    assert!(decode_wire("0601000000").is_err());
    assert!(!same_trace("case\t00;00\n", "case\t00\n"));
}
#[test]
#[ignore = "complete original actual scan/store graph needs Node22"]
fn pinned_scan_session_commit_comparison() {
    use std::{
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = root.join(format!("rust/target/scan-session-commit-diff/run-{stamp}"));
    std::fs::create_dir_all(&dir).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    std::fs::write(dir.join("candidate.txt"), &sha.stdout).unwrap();
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(dirty.status.success());
    std::fs::write(
        dir.join("identity.txt"),
        format!(
            "reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7\ncandidate: {}dirty: {}",
            String::from_utf8_lossy(&sha.stdout),
            String::from_utf8_lossy(&dirty.stdout)
        ),
    )
    .unwrap();
    let built = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/build-scan-session-commit.mjs"))
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::write(dir.join("oracle-build.stdout.txt"), &built.stdout).unwrap();
    std::fs::write(dir.join("oracle-build.stderr.txt"), &built.stderr).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut child = Command::new("node")
        .arg(root.join("docs/port-plan/oracle/scan-session-commit.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(INPUT.as_bytes()).unwrap());
    let source = child.wait_with_output().unwrap();
    writer.join().unwrap();
    std::fs::write(dir.join("oracle-response.stderr.txt"), &source.stderr).unwrap();
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    assert!(source.stderr.is_empty());
    let output = String::from_utf8(source.stdout).unwrap();
    let originals = output.split_inclusive('\n').collect::<Vec<_>>();
    assert_eq!(originals.len(), 800);
    let mut mismatches = 0;
    for (line, original) in INPUT.lines().zip(originals) {
        let id = line.split('\t').next().unwrap();
        let case = dir.join(id);
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("input.bin"), line.as_bytes()).unwrap();
        std::fs::write(case.join("original.bin"), original.as_bytes()).unwrap();
        std::fs::write(case.join("original.stderr.txt"), &source.stderr).unwrap();
        let actual = observe(line);
        std::fs::write(case.join("rust.bin"), actual.as_bytes()).unwrap();
        if !same_trace(original, &actual) {
            mismatches += 1;
            if mismatches <= 3 {
                eprintln!("full state/effect mismatch: {id}")
            }
        }
        let (_, reference) = frames(original).unwrap();
        let (_, mut wrong) = frames(&actual).unwrap();
        if let V::Array(top) = &mut wrong[0]
            && let V::Object(app) = &mut top[1]
        {
            let (_, value) = app.iter_mut().find(|(k, _)| *k == text("nextId")).unwrap();
            if let V::Number(id) = value {
                *id += 1.
            }
        }
        assert!(
            !equivalent(&reference[0], &wrong[0], Rule::Frame),
            "actual wrong ID must fail"
        );
        let mut missing = actual.trim_end().split(';').collect::<Vec<_>>();
        missing.pop();
        assert!(
            !same_trace(original, &missing.join(";")),
            "missing frame must fail"
        );
    }
    println!(
        "800 actual scan document traces; mismatches={mismatches}; raw {}",
        dir.display()
    );
    worker_profiles::compare_into(&dir);
    assert_eq!(mismatches, 0);
    eprintln!(
        "849 complete scan/session controls retained in {}",
        dir.display()
    );
}

struct QuietView {
    commits: Rc<Cell<usize>>,
    disposes: Rc<Cell<usize>>,
}
impl PreparedView for QuietView {
    fn commit(&self) -> Result<(), Fault> {
        self.commits.set(self.commits.get() + 1);
        Ok(())
    }
    fn dispose(&self) -> Result<(), Fault> {
        self.disposes.set(self.disposes.get() + 1);
        Ok(())
    }
}
#[test]
fn immutable_document_cells_and_sources_retain_real_identity() {
    let document = Session::new(None, Rc::new(|| Ok(())), scan_session::Limits::default()).unwrap();
    let store = document.store(StoreId::App);
    let before = store.snapshot();
    let elements = store.get("elements");
    let settings = store.get("settings");
    store.patch(fields!["busy"=>boolv(true)]).unwrap();
    assert!(!Arc::ptr_eq(&before, &store.snapshot()));
    assert!(elements.strict_equal(&store.get("elements")));
    assert!(settings.strict_equal(&store.get("settings")));
    let source = file("owned.ply", vec![1, 2, 3], Some(MeshUnits::Mm));
    document
        .set_sources(Sources {
            scan: Some(source.clone()),
            reference: None,
            image: None,
        })
        .unwrap();
    assert!(Arc::ptr_eq(
        document.sources().scan.as_ref().unwrap(),
        &source
    ));
    let C::Array(old) = elements else {
        panic!("array")
    };
    assert!(old.is_empty());
}
#[test]
fn dropping_session_with_weak_callbacks_releases_assets_and_owner() {
    let document = Session::new(None, Rc::new(|| Ok(())), scan_session::Limits::default()).unwrap();
    let weak = document.downgrade();
    let callback = weak.clone();
    document
        .on_loaded(Rc::new(move |_, _| {
            assert!(callback.upgrade().is_some());
            Ok(())
        }))
        .unwrap();
    let store = document.store(StoreId::App);
    let weak_store = store.downgrade();
    let callback = weak_store.clone();
    store
        .subscribe(Rc::new(move |_, _, _| {
            assert!(callback.upgrade().is_some());
            Ok(())
        }))
        .unwrap();
    let snapshot = store.snapshot();
    let positions = Arc::new(vec![0., 0., 0.]);
    let positions_weak = Arc::downgrade(&positions);
    let commits = Rc::new(Cell::new(0));
    let disposes = Rc::new(Cell::new(0));
    let prepared = PreparedScan {
        asset: Arc::new(ScanAsset {
            source: file("owned.ply", vec![1], None),
            positions,
            indices: Arc::new(vec![]),
            vertex_count: 1,
            triangle_count: 0,
            colors: None,
        }),
        view: Rc::new(QuietView {
            commits: commits.clone(),
            disposes: disposes.clone(),
        }),
    };
    document.commit_scan(Some(&prepared)).unwrap();
    assert!(Arc::ptr_eq(
        document.asset().as_ref().unwrap(),
        &prepared.asset
    ));
    let region = Arc::new(vec![0]);
    document.set_preview_region(Some(region.clone())).unwrap();
    assert!(Arc::ptr_eq(document.preview().1.as_ref().unwrap(), &region));
    document.clear_preview().unwrap();
    assert!(document.preview().1.is_none());
    assert_eq!(
        region.as_slice(),
        &[0],
        "old immutable preview survives clearing"
    );
    assert_eq!(disposes.get(), 0, "consumer does not own view disposal");
    prepared.view.dispose().unwrap();
    assert_eq!(disposes.get(), 1, "caller owns exactly this disposal");
    drop(prepared);
    assert!(positions_weak.upgrade().is_some());
    assert_eq!(commits.get(), 1);
    assert_eq!(disposes.get(), 1, "consumer must not dispose again");
    drop(document);
    assert!(weak.upgrade().is_none());
    assert!(positions_weak.upgrade().is_none());
    assert!(weak_store.upgrade().is_some());
    drop(store);
    assert!(weak_store.upgrade().is_none());
    assert!(!snapshot.is_empty());
}
#[test]
fn host_admission_rejects_duplicate_and_deep_state_before_publication() {
    let document = Session::new(None, Rc::new(|| Ok(())), scan_session::Limits::default()).unwrap();
    let store = document.store(StoreId::App);
    let before = store.snapshot();
    assert!(
        store
            .patch(fields!["busy"=>boolv(true),"busy"=>boolv(false)])
            .is_err()
    );
    assert!(Arc::ptr_eq(&before, &store.snapshot()));
    let mut nested = C::Null;
    for _ in 0..40 {
        nested = C::Array(Arc::new(vec![nested]))
    }
    assert!(store.patch(vec![(text("untrusted"), nested)]).is_err());
    assert!(Arc::ptr_eq(&before, &store.snapshot()));
    assert!(
        Session::new(
            None,
            Rc::new(|| Ok(())),
            scan_session::Limits {
                fields: 1,
                ..scan_session::Limits::default()
            }
        )
        .is_err()
    );
}
#[test]
fn real_render_size_and_companion_metadata_have_host_limits() {
    let limits = scan_session::Limits {
        vertices: 1,
        fields: 64,
        ..scan_session::Limits::default()
    };
    let document = Session::new(None, Rc::new(|| Ok(())), limits).unwrap();
    let before = document.store(StoreId::App).snapshot();
    let commits = Rc::new(Cell::new(0));
    let disposes = Rc::new(Cell::new(0));
    let prepared = PreparedScan {
        asset: Arc::new(ScanAsset {
            source: file("oversized.ply", vec![1], None),
            positions: Arc::new(vec![0., 0., 0., 1., 1., 1.]),
            indices: Arc::new(vec![]),
            vertex_count: 1,
            triangle_count: 0,
            colors: None,
        }),
        view: Rc::new(QuietView {
            commits: commits.clone(),
            disposes,
        }),
    };
    assert!(document.commit_scan(Some(&prepared)).is_err());
    assert_eq!(document.preview().0, 0.);
    assert_eq!(commits.get(), 0);
    assert!(Arc::ptr_eq(
        &before,
        &document.store(StoreId::App).snapshot()
    ));
    let companions = (0..65)
        .map(|_| scan_session::Companion {
            name: vec![],
            bytes: Arc::new(vec![]),
        })
        .collect();
    let source = Arc::new(SourceFile {
        name: text("many.obj"),
        bytes: Arc::new(vec![]),
        units: None,
        companions: Some(companions),
    });
    assert!(
        document
            .set_sources(Sources {
                scan: Some(source),
                reference: None,
                image: None
            })
            .is_err()
    );
    assert!(document.sources().scan.is_none());
}

#[test]
fn original_units_and_bytes_are_retained_without_regenerating_source() {
    for unit in [MeshUnits::Mm, MeshUnits::Cm, MeshUnits::M, MeshUnits::In] {
        let document =
            Session::new(None, Rc::new(|| Ok(())), scan_session::Limits::default()).unwrap();
        let source = file("original.stl", vec![0, 255, 17], Some(unit));
        document
            .set_sources(Sources {
                scan: Some(source.clone()),
                reference: None,
                image: None,
            })
            .unwrap();
        let retained = document.sources().scan.unwrap();
        assert!(Arc::ptr_eq(&retained, &source));
        assert_eq!(&*retained.bytes, &[0, 255, 17]);
        assert_eq!(retained.units.unwrap().id(), unit.id());
    }
}
