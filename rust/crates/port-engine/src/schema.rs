// SPDX-License-Identifier: AGPL-3.0-only
//! Faithful schema.ts validation/coercion and schema weight over owned data.
//! Limits are host refusals, never schema validation lines. The schema is a
//! well-typed JsonSchema-shaped Value; arbitrary invalid schema types are refused.
use crate::schema_value::{Text, Value, order};

/// Registration JSON length/depth budget; length is UTF-16 units, not UTF-8 bytes.
pub const SCHEMA_BUDGET: Weight = Weight {
    bytes: 8_000,
    depth: 12,
};
/// JSON.stringify(schema).length and nesting of *all* objects/arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weight {
    /// Original field name; counts UTF-16 units.
    pub bytes: usize,
    /// Root object has depth one.
    pub depth: usize,
}
/// Explicit host work/memory policy, separate from the original schema budget.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Combined schema and input nodes.
    pub nodes: usize,
    /// Combined UTF-16 and byte payload units, including keys/path.
    pub units: usize,
    /// Maximum acyclic input/schema nesting.
    pub depth: usize,
    /// Recursive helper invocations and scanned data, bounding oneOf blow-up.
    pub work: usize,
    /// Aggregate generated UTF-16 error/JSON units, including temporary branches.
    pub output: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            nodes: 20_000,
            units: 1_000_000,
            depth: 64,
            work: 2_000_000,
            output: 4_000_000,
        }
    }
}
/// Host refusal; no coerced value is published on a host failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaError {
    /// Caller policy exceeded.
    Limit,
    /// Fallible allocation failed.
    Allocation,
    /// Duplicate keys, or a schema outside the documented JsonSchema data subset.
    InvalidData,
}
/// The one original exception possible for a well-shaped schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFault {
    /// Array.reduce on no failing branches of an empty oneOf.
    EmptyOneOf,
}
/// Complete observable result, including mutations even when validation fails.
#[derive(Debug, Clone, PartialEq)]
pub struct Validation {
    /// Every original refusal line in original order.
    pub errors: Vec<Text>,
    /// Independent owned input copy, with in-place coercions applied sequentially.
    pub value: Value,
    /// Original exception, distinct from a host refusal or an error line.
    pub fault: Option<SourceFault>,
}
#[derive(Debug)]
enum Failure {
    Host(SchemaError),
    Source(SourceFault),
}
impl From<SchemaError> for Failure {
    fn from(e: SchemaError) -> Self {
        Self::Host(e)
    }
}
type R<T> = Result<T, Failure>;
struct Meter {
    work: usize,
    output: usize,
}
impl Meter {
    fn tick(&mut self, n: usize) -> R<()> {
        self.work = self.work.checked_sub(n).ok_or(SchemaError::Limit)?;
        Ok(())
    }
    fn generated(&mut self, n: usize) -> R<()> {
        self.output = self.output.checked_sub(n).ok_or(SchemaError::Limit)?;
        Ok(())
    }
}
fn push<T>(out: &mut Vec<T>, v: T) -> R<()> {
    out.try_reserve(1).map_err(|_| SchemaError::Allocation)?;
    out.push(v);
    Ok(())
}
fn copy<T: Copy>(v: &[T]) -> R<Vec<T>> {
    let mut out = Vec::new();
    out.try_reserve_exact(v.len())
        .map_err(|_| SchemaError::Allocation)?;
    out.extend_from_slice(v);
    Ok(out)
}
fn check(
    v: &Value,
    depth: usize,
    l: &Limits,
    nodes: &mut usize,
    units: &mut usize,
) -> Result<(), SchemaError> {
    *nodes = nodes.checked_sub(1).ok_or(SchemaError::Limit)?;
    if depth > l.depth.min(128) {
        return Err(SchemaError::Limit);
    }
    let mut use_units = |n| {
        *units = units.checked_sub(n).ok_or(SchemaError::Limit)?;
        Ok(())
    };
    match v {
        Value::String(v) => use_units(v.len())?,
        Value::Bytes(v) => use_units(v.len())?,
        Value::Array(v) => {
            for x in v {
                check(x, depth + 1, l, nodes, units)?;
            }
        }
        Value::Object(v) => {
            // Count duplicate-check work in the structural budget: bound fanout
            // before the quadratic check, independently of the recursion meter.
            if v.len().saturating_mul(v.len()) > l.work {
                return Err(SchemaError::Limit);
            }
            for (i, (k, _)) in v.iter().enumerate() {
                use_units(k.len())?;
                if v[..i].iter().any(|(a, _)| a == k) {
                    return Err(SchemaError::InvalidData);
                }
            }
            for (_, x) in v {
                check(x, depth + 1, l, nodes, units)?;
            }
        }
        _ => {}
    };
    Ok(())
}
fn owned(v: &Value) -> R<Value> {
    Ok(match v {
        Value::String(v) => Value::String(copy(v)?),
        Value::Bytes(v) => Value::Bytes(copy(v)?),
        Value::Array(v) => {
            let mut a = Vec::new();
            for x in v {
                push(&mut a, owned(x)?)?;
            }
            Value::Array(a)
        }
        Value::Object(v) => {
            let mut a = Vec::new();
            for (k, x) in v {
                push(&mut a, (copy(k)?, owned(x)?))?;
            }
            Value::Object(a)
        }
        Value::Number(v) => Value::Number(*v),
        Value::Boolean(v) => Value::Boolean(*v),
        Value::Null => Value::Null,
        Value::Undefined => Value::Undefined,
    })
}
fn host(e: Failure) -> SchemaError {
    match e {
        Failure::Host(e) => e,
        Failure::Source(_) => SchemaError::InvalidData,
    }
}
fn preflight(
    schema: &Value,
    value: &Value,
    path: &[u16],
    limits: &Limits,
) -> Result<(), SchemaError> {
    let mut nodes = limits.nodes;
    let mut units = limits
        .units
        .checked_sub(path.len())
        .ok_or(SchemaError::Limit)?;
    check(schema, 0, limits, &mut nodes, &mut units)?;
    check(value, 0, limits, &mut nodes, &mut units)?;
    shape(schema)
}
fn shape(s: &Value) -> Result<(), SchemaError> {
    if !matches!(s, Value::Object(_)) {
        return Err(SchemaError::InvalidData);
    }
    let t = s.get("type");
    if !matches!(t, Value::Undefined)
        && ![
            "object", "array", "string", "number", "integer", "boolean", "null",
        ]
        .iter()
        .any(|x| t.is(x))
    {
        return Err(SchemaError::InvalidData);
    }
    for key in [
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "minLength",
        "minItems",
        "maxItems",
    ] {
        if !matches!(s.get(key), Value::Undefined | Value::Number(_)) {
            return Err(SchemaError::InvalidData);
        }
    }
    for key in ["title", "format", "description"] {
        if !matches!(s.get(key), Value::Undefined | Value::String(_)) {
            return Err(SchemaError::InvalidData);
        }
    }
    if !matches!(
        s.get("additionalProperties"),
        Value::Undefined | Value::Boolean(_)
    ) {
        return Err(SchemaError::InvalidData);
    }
    for key in ["enum", "required", "oneOf"] {
        if !matches!(s.get(key), Value::Undefined | Value::Array(_)) {
            return Err(SchemaError::InvalidData);
        }
    }
    if let Some(a) = s.get("required").list()
        && a.iter().any(|x| !matches!(x, Value::String(_)))
    {
        return Err(SchemaError::InvalidData);
    }
    if let Some(a) = s.get("enum").list()
        && a.iter().any(|x| {
            !matches!(
                x,
                Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null
            )
        })
    {
        return Err(SchemaError::InvalidData);
    }
    if let Some(a) = s.get("oneOf").list() {
        for x in a {
            shape(x)?;
        }
    }
    if !matches!(s.get("items"), Value::Undefined) {
        shape(s.get("items"))?;
    }
    match s.get("properties") {
        Value::Undefined => {}
        Value::Object(a) => {
            for (_, x) in a {
                shape(x)?;
            }
        }
        _ => return Err(SchemaError::InvalidData),
    }
    match s.get("names") {
        Value::Undefined => {}
        n @ Value::Object(_) => {
            if n.get("field").text().is_none()
                || !n
                    .get("slots")
                    .list()
                    .is_some_and(|a| a.iter().all(|x| x.text().is_some()))
            {
                return Err(SchemaError::InvalidData);
            }
        }
        _ => return Err(SchemaError::InvalidData),
    }
    Ok(())
}
/// Validate independent copies; return the original's complete error and mutation
/// state. Top-level string booleans are not coerced (only fields/list items are).
pub fn validate(
    schema: &Value,
    value: &Value,
    path: &[u16],
    limits: Limits,
) -> Result<Validation, SchemaError> {
    preflight(schema, value, path, &limits)?;
    let mut value = owned(value).map_err(host)?;
    let mut meter = Meter {
        work: limits.work,
        output: limits.output,
    };
    match run(schema, &mut value, path, None, &mut meter, 0) {
        Ok(errors) => Ok(Validation {
            errors,
            value,
            fault: None,
        }),
        Err(Failure::Source(fault)) => Ok(Validation {
            errors: Vec::new(),
            value,
            fault: Some(fault),
        }),
        Err(Failure::Host(e)) => Err(e),
    }
}
#[derive(Clone, Copy)]
struct Names<'a> {
    field: &'a [u16],
    slots: &'a [Value],
    at: &'a [u16],
}
fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn joined(parts: &[&[u16]], m: &mut Meter) -> R<Text> {
    let size = parts.iter().try_fold(0usize, |n, p| {
        n.checked_add(p.len()).ok_or(SchemaError::Limit)
    })?;
    m.generated(size)?;
    let mut out = Vec::new();
    out.try_reserve_exact(size)
        .map_err(|_| SchemaError::Allocation)?;
    for p in parts {
        out.extend_from_slice(p);
    }
    Ok(out)
}
fn suffix(path: &[u16], s: &str, m: &mut Meter) -> R<Text> {
    joined(&[path, &text(s)], m)
}
fn line(path: &[u16], s: &str, m: &mut Meter) -> R<Vec<Text>> {
    let mut a = Vec::new();
    push(&mut a, suffix(path, s, m)?)?;
    Ok(a)
}
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y || (x.is_nan() && y.is_nan()),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}
fn takes(s: &Value, kind: &str, m: &mut Meter) -> R<bool> {
    m.tick(1)?;
    if s.get("type").is(kind) || (kind == "number" && s.get("type").is("integer")) {
        return Ok(true);
    }
    if let Some(a) = s.get("oneOf").list() {
        for b in a {
            if takes(b, kind, m)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn fits(s: &Value, v: &Value, m: &mut Meter) -> R<bool> {
    m.tick(1)?;
    if let Some(a) = s.get("oneOf").list() {
        for b in a {
            if fits(b, v, m)? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    let t = s.get("type");
    Ok(if matches!(t, Value::Undefined) {
        s.get("enum").list().is_none_or(|a| {
            a.iter().any(|x| {
                matches!(
                    (x, v),
                    (Value::Null, Value::Null)
                        | (Value::Boolean(_), Value::Boolean(_))
                        | (Value::Number(_), Value::Number(_))
                        | (Value::String(_), Value::String(_))
                )
            })
        })
    } else if t.is("null") {
        matches!(v, Value::Null)
    } else if t.is("boolean") {
        matches!(v, Value::Boolean(_))
    } else if t.is("string") {
        if s.get("format").is("binary") {
            matches!(v, Value::Bytes(_))
        } else {
            matches!(v, Value::String(_))
        }
    } else if t.is("number") || t.is("integer") {
        matches!(v, Value::Number(_))
    } else if t.is("array") {
        matches!(v, Value::Array(_))
    } else {
        matches!(v, Value::Object(_))
    })
}
fn as_boolean(s: &Value, v: &mut Value, m: &mut Meter, depth: usize) -> R<()> {
    if !(v.is("true") || v.is("false")) || !takes(s, "boolean", m)? {
        return Ok(());
    }
    if run(s, v, &text("input"), None, m, depth + 1)?.is_empty() {
        return Ok(());
    }
    let mut flag = Value::Boolean(v.is("true"));
    if run(s, &mut flag, &text("input"), None, m, depth + 1)?.is_empty() {
        *v = flag;
    }
    Ok(())
}
fn name_of(s: &Value, m: &mut Meter) -> R<Text> {
    if let Some(t) = s.get("title").text().filter(|x| !x.is_empty()) {
        return copy(t);
    }
    if s.get("type").is("object")
        && let Some(a) = s.get("required").list().filter(|a| a.len() == 1)
    {
        return joined(
            &[
                &text("{ "),
                a[0].text().ok_or(SchemaError::InvalidData)?,
                &text(" }"),
            ],
            m,
        );
    }
    copy(s.get("type").text().unwrap_or(&text("value")))
}
fn names_list(a: &[Value], m: &mut Meter) -> R<Text> {
    let mut out = Vec::new();
    for (i, b) in a.iter().enumerate() {
        if i > 0 {
            out.extend(text(", "));
        }
        out.extend(name_of(b, m)?);
    }
    m.generated(out.len())?;
    Ok(out)
}
fn run<'a>(
    s: &'a Value,
    v: &mut Value,
    path: &'a [u16],
    within: Option<Names<'a>>,
    m: &mut Meter,
    depth: usize,
) -> R<Vec<Text>> {
    m.tick(1 + path.len())?;
    if depth > 256 {
        return Err(SchemaError::Limit.into());
    }
    let n = s.get("names");
    let names = if matches!(n, Value::Undefined) {
        within
    } else {
        Some(Names {
            field: n.get("field").text().ok_or(SchemaError::InvalidData)?,
            slots: n.get("slots").list().ok_or(SchemaError::InvalidData)?,
            at: path,
        })
    };
    if let Some(a) = s.get("oneOf").list() {
        let mut results = Vec::new();
        for b in a {
            push(&mut results, run(b, v, path, names, m, depth + 1)?)?;
        }
        let passing = results.iter().filter(|r| r.is_empty()).count();
        if passing == 1 {
            return Ok(Vec::new());
        }
        let labels = names_list(a, m)?;
        if passing > 1 {
            let mut out = Vec::new();
            push(
                &mut out,
                joined(&[path, &text(": matches more than one of "), &labels], m)?,
            )?;
            return Ok(out);
        }
        let mut fitting = Vec::new();
        for (i, b) in a.iter().enumerate() {
            if fits(b, v, m)? {
                push(&mut fitting, i)?;
            }
        }
        let pool: Vec<_> = if fitting.is_empty() {
            (0..results.len()).collect()
        } else {
            fitting
        };
        let mut closest = *pool
            .first()
            .ok_or(Failure::Source(SourceFault::EmptyOneOf))?;
        for &i in &pool {
            if results[i].len() < results[closest].len() {
                closest = i
            }
        }
        let best = results.swap_remove(closest);
        if a.len() == 1 {
            return Ok(best);
        }
        let mut out = Vec::new();
        push(
            &mut out,
            joined(&[path, &text(": expected one of "), &labels], m)?,
        )?;
        for e in best {
            push(&mut out, e)?;
        }
        return Ok(out);
    }
    if let Some(a) = s.get("enum").list() {
        m.tick(a.len())?;
        if !a.iter().any(|e| same(e, v)) {
            let mut choices = Vec::new();
            for (i, e) in a.iter().enumerate() {
                if i > 0 {
                    choices.extend(text(", "));
                }
                choices.extend(json(e, m)?);
            }
            let mut out = Vec::new();
            push(
                &mut out,
                joined(&[path, &text(": expected one of "), &choices], m)?,
            )?;
            return Ok(out);
        }
    }
    let t = s.get("type");
    if matches!(t, Value::Undefined) {
        return Ok(Vec::new());
    }
    if t.is("null") {
        return if matches!(v, Value::Null) {
            Ok(Vec::new())
        } else {
            line(path, ": expected null", m)
        };
    }
    if t.is("boolean") {
        return if matches!(v, Value::Boolean(_)) {
            Ok(Vec::new())
        } else {
            line(path, ": expected true or false", m)
        };
    }
    if t.is("string") {
        if s.get("format").is("binary") {
            return if matches!(v, Value::Bytes(_)) {
                Ok(Vec::new())
            } else {
                line(path, ": expected the file's bytes", m)
            };
        }
        let Value::String(x) = v else {
            return line(path, ": expected a string", m);
        };
        if let Some(n) = s.get("minLength").number()
            && (x.len() as f64) < n
        {
            return line(
                path,
                &format!(
                    ": expected at least {} character{}",
                    number(n),
                    if n == 1. { "" } else { "s" }
                ),
                m,
            );
        }
        return Ok(Vec::new());
    }
    if t.is("number") || t.is("integer") {
        let Value::Number(x) = v else {
            return line(path, ": expected a finite number", m);
        };
        if !x.is_finite() {
            return line(path, ": expected a finite number", m);
        }
        if t.is("integer") && x.trunc() != *x {
            return line(path, ": expected a whole number", m);
        }
        for (key, phrase, exclusive) in [
            ("minimum", "at least", false),
            ("exclusiveMinimum", "more than", true),
            ("maximum", "at most", false),
        ] {
            if let Some(n) = s.get(key).number()
                && if key == "maximum" {
                    *x > n
                } else if exclusive {
                    *x <= n
                } else {
                    *x < n
                }
            {
                return line(path, &format!(": must be {phrase} {}", number(n)), m);
            }
        }
        return Ok(Vec::new());
    }
    if t.is("array") {
        let Value::Array(a) = v else {
            return line(path, ": expected a list", m);
        };
        for (key, phrase) in [("minItems", "at least"), ("maxItems", "at most")] {
            if let Some(n) = s.get(key).number()
                && if key == "minItems" {
                    (a.len() as f64) < n
                } else {
                    (a.len() as f64) > n
                }
            {
                return line(
                    path,
                    &format!(
                        ": expected {phrase} {} item{}",
                        number(n),
                        if n == 1. { "" } else { "s" }
                    ),
                    m,
                );
            }
        }
        let item = s.get("items");
        if matches!(item, Value::Undefined) {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for (i, x) in a.iter_mut().enumerate() {
            as_boolean(item, x, m, depth)?;
            let p = suffix(path, &format!("[{i}]"), m)?;
            for e in run(item, x, &p, names, m, depth + 1)? {
                push(&mut out, e)?;
            }
        }
        return Ok(out);
    }
    let Value::Object(a) = v else {
        return line(path, ": expected an object", m);
    };
    let required = s.get("required").list().unwrap_or(&[]);
    let properties = s.get("properties");
    let mut out = Vec::new();
    m.tick(a.len().saturating_mul(required.len() + 1))?;
    for key in required {
        let key = key.text().ok_or(SchemaError::InvalidData)?;
        if a.iter()
            .find(|(k, _)| k == key)
            .map_or_else(|| !inherited(key), |(_, v)| matches!(v, Value::Undefined))
        {
            push(
                &mut out,
                joined(&[path, &text("."), key, &text(": required")], m)?,
            )?;
        }
    }
    for i in order(a) {
        let (key, x) = &mut a[i];
        if matches!(x, Value::Undefined) {
            continue;
        }
        let own = if let Value::Object(p) = properties {
            p.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        };
        // properties?.[key] uses ordinary Object.prototype even for an unknown
        // own field. Its inherited functions/Object.prototype have no schema type.
        let empty = Value::Object(Vec::new());
        let prop = own.or_else(|| {
            (!matches!(properties, Value::Undefined) && inherited(key)).then_some(&empty)
        });
        let p = joined(&[path, &text("."), key], m)?;
        if let Some(prop) = prop {
            as_boolean(prop, x, m, depth)?;
            let req = required.iter().any(|r| r.text() == Some(key));
            if let Some(named) = name_given(names, &p, prop, x, req, m, depth)? {
                push(&mut out, named)?;
            } else if let Some(named) = name_id(s, key, &p, x, m)? {
                push(&mut out, named)?;
            } else {
                for e in run(prop, x, &p, names, m, depth + 1)? {
                    push(&mut out, e)?;
                }
            }
        } else if matches!(s.get("additionalProperties"), Value::Boolean(false)) {
            let mut keys = Vec::new();
            if let Value::Object(a) = properties {
                for (j, i) in order(a).iter().enumerate() {
                    if j > 0 {
                        keys.extend(text(", "));
                    }
                    keys.extend_from_slice(&a[*i].0);
                }
            }
            if keys.is_empty() {
                keys = text("none")
            }
            push(
                &mut out,
                joined(
                    &[&p, &text(": not a known field — the fields here: "), &keys],
                    m,
                )?,
            )?;
        }
    }
    Ok(out)
}
fn name_given(
    names: Option<Names<'_>>,
    path: &[u16],
    s: &Value,
    v: &mut Value,
    required: bool,
    m: &mut Meter,
    depth: usize,
) -> R<Option<Text>> {
    let Some(n) = names else { return Ok(None) };
    if !matches!(v, Value::String(_)) || !takes(s, "number", m)? {
        return Ok(None);
    }
    if !path.starts_with(n.at) || path.get(n.at.len()) != Some(&46) {
        return Ok(None);
    }
    let slot = &path[n.at.len() + 1..];
    if !n.slots.iter().any(|v| v.text() == Some(slot)) {
        return Ok(None);
    }
    if run(s, v, &text("input"), None, m, depth + 1)?.is_empty() {
        return Ok(None);
    }
    let q = json(v, m)?;
    let slot = if slot.contains(&46) {
        json(&Value::String(copy(slot)?), m)?
    } else {
        copy(slot)?
    };
    Ok(Some(joined(
        &[
            path,
            &text(": "),
            &q,
            &text(" is a name or an expression, and this field takes a number — give "),
            n.field,
            &text(": { "),
            &slot,
            &text(": "),
            &q,
            &text(" }"),
            &text(if required {
                ", with any number here (the expression sets it)"
            } else {
                ""
            }),
        ],
        m,
    )?))
}
fn name_id(s: &Value, key: &[u16], path: &[u16], v: &Value, m: &mut Meter) -> R<Option<Text>> {
    let Some(value) = v.text() else {
        return Ok(None);
    };
    if key != text("id") || js_trim(value).is_empty() || numeric_string(value) {
        return Ok(None);
    }
    let p = s.get("properties");
    let id = p.get("id");
    let name = p.get("name");
    if matches!(id, Value::Undefined)
        || matches!(name, Value::Undefined)
        || fits(id, v, m)?
        || !fits(name, v, m)?
    {
        return Ok(None);
    }
    let q = json(v, m)?;
    Ok(Some(joined(
        &[
            path,
            &text(": "),
            &q,
            &text(" is a name, and id takes a number — a name goes in name: give name: "),
            &q,
            &text(" in place of id"),
        ],
        m,
    )?))
}
fn whitespace(c: u16) -> bool {
    matches!(c,9..=13|32|160|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn js_trim(v: &[u16]) -> &[u16] {
    let start = v.iter().position(|&c| !whitespace(c)).unwrap_or(v.len());
    let end = v
        .iter()
        .rposition(|&c| !whitespace(c))
        .map_or(start, |i| i + 1);
    &v[start..end]
}
fn numeric_string(v: &[u16]) -> bool {
    let v = js_trim(v);
    if v.is_empty() {
        return true;
    }
    let Ok(s) = String::from_utf16(v) else {
        return false;
    };
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0b", 2),
        ("0B", 2),
        ("0o", 8),
        ("0O", 8),
    ] {
        if let Some(s) = s.strip_prefix(prefix) {
            if s.is_empty() {
                return false;
            }
            let mut n = 0.;
            for c in s.chars() {
                let Some(d) = c.to_digit(radix) else {
                    return false;
                };
                n = n * f64::from(radix) + f64::from(d);
            }
            return n.is_finite();
        }
    }
    // Rust accepts inf/NaN; neither is a finite JavaScript Number(string).
    if !s
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
    {
        return false;
    }
    s.parse::<f64>().is_ok_and(f64::is_finite)
}
// Same shortest-decimal layout rule as the existing original PLY error helper;
// no new dependency or cross-crate coupling. Native std shortest digit selection
// is compared on fixed boundaries and seeded doubles; exhaustive double-space
// equivalence is not claimed.
fn number(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x == 0. {
        return "0".into();
    }
    if x.is_infinite() {
        return if x < 0. { "-Infinity" } else { "Infinity" }.into();
    }
    let sci = format!("{:e}", x.abs());
    let (m, e) = sci.split_once('e').unwrap_or((&sci, "0"));
    let digits = m.replace('.', "");
    let k = digits.len() as i32;
    let n = e.parse::<i32>().unwrap_or(0) + 1;
    let body = if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let e = n - 1;
        let es = if e < 0 {
            e.to_string()
        } else {
            format!("+{e}")
        };
        if k == 1 {
            format!("{digits}e{es}")
        } else {
            format!("{}.{}e{es}", &digits[..1], &digits[1..])
        }
    };
    format!("{}{body}", if x < 0. { "-" } else { "" })
}
fn json(v: &Value, m: &mut Meter) -> R<Text> {
    m.tick(1)?;
    let mut out = Vec::new();
    match v {
        Value::Undefined | Value::Null => out = text("null"),
        Value::Boolean(b) => out = text(if *b { "true" } else { "false" }),
        Value::Number(x) => {
            out = text(&if x.is_finite() {
                number(*x)
            } else {
                "null".into()
            })
        }
        Value::String(s) => {
            push(&mut out, 34)?;
            let mut i = 0;
            while i < s.len() {
                let c = s[i];
                let escape = match c {
                    34 => Some("\\\"".to_string()),
                    92 => Some("\\\\".to_string()),
                    8 => Some("\\b".to_string()),
                    9 => Some("\\t".to_string()),
                    10 => Some("\\n".to_string()),
                    12 => Some("\\f".to_string()),
                    13 => Some("\\r".to_string()),
                    0..=31 => Some(format!("\\u{c:04x}")),
                    0xd800..=0xdbff
                        if s.get(i + 1).is_some_and(|c| (0xdc00..=0xdfff).contains(c)) =>
                    {
                        push(&mut out, c)?;
                        i += 1;
                        push(&mut out, s[i])?;
                        i += 1;
                        continue;
                    }
                    0xd800..=0xdfff => Some(format!("\\u{c:04x}")),
                    _ => None,
                };
                if let Some(e) = escape {
                    out.extend(text(&e))
                } else {
                    push(&mut out, c)?;
                }
                i += 1;
            }
            push(&mut out, 34)?;
        }
        Value::Array(a) => {
            push(&mut out, 91)?;
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    push(&mut out, 44)?;
                }
                out.extend(json(x, m)?);
            }
            push(&mut out, 93)?;
        }
        Value::Object(a) => {
            push(&mut out, 123)?;
            let mut first = true;
            for i in order(a) {
                let (k, x) = &a[i];
                if matches!(x, Value::Undefined) {
                    continue;
                }
                if !first {
                    push(&mut out, 44)?;
                }
                first = false;
                out.extend(json(&Value::String(copy(k)?), m)?);
                push(&mut out, 58)?;
                out.extend(json(x, m)?);
            }
            push(&mut out, 125)?;
        }
        Value::Bytes(a) => {
            push(&mut out, 123)?;
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    push(&mut out, 44)?;
                }
                out.extend(text(&format!("\"{i}\":{x}")));
            }
            push(&mut out, 125)?;
        }
    }
    m.generated(out.len())?;
    Ok(out)
}
fn nesting(v: &Value) -> usize {
    match v {
        Value::Object(a) => 1 + a.iter().map(|(_, v)| nesting(v)).max().unwrap_or(0),
        Value::Array(a) => 1 + a.iter().map(nesting).max().unwrap_or(0),
        Value::Bytes(_) => 1,
        _ => 0,
    }
}
/// Compute the complete schema's JSON weight (including ignored metadata and
/// default values), without enforcing the registration budget itself.
pub fn schema_weight(schema: &Value, limits: Limits) -> Result<Weight, SchemaError> {
    preflight(schema, &Value::Undefined, &[], &limits)?;
    let mut m = Meter {
        work: limits.work,
        output: limits.output,
    };
    Ok(Weight {
        bytes: json(schema, &mut m).map_err(host)?.len(),
        depth: nesting(schema),
    })
}

fn inherited(key: &[u16]) -> bool {
    [
        "constructor",
        "__defineGetter__",
        "__defineSetter__",
        "hasOwnProperty",
        "__lookupGetter__",
        "__lookupSetter__",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toString",
        "valueOf",
        "__proto__",
        "toLocaleString",
    ]
    .iter()
    .any(|s| key.iter().copied().eq(s.encode_utf16()))
}
