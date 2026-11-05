// SPDX-License-Identifier: AGPL-3.0-only
//! Global history over the actual native document/recorder for the supported
//! no-viewport registry session. UI/project/family rebuild hooks remain explicit.
use super::*;
use crate::history_record::{
    Patch,
    restore::{Hook, RestoreFuture, RestoreHost, View},
};
#[path = "native_history_definition.rs"]
mod definition;
struct NativeRestore {
    state: Rc<State>,
    host: Rc<NativeHost>,
}
fn error_text(f: Fault) -> Text {
    match f {
        Fault::Command(e) => e.message_units().to_vec(),
        Fault::Text(t) | Fault::Error { message: t, .. } => t,
        _ => text(&format!("{f:?}")),
    }
}
impl RestoreHost for NativeRestore {
    type Error = Text;
    fn participants(&self) -> usize {
        0
    }
    fn capture(&mut self, _: View) -> Result<Snapshot, Text> {
        self.host.capture().map_err(error_text)
    }
    fn hook<'a>(
        &'a mut self,
        _: Hook,
        _: &'a Patch,
        _: &'a Snapshot,
        _: View,
    ) -> RestoreFuture<'a, Result<(), Text>> {
        Box::pin(std::future::ready(Ok(())))
    }
    fn apply(&mut self, patch: &Patch, _: View) -> Result<(), Text> {
        for (key, value) in patch {
            if *key != text("measure") {
                return Err(text(
                    "Native history restoration for this family is not implemented.",
                ));
            }
            let C::Object(fields) = value else {
                return Err(text("Invalid native measure patch."));
            };
            self.state
                .document
                .store(StoreId::App)
                .patch((**fields).clone())
                .map_err(error_text)?;
        }
        Ok(())
    }
    fn workspace(&mut self, workspace: &[u16]) -> Result<(), Text> {
        if workspace == text("elements") {
            Ok(())
        } else {
            Err(text("Native history workspace is not implemented."))
        }
    }
    fn status(&mut self, status: Text) -> Result<(), Text> {
        patch(
            &self.state.document,
            vec![("statusText", C::String(status))],
        )
        .map_err(error_text)
    }
    fn clear_busy(&mut self) {
        if let Err(e) = patch(&self.state.document, vec![("busy", C::Boolean(false))]) {
            self.host.observer_fault(e)
        }
    }
    fn error(&mut self, error: Text) {
        if let Err(e) = patch(&self.state.document, vec![("errorText", C::String(error))]) {
            self.host.observer_fault(e)
        }
    }
}
fn bool_field(doc: &Session, store: StoreId, key: &str) -> bool {
    matches!(doc.store(store).get(key), C::Boolean(true))
}
fn running(doc: &Session, store: StoreId, key: &str) -> bool {
    matches!(doc.store(store).get(key),C::String(s) if s==text("running"))
}
fn count(doc: &Session, key: &str) -> Value {
    Value::Number(match doc.store(StoreId::App).get(key) {
        C::Array(v) => v.len() as f64,
        _ => 0.,
    })
}
fn label(entries: &[crate::history_record::Entry]) -> Value {
    entries
        .last()
        .map_or(Value::Null, |e| Value::String(e.label.clone()))
}
pub(super) fn register(
    runtime: &Runtime,
    state: &Rc<State>,
    host: &Rc<NativeHost>,
) -> Result<(), Fault> {
    for redo in [false, true] {
        let state = Rc::downgrade(state);
        let host = Rc::downgrade(host);
        let weak = runtime.downgrade();
        runtime.register(
            &text("history"),
            if redo {
                definition::redo()
            } else {
                definition::undo()
            },
            Callbacks {
                label: None,
                discards: None,
                run: Rc::new(move |input, ctx| {
                    let runtime = weak
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed native runtime"))?;
                    let state = state
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed native document"))?;
                    let host = host
                        .upgrade()
                        .ok_or_else(|| Fault::host("closed native history host"))?;
                    start(runtime, state, host, redo, input, ctx)
                }),
            },
        )?;
    }
    Ok(())
}
fn start(
    runtime: Runtime,
    state: Rc<State>,
    host: Rc<NativeHost>,
    redo: bool,
    input: &Value,
    ctx: CommandContext,
) -> Result<(), Fault> {
    let h = runtime.history();
    let entry = if redo { h.future.last() } else { h.past.last() }
        .cloned()
        .ok_or_else(|| {
            command_error(
                CommandErrorCode::InvalidState,
                if redo {
                    "There is nothing to redo."
                } else {
                    "There is nothing to undo."
                },
            )
        })?;

    let blocked = h.restoring
        || runtime.history_paused()
        || state.imports.busy()
        || bool_field(&state.document, StoreId::App, "busy")
        || bool_field(&state.document, StoreId::Deviation, "nominalBusy")
        || running(&state.document, StoreId::Deviation, "alignStatus")
        || running(&state.document, StoreId::Deviation, "mapStatus")
        || running(&state.document, StoreId::Thickness, "status")
        || plane::measure_free(&state.document).is_err();

    if blocked {
        return Err(command_error(
            CommandErrorCode::InvalidState,
            &format!(
                "Something open in a panel is in the way of {} — finish or cancel it first.",
                if redo { "redo" } else { "undo" }
            ),
        ));
    }
    if matches!(get(input,"detail"),Some(Value::String(s)) if *s==text("full")) {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native full session.state output is not implemented yet.",
        ));
    }
    if entry
        .before
        .iter()
        .chain(entry.after.iter())
        .any(|(k, _)| *k != text("measure"))
        || entry.workspace != text("elements")
    {
        return Err(command_error(
            CommandErrorCode::Unavailable,
            "Native restoration for this history family is not implemented yet.",
        ));
    }

    ctx.spawn(finish(runtime, state, host, redo, entry))?;
    Ok(())
}
async fn finish(
    runtime: Runtime,
    state: Rc<State>,
    host: Rc<NativeHost>,
    redo: bool,
    entry: crate::history_record::Entry,
) -> Result<Value, Fault> {
    runtime.restore_native_history(
        &mut NativeRestore {
            state: state.clone(),
            host,
        },
        redo,
    )?;
    let h = runtime.history();
    let moved = if redo { h.past.last() } else { h.future.last() };
    if moved.is_none_or(|m| {
        m.group != entry.group
            || m.label != entry.label
            || m.workspace != entry.workspace
            || m.before.len() != entry.before.len()
            || m.after.len() != entry.after.len()
            || !m
                .before
                .iter()
                .zip(&entry.before)
                .chain(m.after.iter().zip(&entry.after))
                .all(|((ka, a), (kb, b))| ka == kb && a.strict_equal(b))
    }) {
        let mut message = text("The step \"");
        message.extend_from_slice(&entry.label);
        message.extend(text(if redo {
            "\" could not be redone."
        } else {
            "\" could not be undone."
        }));
        return Err(Fault::Command(Arc::new(
            CommandError::from_utf16(CommandErrorCode::Failed, &message, 1_000_000)
                .map_err(Fault::host)?,
        )));
    }

    Ok(object(vec![
        (
            if redo { "redone" } else { "undone" },
            Value::String(entry.label),
        ),
        ("workspace", value_text("measure")),
        (
            "after",
            object(vec![
                ("elements", count(&state.document, "elements")),
                ("dimensions", count(&state.document, "dimensions")),
                ("sections", count(&state.document, "sections")),
                (
                    "history",
                    object(vec![("undo", label(&h.past)), ("redo", label(&h.future))]),
                ),
            ]),
        ),
    ]))
}
