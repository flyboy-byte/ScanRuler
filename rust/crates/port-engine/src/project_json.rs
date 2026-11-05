// SPDX-License-Identifier: AGPL-3.0-only
//! Bounded JSON with JavaScript numbers, own-key order and lossless UTF-16 strings.
use crate::schema_value::{Value as V, order};
pub(super) fn parse(bytes: &[u8]) -> Result<V, String> {
    let input = String::from_utf8_lossy(bytes);
    let input = input.strip_prefix('\u{feff}').unwrap_or(&input);
    let mut p = Parser {
        s: input.as_bytes(),
        i: 0,
        nodes: 0,
    };
    let v = p.value(0)?;
    p.space();
    if p.i != p.s.len() {
        return Err("Malformed project: project.json is not JSON.".into());
    }
    Ok(v)
}
struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    nodes: usize,
}
impl Parser<'_> {
    fn bad<T>(&self) -> Result<T, String> {
        Err("Malformed project: project.json is not JSON.".into())
    }
    fn space(&mut self) {
        while self
            .s
            .get(self.i)
            .is_some_and(|b| matches!(b, b' ' | b'\r' | b'\n' | b'\t'))
        {
            self.i += 1;
        }
    }
    fn take(&mut self, b: u8) -> bool {
        self.space();
        if self.s.get(self.i) == Some(&b) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Result<Vec<u16>, String> {
        if !self.take(b'"') {
            return self.bad();
        }
        let mut out = vec![];
        loop {
            let start = self.i;
            while self
                .s
                .get(self.i)
                .is_some_and(|b| *b >= 32 && *b != b'"' && *b != b'\\')
            {
                self.i += 1;
            }
            let part = std::str::from_utf8(&self.s[start..self.i])
                .map_err(|_| "Malformed project: project.json is not JSON.".to_string())?;
            out.extend(part.encode_utf16());
            match self.s.get(self.i).copied() {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = match self.s.get(self.i).copied() {
                        Some(b'"') => 34,
                        Some(b'\\') => 92,
                        Some(b'/') => 47,
                        Some(b'b') => 8,
                        Some(b'f') => 12,
                        Some(b'n') => 10,
                        Some(b'r') => 13,
                        Some(b't') => 9,
                        Some(b'u') => {
                            let mut n = 0u16;
                            for _ in 0..4 {
                                self.i += 1;
                                let Some(b) = self.s.get(self.i) else {
                                    return self.bad();
                                };
                                let Some(d) = char::from(*b).to_digit(16) else {
                                    return self.bad();
                                };
                                n = n * 16 + d as u16;
                            }
                            n
                        }
                        _ => return self.bad(),
                    };
                    out.push(c);
                    self.i += 1;
                }
                _ => return self.bad(),
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<V, String> {
        self.nodes += 1;
        if depth > 128 {
            return Err("Malformed project: nesting depth.".into());
        }
        if self.nodes > 2_000_000 {
            return Err("Native project JSON node budget exceeded.".into());
        }
        self.space();
        match self.s.get(self.i).copied() {
            Some(b'"') => Ok(V::String(self.string()?)),
            Some(b'{') => {
                self.i += 1;
                let mut fields: Vec<(Vec<u16>, V)> = vec![];
                let mut positions = std::collections::HashMap::<Vec<u16>, usize>::new();
                if self.take(b'}') {
                    return Ok(V::Object(fields));
                }
                loop {
                    let k = self.string()?;
                    if !self.take(b':') {
                        return self.bad();
                    }
                    let v = self.value(depth + 1)?;
                    if let Some(i) = positions.get(&k) {
                        fields[*i].1 = v;
                    } else {
                        positions.insert(k.clone(), fields.len());
                        fields.push((k, v));
                    }
                    if self.take(b'}') {
                        break;
                    }
                    if !self.take(b',') {
                        return self.bad();
                    }
                }
                Ok(V::Object(fields))
            }
            Some(b'[') => {
                self.i += 1;
                let mut a = vec![];
                if self.take(b']') {
                    return Ok(V::Array(a));
                }
                loop {
                    a.push(self.value(depth + 1)?);
                    if self.take(b']') {
                        break;
                    }
                    if !self.take(b',') {
                        return self.bad();
                    }
                }
                Ok(V::Array(a))
            }
            Some(b't' | b'f' | b'n') => {
                for (s, v) in [
                    ("true", V::Boolean(true)),
                    ("false", V::Boolean(false)),
                    ("null", V::Null),
                ] {
                    if self.s[self.i..].starts_with(s.as_bytes()) {
                        self.i += s.len();
                        return Ok(v);
                    }
                }
                self.bad()
            }
            Some(b'-' | b'0'..=b'9') => {
                let start = self.i;
                if self.s[self.i] == b'-' {
                    self.i += 1
                }
                match self.s.get(self.i) {
                    Some(b'0') => self.i += 1,
                    Some(b'1'..=b'9') => {
                        self.i += 1;
                        while self.s.get(self.i).is_some_and(u8::is_ascii_digit) {
                            self.i += 1
                        }
                    }
                    _ => return self.bad(),
                }
                if self.s.get(self.i) == Some(&b'.') {
                    self.i += 1;
                    let n = self.i;
                    while self.s.get(self.i).is_some_and(u8::is_ascii_digit) {
                        self.i += 1
                    }
                    if self.i == n {
                        return self.bad();
                    }
                }
                if self.s.get(self.i).is_some_and(|b| matches!(b, b'e' | b'E')) {
                    self.i += 1;
                    if self.s.get(self.i).is_some_and(|b| matches!(b, b'+' | b'-')) {
                        self.i += 1
                    }
                    let n = self.i;
                    while self.s.get(self.i).is_some_and(u8::is_ascii_digit) {
                        self.i += 1
                    }
                    if self.i == n {
                        return self.bad();
                    }
                }
                let s = std::str::from_utf8(&self.s[start..self.i])
                    .map_err(|_| "Malformed project: project.json is not JSON.".to_string())?;
                Ok(V::Number(s.parse().map_err(|_| {
                    "Malformed project: project.json is not JSON.".to_string()
                })?))
            }
            _ => self.bad(),
        }
    }
}
fn quote(s: &[u16], out: &mut String) {
    out.push('"');
    for c in char::decode_utf16(s.iter().copied()) {
        match c {
            Ok('"') => out.push_str("\\\""),
            Ok('\\') => out.push_str("\\\\"),
            Ok('\n') => out.push_str("\\n"),
            Ok('\r') => out.push_str("\\r"),
            Ok('\t') => out.push_str("\\t"),
            Ok('\u{8}') => out.push_str("\\b"),
            Ok('\u{c}') => out.push_str("\\f"),
            Ok(c) if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            Ok(c) => out.push(c),
            Err(e) => out.push_str(&format!("\\u{:04x}", e.unpaired_surrogate())),
        }
    }
    out.push('"')
}
pub(super) fn encode(v: &V, pretty: bool) -> String {
    fn write(v: &V, out: &mut String, pretty: bool, depth: usize) {
        let pad = |out: &mut String, d: usize| {
            if pretty {
                out.push('\n');
                out.push_str(&"  ".repeat(d));
            }
        };
        match v {
            V::Undefined | V::Null => out.push_str("null"),
            V::Boolean(b) => out.push_str(if *b { "true" } else { "false" }),
            V::Number(n) => out.push_str(&if n.is_finite() {
                port_core::js_number::number_string(*n)
            } else {
                "null".into()
            }),
            V::String(s) => quote(s, out),
            V::Array(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',')
                    }
                    pad(out, depth + 1);
                    write(v, out, pretty, depth + 1)
                }
                if !a.is_empty() {
                    pad(out, depth)
                }
                out.push(']')
            }
            V::Object(f) => {
                out.push('{');
                let mut n = 0;
                for i in order(f) {
                    let (k, v) = &f[i];
                    if matches!(v, V::Undefined) {
                        continue;
                    }
                    if n > 0 {
                        out.push(',')
                    }
                    pad(out, depth + 1);
                    quote(k, out);
                    out.push(':');
                    if pretty {
                        out.push(' ')
                    }
                    write(v, out, pretty, depth + 1);
                    n += 1;
                }
                if n > 0 {
                    pad(out, depth)
                }
                out.push('}')
            }
            V::Bytes(b) => write(
                &V::Object(
                    b.iter()
                        .enumerate()
                        .map(|(i, b)| {
                            (
                                i.to_string().encode_utf16().collect(),
                                V::Number(f64::from(*b)),
                            )
                        })
                        .collect(),
                ),
                out,
                pretty,
                depth,
            ),
        }
    }
    let mut out = String::new();
    write(v, &mut out, pretty, 0);
    out
}
