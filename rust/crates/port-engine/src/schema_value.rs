// SPDX-License-Identifier: AGPL-3.0-only
//! Owned acyclic JavaScript data needed by command validation. Strings and keys
//! are UTF-16, objects store unique own enumerable keys in insertion order.
//! Ordinary Object.prototype lookups are modelled by validation. No getters,
//! proxies, custom prototypes, functions or shared/cyclic references.

/// Lossless UTF-16 string.
pub type Text = Vec<u16>;
/// Data, not JSON: undefined, bytes and non-finite numbers remain distinct.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A missing or explicitly undefined field.
    Undefined,
    /// JSON null.
    Null,
    /// Boolean.
    Boolean(bool),
    /// JavaScript IEEE-754 double (including signed zero, infinities and NaN).
    Number(f64),
    /// UTF-16, including lone surrogates.
    String(Text),
    /// Dense list (an explicit undefined item is not a hole).
    Array(Vec<Value>),
    /// Unique own enumerable fields, insertion order. Validation rejects duplicates.
    Object(Vec<(Text, Value)>),
    /// Uint8Array, never an ordinary array or object to the validator.
    Bytes(Vec<u8>),
}
impl Value {
    pub(crate) fn get(&self, key: &str) -> &Self {
        if let Self::Object(fields) = self
            && let Some((_, v)) = fields
                .iter()
                .find(|(k, _)| k.iter().copied().eq(key.encode_utf16()))
        {
            return v;
        }
        &Self::Undefined
    }
    pub(crate) fn list(&self) -> Option<&[Self]> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub(crate) fn text(&self) -> Option<&[u16]> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub(crate) fn number(&self) -> Option<f64> {
        if let Self::Number(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub(crate) fn is(&self, s: &str) -> bool {
        self.text()
            .is_some_and(|v| v.iter().copied().eq(s.encode_utf16()))
    }
}
/// JavaScript own-key order: array-index strings first, then insertion order.
/// The caller has bounded and checked the object before this helper is reached.
pub(crate) fn order(fields: &[(Text, Value)]) -> Vec<usize> {
    fn index(k: &[u16]) -> Option<u32> {
        if k.is_empty() || (k.len() > 1 && k[0] == 48) {
            return None;
        }
        let mut n = 0u32;
        for &c in k {
            if !(48..=57).contains(&c) {
                return None;
            }
            n = n.checked_mul(10)?.checked_add(u32::from(c - 48))?;
        }
        (n < u32::MAX).then_some(n)
    }
    let mut ids: Vec<_> = (0..fields.len()).collect();
    ids.sort_by_key(|&i| {
        (
            index(&fields[i].0).is_none(),
            index(&fields[i].0).unwrap_or(0),
        )
    });
    ids
}
