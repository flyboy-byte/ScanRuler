// SPDX-License-Identifier: AGPL-3.0-only
//! keptResult on owned acyclic schema data, not arbitrary host JS objects.
use crate::schema_value::{Value, order};
use std::collections::HashSet;
/// Hard native stack-safety ceiling, independent of caller limits.
pub const MAX_DEPTH: usize = 128;
/// Original JSON UTF-16 character threshold, including quotes and punctuation.
pub const RESULT_CHARS: usize = 16_000;
/// Source result property presence and size-only omission.
#[derive(Debug, PartialEq)]
pub enum Kept {
    /// Root undefined has no result property.
    Absent,
    /// Fresh normalized JSON value, independent of the original input tree.
    Result(Value),
    /// Size after source JSON encoding; no truncated result is retained.
    Omitted {
        /// Exact JSON UTF-16 character count, not bytes or input length.
        characters: usize,
    },
}
impl Kept {
    /// Exact original omission text, if a size omitted the result.
    pub fn omitted_message(&self) -> Option<String> {
        match self {
            Self::Omitted { characters } => Some(format!(
                "{characters} characters, past the {RESULT_CHARS} kept"
            )),
            _ => None,
        }
    }
}
/// Native resource/typed-input failures; never the source 'not JSON' result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Caller node/depth/text/byte/encoded-character bound exceeded.
    Limit,
    /// Fallible output or key-set allocation failed.
    Allocation,
    /// Duplicate own fields cannot represent the supported unique-key object domain.
    DuplicateKey,
}
/// Explicit finite work and copy bounds; separate from the original size threshold.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Total input values, including omitted undefined fields.
    pub nodes: usize,
    /// Maximum nested container depth.
    pub depth: usize,
    /// Total input UTF-16 key/string units plus byte payload bytes.
    pub input_units: usize,
    /// Largest encoded JSON size whose exact count the host will compute.
    pub json_units: usize,
}
fn add(a: usize, b: usize, max: usize) -> Result<usize, Error> {
    let n = a.checked_add(b).ok_or(Error::Limit)?;
    if n > max { Err(Error::Limit) } else { Ok(n) }
}
fn validate(
    value: &Value,
    limits: Limits,
    depth: usize,
    nodes: &mut usize,
    units: &mut usize,
) -> Result<(), Error> {
    if depth > limits.depth.min(MAX_DEPTH) {
        return Err(Error::Limit);
    }
    *nodes = add(*nodes, 1, limits.nodes)?;
    match value {
        Value::String(s) => *units = add(*units, s.len(), limits.input_units)?,
        Value::Bytes(b) => {
            if b.len() as u128 > 9_007_199_254_740_991 {
                return Err(Error::Limit);
            }
            *units = add(*units, b.len(), limits.input_units)?
        }
        Value::Array(items) => {
            for item in items {
                validate(item, limits, depth + 1, nodes, units)?
            }
        }
        Value::Object(fields) => {
            if fields.len() > limits.nodes.saturating_sub(*nodes) {
                return Err(Error::Limit);
            }
            let mut keys = HashSet::new();
            keys.try_reserve(fields.len())
                .map_err(|_| Error::Allocation)?;
            for (key, item) in fields {
                *units = add(*units, key.len(), limits.input_units)?;
                if !keys.insert(key.as_slice()) {
                    return Err(Error::DuplicateKey);
                }
                validate(item, limits, depth + 1, nodes, units)?
            }
        }
        _ => {}
    }
    Ok(())
}
fn copy(s: &[u16]) -> Result<Vec<u16>, Error> {
    let mut out = Vec::new();
    out.try_reserve_exact(s.len())
        .map_err(|_| Error::Allocation)?;
    out.extend_from_slice(s);
    Ok(out)
}
fn quoted(s: &[u16], max: usize) -> Result<usize, Error> {
    let mut n = add(0, 2, max)?;
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        let size = match c {
            8 | 9 | 10 | 12 | 13 | 34 | 92 => 2,
            0..=31 => 6,
            0xd800..=0xdbff if i + 1 < s.len() && (0xdc00..=0xdfff).contains(&s[i + 1]) => {
                i += 1;
                2
            }
            0xd800..=0xdfff => 6,
            _ => 1,
        };
        n = add(n, size, max)?;
        i += 1;
    }
    Ok(n)
}
fn normalized(value: &Value, max: usize) -> Result<(Value, usize), Error> {
    match value {
        Value::Undefined | Value::Null => Ok((Value::Null, add(0, 4, max)?)),
        Value::Boolean(b) => Ok((Value::Boolean(*b), add(0, if *b { 4 } else { 5 }, max)?)),
        Value::Number(n) => {
            if !n.is_finite() {
                return Ok((Value::Null, add(0, 4, max)?));
            }
            let n = if *n == 0.0 { 0.0 } else { *n };
            let len = port_core::js_number::number_string(n).len();
            Ok((Value::Number(n), add(0, len, max)?))
        }
        Value::String(s) => Ok((Value::String(copy(s)?), quoted(s, max)?)),
        Value::Bytes(bytes) => {
            let text: Vec<_> = format!("{} bytes", bytes.len()).encode_utf16().collect();
            let size = quoted(&text, max)?;
            Ok((Value::String(text), size))
        }
        Value::Array(items) => {
            let mut out = Vec::new();
            out.try_reserve_exact(items.len())
                .map_err(|_| Error::Allocation)?;
            let mut n = add(0, 2, max)?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    n = add(n, 1, max)?
                }
                let (value, len) = normalized(item, max)?;
                n = add(n, len, max)?;
                out.push(value)
            }
            Ok((Value::Array(out), n))
        }
        Value::Object(fields) => {
            let mut out = Vec::new();
            out.try_reserve_exact(fields.len())
                .map_err(|_| Error::Allocation)?;
            let mut n = add(0, 2, max)?;
            for i in order(fields) {
                let (key, value) = &fields[i];
                if matches!(value, Value::Undefined) {
                    continue;
                }
                if !out.is_empty() {
                    n = add(n, 1, max)?
                }
                n = add(n, quoted(key, max)?, max)?;
                n = add(n, 1, max)?;
                let (value, len) = normalized(value, max)?;
                n = add(n, len, max)?;
                out.push((copy(key)?, value))
            }
            Ok((Value::Object(out), n))
        }
    }
}
/// Copy the supported data exactly as JSON.stringify/replacer/parse does:
/// bytes become count strings, nonfinite doubles and array undefined become
/// null, object undefined is omitted, and -0 becomes +0. Arbitrary getters,
/// toJSON, prototypes, functions and cyclic/shared host graphs are not inputs;
/// no native resource failure is relabelled as the original non-JSON branch.
pub fn keep(value: &Value, limits: Limits) -> Result<Kept, Error> {
    if matches!(value, Value::Undefined) {
        return Ok(Kept::Absent);
    }
    validate(value, limits, 0, &mut 0, &mut 0)?;
    let (value, characters) = normalized(value, limits.json_units)?;
    Ok(if characters > RESULT_CHARS {
        Kept::Omitted { characters }
    } else {
        Kept::Result(value)
    })
}
