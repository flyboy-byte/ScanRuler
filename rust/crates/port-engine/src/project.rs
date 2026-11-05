// SPDX-License-Identifier: AGPL-3.0-only
//! Schema-1 project archives. ZIP storage is interoperable; compression is not geometry.
use crate::schema_value::Value;
use port_io::inflate::{self, ZipLimits};
use std::cell::RefCell;
#[path = "project_json.rs"]
mod json;
#[path = "project_validation.rs"]
mod validation;
/// Original manifest size limit.
pub const MAX_MANIFEST: usize = 64 * 1024 * 1024;
/// Native aggregate allocation policy, separate from the original's 1 GiB limit.
pub const MAX_NATIVE_PROJECT: usize = 128 * 1024 * 1024;
const SIZE: &str = "The project exceeds the supported archive size limits.";
const INVALID: &str = "Malformed project: duplicate or invalid archive member.";
/// A validated manifest and original member bytes, in archive order.
#[derive(Clone)]
pub struct Project {
    /// Schema-1 manifest.
    pub manifest: Value,
    /// Non-manifest members, including unknown plugin bytes.
    pub members: Vec<(String, Vec<u8>)>,
}
/// Parse project JSON, retaining UTF-16 strings and JavaScript own-key order.
pub fn parse_json(bytes: &[u8]) -> Result<Value, String> {
    json::parse(bytes)
}
/// JavaScript-compatible JSON serialization for the project manifest.
pub fn manifest_text(v: &Value) -> String {
    let compact = json::encode(v, false);
    if compact.encode_utf16().count() > 2_000_000 {
        compact
    } else {
        json::encode(v, true)
    }
}
/// Validate the frozen schema-1 boundary, including unknown nested plugin data.
pub fn validate(v: &Value) -> Result<(), String> {
    validation::validate(v)
}
fn u16at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?))
}
fn u32at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}
// Read only declared capacities here. The existing original-compatible ZIP reader
// performs decoding; its callback checks these capacities before any inflate.
fn sizes(bytes: &[u8]) -> Result<Vec<usize>, String> {
    let e = (0..=bytes.len().saturating_sub(22))
        .rev()
        .take(65537)
        .find(|i| u32at(bytes, *i) == Some(0x06054b50))
        .ok_or("Not a ScanRuler project file.")?;
    let n = u16at(bytes, e + 8).ok_or("Not a ScanRuler project file.")?;
    if u32at(bytes, e.saturating_sub(20)) == Some(0x07064b50) {
        return Err("Native ZIP64 project loading is not implemented.".into());
    }
    let mut o = u32at(bytes, e + 16).ok_or("Not a ScanRuler project file.")? as usize;
    let mut out = vec![];
    for _ in 0..n {
        out.push(u32at(bytes, o + 24).ok_or("Not a ScanRuler project file.")? as usize);
        o = o
            .checked_add(
                46 + u16at(bytes, o + 28).ok_or("Not a ScanRuler project file.")? as usize
                    + u16at(bytes, o + 30).ok_or("Not a ScanRuler project file.")? as usize
                    + u16at(bytes, o + 32).ok_or("Not a ScanRuler project file.")? as usize,
            )
            .ok_or("Not a ScanRuler project file.")?;
    }
    Ok(out)
}
/// Read a bounded original ZIP, rejecting duplicate/path members before decompression.
pub fn unpack(bytes: &[u8]) -> Result<Project, String> {
    if bytes.len() > MAX_NATIVE_PROJECT {
        return Err("Native project byte budget exceeded.".into());
    }
    let sizes = sizes(bytes)?;
    let names = RefCell::new(Vec::<String>::new());
    let error = RefCell::new(None);
    let total = std::cell::Cell::new(0usize);
    let files = inflate::unzip_filtered_bounded(
        bytes,
        |name| {
            if error.borrow().is_some() {
                return false;
            }
            let mut names = names.borrow_mut();
            if names.iter().any(|n| n == name) || name.contains(['/', '\\']) || name == "__proto__"
            {
                *error.borrow_mut() = Some(INVALID.to_string());
                return false;
            }
            let size = sizes.get(names.len()).copied().unwrap_or(0);
            names.push(name.into());
            let sum = total.get().saturating_add(size);
            total.set(sum);
            if names.len() > 16
                || sum > 1024 * 1024 * 1024
                || (name == "project.json" && size > MAX_MANIFEST)
            {
                *error.borrow_mut() = Some(SIZE.to_string());
                return false;
            }
            true
        },
        ZipLimits {
            max_entry_bytes: MAX_NATIVE_PROJECT,
            max_entries: 65535,
        },
        MAX_NATIVE_PROJECT,
    );
    if let Some(e) = error.into_inner() {
        return Err(e);
    }
    let mut files = files.map_err(|e| match e {
        inflate::InflateError::ResourceLimit | inflate::InflateError::AggregateLimit => {
            "Native project decompression budget exceeded.".to_string()
        }
        _ => "Not a ScanRuler project file.".to_string(),
    })?;
    let i = files
        .iter()
        .position(|(n, _)| n == "project.json")
        .ok_or("Not a ScanRuler project file.")?;
    let (_, raw) = files.remove(i);
    let manifest = json::parse(&raw)?;
    validate(&manifest)?;
    let mut entries = vec![
        manifest.get("scan"),
        manifest.get("deviation").get("reference"),
        manifest.get("flat").get("image"),
    ];
    if let Some(a) = manifest.get("scan").get("companions").list() {
        entries.extend(a)
    }
    for entry in entries {
        if let Some(name) = entry.get("member").text() {
            let name = String::from_utf16_lossy(name);
            if !files.iter().any(|(n, _)| *n == name) {
                return Err(format!("Project is missing {name}."));
            }
        }
    }
    Ok(Project {
        manifest,
        members: files,
    })
}
fn crc(b: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, v) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 }
        }
        *v = c;
    }
    let mut c = !0u32;
    for b in b {
        c = (c >> 8) ^ table[((c ^ u32::from(*b)) & 255) as usize];
    }
    !c
}

fn put16(b: &mut Vec<u8>, v: usize) {
    b.extend((v as u16).to_le_bytes())
}
fn put32(b: &mut Vec<u8>, v: u32) {
    b.extend(v.to_le_bytes())
}
/// Write a standard ZIP with stored members. Original readers accept both stored
/// and deflated members; neither compression bytes nor ZIP timestamps are recipes.
pub fn pack(p: &Project) -> Result<Vec<u8>, String> {
    let raw = manifest_text(&p.manifest).into_bytes();
    let total = p
        .members
        .iter()
        .fold(raw.len(), |n, (_, b)| n.saturating_add(b.len()));
    if p.members.len() + 1 > 16 || raw.len() > MAX_MANIFEST || total > 1024 * 1024 * 1024 {
        return Err(SIZE.into());
    }
    if total > MAX_NATIVE_PROJECT {
        return Err("Native project byte budget exceeded.".into());
    }
    let mut files = vec![("project.json".to_string(), raw)];
    for (n, b) in &p.members {
        if let Some((_, old)) = files.iter_mut().find(|(k, _)| k == n) {
            *old = b.clone()
        } else {
            files.push((n.clone(), b.clone()))
        }
    }
    let mut out = vec![];
    let mut central = vec![];
    for (n, b) in &files {
        if n.len() > 65535 {
            return Err(SIZE.into());
        }
        let off = out.len() as u32;
        let crc = crc(b);
        put32(&mut out, 0x04034b50);
        put16(&mut out, 20);
        put16(&mut out, 2048);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put16(&mut out, 33);
        put32(&mut out, crc);
        put32(&mut out, b.len() as u32);
        put32(&mut out, b.len() as u32);
        put16(&mut out, n.len());
        put16(&mut out, 0);
        out.extend(n.as_bytes());
        out.extend(b);
        put32(&mut central, 0x02014b50);
        for x in [20, 20, 2048, 0, 0, 33] {
            put16(&mut central, x)
        }
        put32(&mut central, crc);
        put32(&mut central, b.len() as u32);
        put32(&mut central, b.len() as u32);
        for x in [n.len(), 0, 0, 0, 0] {
            put16(&mut central, x)
        }
        put32(&mut central, 0);
        put32(&mut central, off);
        central.extend(n.as_bytes());
    }
    let off = out.len() as u32;
    let len = central.len() as u32;
    out.extend(central);
    put32(&mut out, 0x06054b50);
    put16(&mut out, 0);
    put16(&mut out, 0);
    put16(&mut out, files.len());
    put16(&mut out, files.len());
    put32(&mut out, len);
    put32(&mut out, off);
    put16(&mut out, 0);
    Ok(out)
}
