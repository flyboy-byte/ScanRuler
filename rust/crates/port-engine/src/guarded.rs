// SPDX-License-Identifier: AGPL-3.0-only
//! Native registered mutation/interrupt execution over the reviewed session providers.
//! Read-only/recent-result dispatch and concrete family state are explicit continuations.
use crate::{
    activity_state::{Activity, Patch, Running},
    command_deadline::{COMMAND_TIMEOUT_MS, Completion},
    command_label,
    error::{CommandError, CommandErrorCode},
    error_normalize::{self, Caught},
    execute::{self, Done, Host as ExecuteHost},
    history_record::{Recorder, Snapshot, Token},
    page_stale::PageStale,
    reactions::{self, Limited},
    registry::{self, Cleanup, Command, Registry},
    schema_value::{Text, Value},
    settle::Work,
    tasks::{Owner, Ticket, WeakOwner},
};
use std::{
    cell::{Cell, RefCell},
    fmt,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    sync::Arc,
};
/// Source faults retain owned UTF-16 and typed error identity. Host failures remain distinct.
#[derive(Clone)]
pub enum Fault {
    /// Exact source CommandError object.
    Command(Arc<CommandError>),
    /// An actual Error (including TypeError), before command normalization.
    Error {
        /// Original Error subclass name.
        name: Text,
        /// Exact message, before normalization.
        message: Text,
    },
    /// Primitive thrown string.
    Text(Text),
    /// Fallible source String conversion, evaluated only by normalization.
    Other(Rc<dyn Fn() -> Result<Text, Fault>>),
    /// Rust metadata/admission/protocol failure, never counted as a source refusal.
    Host(Rc<dyn fmt::Debug>),
}
impl fmt::Debug for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(e) => e.fmt(f),
            Self::Error { name, message } => f
                .debug_struct("Error")
                .field("name", name)
                .field("message", message)
                .finish(),
            Self::Text(t) => t.fmt(f),
            Self::Other(_) => f.write_str("Other(String provider)"),
            Self::Host(e) => f.debug_tuple("Host").field(e).finish(),
        }
    }
}
impl Fault {
    /// Preserve a native host failure without a source error code or lossy text conversion.
    pub fn host(error: impl fmt::Debug + 'static) -> Self {
        Self::Host(Rc::new(error))
    }
    /// Plain source Error.
    pub fn error(message: &str) -> Self {
        Self::Error {
            name: "Error".encode_utf16().collect(),
            message: message.encode_utf16().collect(),
        }
    }
}
/// A command result future, owned independently of external observer tickets.
pub type ResultWork = Pin<Box<dyn Future<Output = Result<Value, Fault>>>>;
/// The application owns the document/assets. The executor owns only history and runtime metadata.
/// Snapshot providers must return immutable shared assets, not another mutable business store.
pub trait Host {
    /// Raw source guard answer; empty text differs from null in execute preflight.
    fn busy(&self) -> Result<Option<Text>, Fault>;
    /// Current workspace, sampled after before-capture.
    fn workspace(&self) -> Result<Text, Fault>;
    /// Capture the actual application's registered history sections.
    fn capture(&self) -> Result<Snapshot, Fault>;
    /// Raw Date.now sample; stale ISO conversion is lazy.
    fn now(&self) -> Result<f64, Fault>;
    /// Trusted serial timer backend. Invoke callback on the owning session thread.
    fn timer(&self, ms: f64, callback: Rc<dyn Fn() -> Result<(), Fault>>) -> u64;
    /// Trusted synchronous clearTimeout equivalent.
    fn clear_timer(&self, id: u64);
    /// Eager source settle factory, including ready work for the null-await branch.
    fn settle(&self, name: &[u16]) -> Result<Work<'static, Fault>, Fault>;
    /// Eager plugin stopping; done is resolved by execute's own finally.
    fn stop(&self, done: Done) -> Work<'static, Fault>;
}
/// Eager native body factory; its result is the source synchronous-throw boundary.
pub type BodyFactory = Rc<dyn Fn(&Value, Context) -> Result<(), Fault>>;
/// Optional-label provider; nullish and an actual empty string remain different.
pub type LabelProvider = Rc<dyn Fn(&Value) -> Result<Option<Text>, Fault>>;
/// Source discards predicate before strict Boolean(true) classification.
pub type DiscardsProvider = Rc<dyn Fn(&Value) -> Result<Value, Fault>>;
/// Synchronous abort observer, passed its context to avoid self-capture cycles.
pub type AbortListener = Rc<dyn Fn(&Context)>;
/// Actual native implementation callbacks for a registered command.
#[derive(Clone)]
pub struct Callbacks {
    /// Eager body factory. Complete its promise through Context; Err is a synchronous throw.
    pub run: BodyFactory,
    /// Optional exact label expression; None result is source nullish fallback.
    pub label: Option<LabelProvider>,
    /// Optional raw discards return; only Boolean(true) selects stopping.
    pub discards: Option<DiscardsProvider>,
}
/// One caller's progress callback; errors follow the source callback stack.
pub type Progress = Rc<dyn Fn(&[u16]) -> Result<(), Fault>>;
/// Current guarded change; interrupts leave this unchanged.
#[derive(Clone, Debug)]
pub struct Current {
    /// Full command name.
    pub name: Rc<[u16]>,
    /// Whether this guarded change was deferred.
    pub deferred: bool,
}
/// Independent host quotas. They do not change source command timeout policy.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Existing registration/schema metadata limits.
    pub registry: registry::Limits,
    /// Retained implementation slots, including cleaned definitions.
    pub bodies: usize,
    /// Outstanding scene/mutation turns.
    pub pending: usize,
    /// UTF-16 labels/errors/progress and abort-listener slots.
    pub text: usize,
    /// Maximum abort listeners per body.
    pub abort_listeners: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            registry: registry::Limits::default(),
            bodies: 1024,
            pending: 1024,
            text: 1_000_000,
            abort_listeners: 128,
        }
    }
}
struct Core {
    owner: WeakOwner<'static>,
    host: Rc<dyn Host>,
    registry: RefCell<Registry>,
    bodies: RefCell<Vec<Callbacks>>,
    activity: Activity<'static, Fault>,
    current: RefCell<Option<Current>>,
    history: RefCell<Recorder>,
    stale: RefCell<PageStale>,
    mutations: crate::scene_queue::SceneQueue<Fault>,
    scene: crate::scene_queue::SceneQueue<Fault>,
    limits: Limits,
}
/// Session-owned executor, separate from its application's sole document.
#[derive(Clone)]
pub struct Runtime {
    owner: Owner<'static>,
    core: Rc<Core>,
}
/// Non-owning registration/session handle for command and host callbacks. Upgrade only for
/// an immediate operation; retaining an upgraded runtime across pending work creates a cycle.
#[derive(Clone)]
pub struct WeakRuntime {
    core: Weak<Core>,
    owner: WeakOwner<'static>,
}
impl WeakRuntime {
    /// Upgrade while the external session owns its driver and runtime metadata still exists.
    pub fn upgrade(&self) -> Option<Runtime> {
        Some(Runtime {
            owner: self.owner.upgrade()?,
            core: self.core.upgrade()?,
        })
    }
}
struct BodyState {
    limited: Limited<Value, Fault>,
    factory: Cell<bool>,
    completed: Cell<bool>,
    staged: RefCell<Option<Result<Value, Fault>>>,
    activity: Option<Activity<'static, Fault>>,
    progress: Option<Progress>,
    deferred: Option<bool>,
    listeners: RefCell<Vec<AbortListener>>,
    listener_limit: usize,
    owner: WeakOwner<'static>,
}
/// The body promise and signal, retained by native jobs after caller timeout.
#[derive(Clone)]
pub struct Context {
    state: Rc<BodyState>,
}
impl Context {
    /// Source ctx.deferred is absent unless guarded runs deferred; interrupts never carry it.
    pub fn deferred(&self) -> Option<bool> {
        self.state.deferred
    }
    /// Exact signal reason; reading it never normalizes/copies a typed error.
    pub fn abort_reason(&self) -> Option<Arc<CommandError>> {
        self.state.limited.abort_reason()
    }
    /// Deliver progress: suppress only aborted strip publication, always call the caller afterward.
    pub fn progress(&self, text: &[u16]) -> Result<(), Fault> {
        if self.abort_reason().is_none()
            && let Some(activity) = &self.state.activity
        {
            activity
                .publish(Patch::Progress(Rc::from(text)))
                .map_err(activity_fault)?;
        }
        if let Some(progress) = &self.state.progress {
            progress(text)?;
        }
        Ok(())
    }
    /// Attach a synchronous native abort observer (not a JS global exception-reporting emulation).
    pub fn on_abort(&self, listener: AbortListener) -> Result<(), Fault> {
        let mut listeners = self.state.listeners.borrow_mut();
        if listeners
            .iter()
            .any(|existing| Rc::ptr_eq(existing, &listener))
        {
            return Ok(());
        }
        if listeners.len() >= self.state.listener_limit {
            return Err(Fault::host("abort listener budget"));
        }
        listeners.try_reserve(1).map_err(Fault::host)?;
        listeners.push(listener);
        Ok(())
    }
    /// Resolve/reject this body once. Results during the factory are staged so a synchronous
    /// throw can still select the rejected promise that the source factory actually returns.
    pub fn complete(&self, result: Result<Value, Fault>) -> bool {
        if self.state.completed.replace(true) {
            drop(result);
            return false;
        }
        if self.state.factory.get() {
            *self.state.staged.borrow_mut() = Some(result);
            true
        } else {
            self.state.limited.body_settled(result)
        }
    }
    /// Retain a native producer, reporting its final body result even after caller abandonment.
    pub fn spawn(
        &self,
        future: impl Future<Output = Result<Value, Fault>> + 'static,
    ) -> Result<Ticket, Fault> {
        let owner = self
            .state
            .owner
            .upgrade()
            .ok_or_else(|| Fault::host("closed runtime"))?;
        let context = self.clone();
        owner
            .spawn(async move {
                let result = future.await;
                context.complete(result);
            })
            .map_err(Fault::host)
    }
    fn fire(&self) -> Result<(), Fault> {
        if let Some(event) = self.state.limited.fire_begin().map_err(Fault::host)? {
            // Live listener iteration; no RefCell borrow spans callback invocation.
            let mut index = 0;
            loop {
                let listener = self.state.listeners.borrow().get(index).cloned();
                let Some(listener) = listener else { break };
                index += 1;
                listener(self);
            }
            self.state.limited.fire_end(event).map_err(Fault::host)?;
        }
        Ok(())
    }
}
fn activity_fault(error: crate::activity_state::Error<Fault>) -> Fault {
    match error {
        crate::activity_state::Error::Listener(e) => e,
        e => Fault::host(e),
    }
}
fn history_fault(error: crate::history_record::HistoryError<Fault>) -> Fault {
    match error {
        crate::history_record::HistoryError::Capture(e) => e,
        e => Fault::host(e),
    }
}
fn truthy(value: &Value) -> bool {
    match value {
        Value::Undefined | Value::Null => false,
        Value::Boolean(v) => *v,
        Value::Number(v) => *v != 0. && !v.is_nan(),
        Value::String(v) => !v.is_empty(),
        _ => true,
    }
}
fn ready(result: Result<Value, Fault>) -> ResultWork {
    Box::pin(std::future::ready(result))
}
impl Runtime {
    /// Fresh runtime; the application's Host retains the actual family document.
    pub fn new(owner: Owner<'static>, host: Rc<dyn Host>, limits: Limits) -> Self {
        let core = Rc::new(Core {
            owner: owner.downgrade(),
            host,
            registry: RefCell::new(Registry::new()),
            bodies: RefCell::new(Vec::new()),
            activity: Activity::new(crate::activity_state::Limits {
                slots: 1024,
                visits: 100_000,
                depth: 32,
                text_units: limits.text,
            }),
            current: RefCell::new(None),
            history: RefCell::new(Recorder::default()),
            stale: RefCell::new(PageStale::default()),
            mutations: crate::scene_queue::SceneQueue::new(limits.pending),
            scene: crate::scene_queue::SceneQueue::new(limits.pending),
            limits,
        });
        Self { owner, core }
    }
    /// A non-owning handle suitable for callbacks installed into this runtime itself.
    pub fn downgrade(&self) -> WeakRuntime {
        WeakRuntime {
            core: Rc::downgrade(&self.core),
            owner: self.owner.downgrade(),
        }
    }
    /// Clear source history/epoch after a real document replacement. Pending steps retain
    /// their old generation and are discarded when they subsequently finish.
    pub fn clear_history(&self) -> Result<(), Fault> {
        self.core.history.borrow_mut().clear().map_err(Fault::host)
    }
    /// Enumerate only definitions whose native implementations were registered.
    pub fn list(&self) -> Result<Value, Fault> {
        self.core.registry.borrow().list().map_err(registry_fault)
    }
    /// Register a real callback and the original owned schema/metadata. Read-only dispatch is a
    /// separate continuation; this bounded executor exposes mutation/interrupt registration only.
    pub fn register(
        &self,
        namespace: &[u16],
        definition: Value,
        callbacks: Callbacks,
    ) -> Result<Cleanup, Fault> {
        if truthy(definition.get("readOnly")) {
            return Err(Fault::host(
                "read-only runtime continuation is not implemented",
            ));
        }
        let mut bodies = self.core.bodies.borrow_mut();
        if bodies.len() >= self.core.limits.bodies {
            return Err(Fault::host("implementation budget"));
        }
        bodies.try_reserve(1).map_err(Fault::host)?;
        let cleanup = self
            .core
            .registry
            .borrow_mut()
            .register(
                namespace,
                &[Command {
                    definition,
                    body: bodies.len(),
                }],
                self.core.limits.registry,
            )
            .map_err(registry_fault)?;
        bodies.push(callbacks);
        Ok(cleanup)
    }
    /// Invoke source cleanup without dropping implementations held by earlier queued calls.
    pub fn cleanup(&self, cleanup: Cleanup) -> Result<(), Fault> {
        self.core
            .registry
            .borrow_mut()
            .cleanup(cleanup)
            .map_err(registry_fault)
    }
    /// The actual activity store (live listener identity/reentry semantics are reused).
    pub fn activity(&self) -> Activity<'static, Fault> {
        self.core.activity.clone()
    }
    /// Snapshot current guarded metadata; never a second business document.
    pub fn current(&self) -> Option<Current> {
        self.core.current.borrow().clone()
    }
    /// Read history metadata/immutable patch assets without exposing mutable document duplication.
    pub fn history(&self) -> crate::history_record::State {
        self.core.history.borrow().state.clone()
    }
    /// Whether the source async history wrapper is open, including interrupted shared history.
    pub fn history_paused(&self) -> bool {
        self.core.history.borrow().paused()
    }
    /// First stale-page timestamp, retained after normalization even when later cleanup fails.
    pub fn page_stale(&self) -> Option<Text> {
        self.core.stale.borrow().since().map(<[u16]>::to_vec)
    }
    /// Begin one registered command and capture the requesting receiver, never a global request ID.
    /// Admission failure returns before any registry/queue side effect or body factory invocation.
    pub fn submit(
        &self,
        name: Text,
        input: Value,
        progress: Option<Progress>,
        deferred: bool,
        receive: impl FnOnce(Result<Value, Fault>) + 'static,
    ) -> Result<Ticket, crate::tasks::Error> {
        let core = self.core.clone();
        reactions::retain(
            &self.owner,
            move || core.start(name, input, progress, deferred),
            receive,
        )
    }
}
fn registry_fault(error: registry::RegistryError) -> Fault {
    match error {
        registry::RegistryError::Command(error) => Fault::Command(Arc::new(error)),
        error => Fault::host(error),
    }
}
fn queue_fault(error: crate::import_queue::QueueError<Fault>) -> Fault {
    match error {
        crate::import_queue::QueueError::Callback(error) => error,
        error => Fault::host(error),
    }
}
#[derive(Clone)]
struct Call {
    name: Rc<[u16]>,
    definition: Value,
    callbacks: Callbacks,
    input: Value,
    progress: Option<Progress>,
    deferred: bool,
}
impl Core {
    fn start(
        self: Rc<Self>,
        name: Text,
        mut input: Value,
        progress: Option<Progress>,
        deferred: bool,
    ) -> ResultWork {
        let captured = {
            let registry = self.registry.borrow();
            let admitted = match registry.admit(&name, &mut input, self.limits.registry) {
                Ok(v) => v,
                Err(e) => return ready(Err(registry_fault(e))),
            };
            let Some(command) = registry.definition(&name).cloned() else {
                return ready(Err(Fault::host("admitted command missing")));
            };
            let callbacks = self.bodies.borrow().get(admitted.body).cloned();
            (command, admitted, callbacks)
        };
        let (command, admitted, Some(callbacks)) = captured else {
            return ready(Err(Fault::host("admitted implementation missing")));
        };
        let call = Call {
            name: Rc::from(name),
            definition: command.definition,
            callbacks,
            input: admitted.input,
            progress,
            deferred,
        };
        if truthy(call.definition.get("interrupts")) {
            return self.guarded(call, true);
        }
        let queue = self.mutations.clone();
        let core = self.clone();
        match queue.run(true, move || {
            let scene = core.scene.clone();
            let is_scene = truthy(call.definition.get("scene"));
            let routed = scene.run(is_scene, move || {
                let deferred = call.deferred;
                let run = execute::execute(Provider { core, call }, deferred, 1024);
                Ok(Box::pin(async move {
                    run.await.map_err(|e| match e {
                        execute::ExecuteError::Callback(e) => e,
                        e => Fault::host(e),
                    })
                }) as ResultWork)
            });
            Ok(Box::pin(async move {
                let route = routed.map_err(queue_fault)?;
                route.await.map_err(queue_fault)
            }) as ResultWork)
        }) {
            Ok(route) => Box::pin(async move { route.await.map_err(queue_fault) }),
            Err(e) => ready(Err(queue_fault(e))),
        }
    }
    fn normalize(&self, fault: Fault) -> Fault {
        if matches!(fault, Fault::Host(_)) {
            return fault;
        }
        if let Fault::Other(convert) = &fault {
            return match error_normalize::normalize_other(self.limits.text, || convert()) {
                Ok(error) => Fault::Command(error),
                Err(error_normalize::NormalizeError::Message(error)) => error,
                Err(error) => Fault::host(error),
            };
        }
        let caught = match &fault {
            Fault::Command(e) => Caught::Command(e.clone()),
            Fault::Error { message, .. } => Caught::Error(message),
            Fault::Text(t) => Caught::Text(t),
            Fault::Other(_) => Caught::Other,
            Fault::Host(_) => return fault,
        };
        match error_normalize::normalize(
            &mut self.stale.borrow_mut(),
            caught,
            self.limits.text,
            self.limits.text,
            || {
                let ms = self.host.now()?;
                port_core::js_date::utc_iso_millis(ms).map_err(|_| Fault::Error {
                    name: "RangeError".encode_utf16().collect(),
                    message: "Invalid time value".encode_utf16().collect(),
                })
            },
            || match &fault {
                Fault::Other(convert) => convert(),
                _ => Err(Fault::host("unexpected conversion")),
            },
        ) {
            Ok(error) => Fault::Command(error),
            Err(error) => match error {
                error_normalize::NormalizeError::Clock(e)
                | error_normalize::NormalizeError::Message(e) => e,
                error => Fault::host(error),
            },
        }
    }
    fn guarded(self: Rc<Self>, call: Call, interrupt: bool) -> ResultWork {
        if !interrupt {
            match self.host.busy() {
                Err(e) => return ready(Err(e)),
                Ok(Some(busy)) if !busy.is_empty() => {
                    let mut message: Text = "The session is busy — ".encode_utf16().collect();
                    message.extend(busy);
                    message.extend(". Try again once it has finished.".encode_utf16());
                    return ready(Err(
                        match CommandError::from_utf16(
                            CommandErrorCode::Busy,
                            &message,
                            self.limits.text,
                        ) {
                            Ok(error) => Fault::Command(Arc::new(error)),
                            Err(error) => Fault::host(error),
                        },
                    ));
                }
                _ => {}
            }
            let Some(title) = call.definition.get("title").text() else {
                return ready(Err(Fault::host("invalid title")));
            };
            let since = match self.host.now() {
                Ok(v) => v,
                Err(e) => return ready(Err(e)),
            };
            if let Err(e) = self.activity.publish(Patch::Start(Rc::new(Running {
                name: call.name.clone(),
                title: Rc::from(title),
                since,
            }))) {
                return ready(Err(activity_fault(e)));
            }
            *self.current.borrow_mut() = Some(Current {
                name: call.name.clone(),
                deferred: call.deferred,
            });
        }
        let history = !matches!(call.definition.get("history"), Value::Boolean(false));
        let work = if history {
            // The label expression is evaluated before entering historyAsync's await.
            // Its synchronous throw runs catch/finally before an interrupt returns its promise.
            let label = match self.history_label(&call) {
                Ok(label) => label,
                Err(error) => return ready(self.finish_guarded(Err(error), interrupt)),
            };
            match self.begin_history(&label) {
                Ok(token) => {
                    let timed = self.timed(call.clone(), interrupt);
                    let core = self.clone();
                    Box::pin(async move {
                        let result = timed.await;
                        if let Some(token) = token {
                            let finishing = core
                                .history
                                .borrow_mut()
                                .finish_begin::<Fault>(token)
                                .map_err(history_fault)?;
                            let snapshot = core.host.capture()?;
                            core.history
                                .borrow_mut()
                                .finish_captured(finishing, snapshot)
                                .map_err(history_fault)?;
                        }
                        result
                    }) as ResultWork
                }
                Err(e) => ready(Err(e)),
            }
        } else {
            self.timed(call.clone(), interrupt)
        };
        Box::pin(async move { self.finish_guarded(work.await, interrupt) })
    }
    fn finish_guarded(
        &self,
        result: Result<Value, Fault>,
        interrupt: bool,
    ) -> Result<Value, Fault> {
        let result = result.map_err(|error| self.normalize(error));
        if !interrupt {
            *self.current.borrow_mut() = None;
            self.activity
                .publish(Patch::Clear)
                .map_err(activity_fault)?;
        }
        result
    }
    fn history_label(&self, call: &Call) -> Result<Text, Fault> {
        let mut label = call.callbacks.label.as_ref().map(|label| {
            let label = label.clone();
            move |_: &Call, input: &Value| label(input)
        });
        let label = command_label::history_label(
            call,
            &call.input,
            |call| {
                Ok(call
                    .definition
                    .get("title")
                    .text()
                    .ok_or_else(|| Fault::host("invalid title"))?
                    .to_vec())
            },
            label
                .as_mut()
                .map(|p| p as &mut command_label::Label<'_, Call, Value, Fault>),
            command_label::Limits {
                title_units: self.limits.text,
                output_units: self.limits.text,
            },
        )
        .map_err(|e| match e {
            command_label::LabelError::Callback(e) => e,
            e => Fault::host(e),
        })?;
        Ok(label)
    }
    fn begin_history(&self, label: &[u16]) -> Result<Option<Token>, Fault> {
        {
            let history = self.history.borrow();
            if history.paused() || history.state.restoring {
                return Ok(None);
            }
        }
        let snapshot = self.host.capture()?;
        let workspace = self.host.workspace()?;
        self.history
            .borrow_mut()
            .begin_captured_async(label, &workspace, snapshot)
            .map_err(history_fault)
    }
    fn timed(self: &Rc<Self>, call: Call, interrupt: bool) -> ResultWork {
        let Some(owner) = self.owner.upgrade() else {
            return ready(Err(Fault::host("closed runtime")));
        };
        let host = self.host.clone();
        let ms = match call.definition.get("timeoutMs") {
            Value::Null | Value::Undefined => COMMAND_TIMEOUT_MS as f64,
            Value::Number(n) => *n,
            _ => return ready(Err(Fault::host("invalid timeout metadata"))),
        };
        let limited = match Limited::new(&owner, &call.name, ms, self.limits.text, move |id| {
            host.clear_timer(id)
        }) {
            Ok(v) => v,
            Err(e) => return ready(Err(Fault::host(e))),
        };
        let context = Context {
            state: Rc::new(BodyState {
                limited: limited.clone(),
                factory: Cell::new(true),
                completed: Cell::new(false),
                staged: RefCell::new(None),
                activity: if interrupt {
                    None
                } else {
                    Some(self.activity.clone())
                },
                progress: call.progress.clone(),
                deferred: if !interrupt && call.deferred {
                    Some(true)
                } else {
                    None
                },
                listeners: RefCell::new(Vec::new()),
                listener_limit: self.limits.abort_listeners,
                owner: self.owner.clone(),
            }),
        };
        let signal = Rc::downgrade(&context.state);
        let timer = self.host.timer(
            ms,
            Rc::new(move || {
                if let Some(state) = signal.upgrade() {
                    Context { state }.fire()
                } else {
                    Ok(())
                }
            }),
        );
        if let Err(e) = limited.registered(timer) {
            return ready(Err(Fault::host(e)));
        }
        let result = (call.callbacks.run)(&call.input, context.clone());
        let staged = context.state.staged.borrow_mut().take();
        context.state.factory.set(false);
        match result {
            Err(e) => {
                drop(staged);
                context.state.completed.set(true);
                limited.body_settled(Err(e));
            }
            Ok(()) => {
                if let Some(result) = staged {
                    limited.body_settled(result);
                }
            }
        }
        if let Err(e) = limited.factory_returned() {
            return ready(Err(Fault::host(e)));
        }
        let observer = match limited.observe() {
            Ok(v) => v,
            Err(e) => return ready(Err(Fault::host(e))),
        };
        Box::pin(async move {
            let result = observer.await.map_err(Fault::host);
            drop(context);
            match result? {
                Completion::Body(result) => result,
                Completion::Timeout(error) => Err(Fault::Command(error)),
            }
        })
    }
}
struct Provider {
    core: Rc<Core>,
    call: Call,
}
impl ExecuteHost<'static> for Provider {
    type Value = Value;
    type Error = Fault;
    fn discards(&mut self) -> Result<bool, Fault> {
        match &self.call.callbacks.discards {
            Some(callback) => Ok(matches!(callback(&self.call.input)?, Value::Boolean(true))),
            None => Ok(false),
        }
    }
    fn busy(&mut self) -> Result<Option<Rc<[u16]>>, Fault> {
        self.core.host.busy().map(|v| v.map(Rc::from))
    }
    fn stop(&mut self, done: Done) -> Work<'static, Fault> {
        self.core.host.stop(done)
    }
    fn settle(&mut self) -> Result<Work<'static, Fault>, Fault> {
        self.core.host.settle(&self.call.name)
    }
    fn guarded(&mut self, _: bool) -> ResultWork {
        self.core.clone().guarded(self.call.clone(), false)
    }
}
