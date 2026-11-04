// SPDX-License-Identifier: AGPL-3.0-only
//! Owned registration/catalog/schema-admission stage of pinned registry.ts.
//! Admission returns a body key and coerced arguments; it does not run a command
//! or silently omit the scheduler. Queue/history/timeout integration is separate.
use crate::{
    error::{CommandError, CommandErrorCode, EnvelopeError},
    schema::{self, SchemaError, SourceFault},
    schema_value::{Text, Value},
};
/// Host policy, independent of the original registration schema budget.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Live definitions.
    pub commands: usize,
    /// Retained repeatable cleanup handles.
    pub cleanups: usize,
    /// Aggregate owned definition/cleanup/name units (UTF16, bytes and nodes).
    pub units: usize,
    /// Maximum fully qualified command name.
    pub name_units: usize,
    /// Schema validation/weight budgets.
    pub schema: schema::Limits,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            commands: 512,
            cleanups: 512,
            units: 4_000_000,
            name_units: 4096,
            schema: schema::Limits::default(),
        }
    }
}
/// Original registration errors and admission envelopes stay distinct from host
/// failures and the original validator's synchronous TypeError.
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// Host size/count/recursion refusal, no invented source error.
    Limit,
    /// A fallible reservation failed.
    Allocation,
    /// Definition outside the owned typed Command data boundary.
    InvalidDefinition,
    /// Original registerCommands Error message, lossless UTF16.
    Registration(Text),
    /// Original unknown_command/invalid_input admission refusal.
    Command(CommandError),
    /// Host validator failure.
    Schema(SchemaError),
    /// Original validation threw (empty oneOf); never relabel invalid_input.
    ValidationFault(SourceFault),
    /// Typed error envelope host failure.
    Envelope(EnvelopeError),
}
/// A caller's command implementation key; not a fake application command.
#[derive(Debug, Clone)]
pub struct Command {
    /// Owned data fields of Command (name, title, description, input, flags).
    /// Executable callbacks/shared references are not represented here.
    pub definition: Value,
    /// Lookup key the future session executor uses after admission.
    pub body: usize,
}
/// Repeatable original cleanup closure, opaque to callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cleanup(usize);
/// Input accepted for the next scheduling stage, never an executed command.
#[derive(Debug, Clone, PartialEq)]
pub struct Admitted {
    /// Registered opaque implementation key.
    pub body: usize,
    /// Full coerced input, null/undefined defaulted to an empty object.
    pub input: Value,
}
struct Entry {
    name: Text,
    command: Command,
    units: usize,
}
/// Per-session command registry. No global state or duplicate business-state store.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Entry>,
    cleanups: Vec<Vec<Text>>,
    units: usize,
}
fn txt(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn join(parts: &[&[u16]]) -> Result<Text, RegistryError> {
    let size = parts.iter().try_fold(0usize, |n, p| {
        n.checked_add(p.len()).ok_or(RegistryError::Limit)
    })?;
    let mut v = Vec::new();
    v.try_reserve_exact(size)
        .map_err(|_| RegistryError::Allocation)?;
    for p in parts {
        v.extend_from_slice(p)
    }
    Ok(v)
}
fn valid(name: &[u16]) -> bool {
    let parts: Vec<_> = name.split(|&u| u == 46).collect();
    parts.len() > 1
        && parts.iter().all(|p| {
            p.first().is_some_and(|u| (97..=122).contains(u))
                && p.iter()
                    .all(|u| (97..=122).contains(u) || (48..=57).contains(u) || *u == 95)
        })
}
fn truthy(v: &Value) -> bool {
    match v {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => *b,
        Value::Number(x) => *x != 0. && !x.is_nan(),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}
fn footprint(v: &Value, depth: usize, remaining: &mut usize) -> Result<(), RegistryError> {
    if depth > 64 {
        return Err(RegistryError::Limit);
    }
    let n = match v {
        Value::String(s) => s.len(),
        Value::Bytes(b) => b.len(),
        _ => 0,
    };
    *remaining = remaining
        .checked_sub(n.saturating_add(1))
        .ok_or(RegistryError::Limit)?;
    match v {
        Value::Array(a) => {
            for x in a {
                footprint(x, depth + 1, remaining)?
            }
        }
        Value::Object(a) => {
            for (i, (k, v)) in a.iter().enumerate() {
                *remaining = remaining.checked_sub(k.len()).ok_or(RegistryError::Limit)?;
                if a[..i].iter().any(|(p, _)| p == k) {
                    return Err(RegistryError::InvalidDefinition);
                }
                footprint(v, depth + 1, remaining)?
            }
        }
        _ => {}
    }
    Ok(())
}
fn source(prefix: &str, name: &[u16], suffix: &str) -> RegistryError {
    match join(&[&txt(prefix), name, &txt(suffix)]) {
        Ok(s) => RegistryError::Registration(s),
        Err(e) => e,
    }
}
impl Registry {
    /// New empty per-session registry.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add definitions in order. The original is *not transactional*: a later
    /// source refusal retains earlier additions and yields no cleanup handle.
    /// Old cleanup handles remove those names even if subsequently re-registered.
    pub fn register(
        &mut self,
        namespace: &[u16],
        commands: &[Command],
        limits: Limits,
    ) -> Result<Cleanup, RegistryError> {
        if self.cleanups.len() >= limits.cleanups {
            return Err(RegistryError::Limit);
        }
        self.cleanups
            .try_reserve(1)
            .map_err(|_| RegistryError::Allocation)?;
        let mut added = Vec::new();
        let mut added_units = 0usize;
        added
            .try_reserve_exact(commands.len().min(limits.commands))
            .map_err(|_| RegistryError::Allocation)?;
        for command in commands {
            let d = &command.definition;
            let short = d
                .get("name")
                .text()
                .ok_or(RegistryError::InvalidDefinition)?;
            if namespace
                .len()
                .saturating_add(short.len())
                .saturating_add(1)
                > limits.name_units
            {
                return Err(RegistryError::Limit);
            }
            let name = join(&[namespace, &[46], short])?;
            if !valid(&name) {
                return Err(source(
                    "\"",
                    &name,
                    "\" is not a command name — namespace and verb, dotted, lower case.",
                ));
            }
            if self.entries.iter().any(|e| e.name == name) {
                return Err(source("Two commands share the name \"", &name, "\"."));
            }
            if !d.get("input").get("type").is("object") {
                return Err(source("Command \"", &name, "\" must take an object."));
            }
            let weight = schema::schema_weight(d.get("input"), limits.schema)
                .map_err(RegistryError::Schema)?;
            if weight.bytes > schema::SCHEMA_BUDGET.bytes
                || weight.depth > schema::SCHEMA_BUDGET.depth
            {
                return Err(source(
                    "Command \"",
                    &name,
                    &format!(
                        "\" has an input schema of {} bytes nested {} deep — past 8000 bytes and 12 deep an agent's client may drop it. List the shape loosely and check it when the command runs.",
                        weight.bytes, weight.depth
                    ),
                ));
            }
            if d.get("title").text().is_none() || d.get("description").text().is_none() {
                return Err(RegistryError::InvalidDefinition);
            }
            if self.entries.len() >= limits.commands {
                return Err(RegistryError::Limit);
            }
            let mut remaining = limits.units.saturating_sub(self.units);
            footprint(d, 0, &mut remaining)?;
            let cost = limits.units.saturating_sub(self.units) - remaining;
            let cost = cost.checked_add(name.len()).ok_or(RegistryError::Limit)?;
            let total = self.units.checked_add(cost).ok_or(RegistryError::Limit)?;
            if total
                .checked_add(added_units)
                .and_then(|n| n.checked_add(name.len()))
                .is_none_or(|n| n > limits.units)
            {
                return Err(RegistryError::Limit);
            }
            self.entries
                .try_reserve(1)
                .map_err(|_| RegistryError::Allocation)?;
            added
                .try_reserve(1)
                .map_err(|_| RegistryError::Allocation)?;
            // Definitions are independently owned; their aggregate size was checked.
            let captured = name.clone();
            self.entries.push(Entry {
                name,
                command: command.clone(),
                units: cost,
            });
            added_units += captured.len();
            added.push(captured);
            self.units = total;
        }
        self.units += added_units;
        let handle = Cleanup(self.cleanups.len());
        self.cleanups.push(added);
        Ok(handle)
    }
    /// Capture the admitted implementation/metadata before asynchronous scheduling.
    pub(crate) fn definition(&self, name: &[u16]) -> Option<&Command> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| &entry.command)
    }
    /// Invoke a prior successful cleanup again, preserving its captured names.
    pub fn cleanup(&mut self, handle: Cleanup) -> Result<(), RegistryError> {
        let names = self
            .cleanups
            .get(handle.0)
            .ok_or(RegistryError::InvalidDefinition)?;
        self.entries.retain(|e| {
            if names.contains(&e.name) {
                self.units -= e.units;
                false
            } else {
                true
            }
        });
        Ok(())
    }
    /// Whether this exact registered name was marked headless === true.
    pub fn runs_headless(&self, name: &[u16]) -> bool {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .is_some_and(|e| matches!(e.command.definition.get("headless"), Value::Boolean(true)))
    }
    /// Exact public metadata field order/presence, independently owned snapshots.
    /// Registered names are ASCII. Ordering follows the observed Node22 default
    /// ICU order on that alphabet (underscore, dot, digits, lowercase letters);
    /// broader locale acceptance remains explicit follow-up scope.
    pub fn list(&self) -> Result<Value, RegistryError> {
        fn key(s: &[u16]) -> Vec<u16> {
            s.iter()
                .map(|&c| {
                    if c == 95 {
                        0
                    } else if c == 46 {
                        1
                    } else {
                        c
                    }
                })
                .collect()
        }
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|e| key(&e.name));
        let mut out = Vec::new();
        out.try_reserve_exact(entries.len())
            .map_err(|_| RegistryError::Allocation)?;
        for e in entries {
            let d = &e.command.definition;
            let mut fields = vec![(txt("name"), Value::String(e.name.clone()))];
            for k in ["title", "description", "input"] {
                fields.push((txt(k), d.get(k).clone()));
            }
            for k in ["readOnly", "returnsFile", "returnsImage"] {
                fields.push((txt(k), Value::Boolean(truthy(d.get(k)))));
            }
            for k in ["exact", "long"] {
                if truthy(d.get(k)) {
                    fields.push((txt(k), Value::Boolean(true)));
                }
            }
            for k in ["names", "askForLess"] {
                if truthy(d.get(k)) {
                    fields.push((txt(k), d.get(k).clone()));
                }
            }
            out.push(Value::Object(fields));
        }
        Ok(Value::Array(out))
    }
    /// Perform the synchronous admission prefix of start. No scheduling occurs:
    /// success is the exact body key/coerced args handed to the next stage.
    pub fn admit(
        &self,
        name: &[u16],
        input: &mut Value,
        limits: Limits,
    ) -> Result<Admitted, RegistryError> {
        if name.len() > limits.name_units {
            return Err(RegistryError::Limit);
        }
        let Some(e) = self.entries.iter().find(|e| e.name == name) else {
            let message = join(&[&txt("There is no command \""), name, &txt("\".")])?;
            return Err(RegistryError::Command(
                CommandError::from_utf16(
                    CommandErrorCode::UnknownCommand,
                    &message,
                    limits.schema.output,
                )
                .map_err(RegistryError::Envelope)?,
            ));
        };
        let empty = Value::Object(Vec::new());
        let defaulted = matches!(input, Value::Undefined | Value::Null);
        let given = if defaulted { &empty } else { &*input };
        let validated = schema::validate(
            e.command.definition.get("input"),
            given,
            &txt("input"),
            limits.schema,
        )
        .map_err(RegistryError::Schema)?;
        // Source validate mutates its caller even when it refuses or throws.
        // Nullish input uses a temporary object and leaves the caller nullish.
        if !defaulted {
            *input = validated.value.clone();
        }
        if let Some(f) = validated.fault {
            return Err(RegistryError::ValidationFault(f));
        }
        if !validated.errors.is_empty() {
            let mut message = Vec::new();
            for (i, line) in validated.errors.iter().enumerate() {
                if i > 0 {
                    message.extend(txt("; "));
                }
                message
                    .try_reserve(line.len())
                    .map_err(|_| RegistryError::Allocation)?;
                message.extend_from_slice(line);
            }
            return Err(RegistryError::Command(
                CommandError::from_utf16(
                    CommandErrorCode::InvalidInput,
                    &message,
                    limits.schema.output,
                )
                .map_err(RegistryError::Envelope)?,
            ));
        }
        Ok(Admitted {
            body: e.command.body,
            input: validated.value,
        })
    }
}
