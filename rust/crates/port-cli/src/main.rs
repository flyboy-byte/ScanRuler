// SPDX-License-Identifier: AGPL-3.0-only
//! Thin native filesystem/presentation adapter over registered engine commands.
use port_engine::{
    guarded::Fault,
    native_engine::{Engine, MAX_INPUT_BYTES},
    schema_value::Value,
};
use std::{io::Read, path::Path, process::ExitCode};
fn text(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn quote(v: &[u16]) -> String {
    let mut out = String::from("\"");
    for c in char::decode_utf16(v.iter().copied()) {
        match c {
            Ok('"') => out.push_str("\\\""),
            Ok('\\') => out.push_str("\\\\"),
            Ok('\n') => out.push_str("\\n"),
            Ok('\r') => out.push_str("\\r"),
            Ok('\t') => out.push_str("\\t"),
            Ok(c) if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            Ok(c) => out.push(c),
            Err(e) => out.push_str(&format!("\\u{:04x}", e.unpaired_surrogate())),
        }
    }
    out.push('"');
    out
}
fn json(v: &Value) -> String {
    match v {
        Value::Undefined | Value::Null => "null".into(),
        Value::Boolean(v) => v.to_string(),
        Value::Number(v) => {
            if v.is_finite() {
                port_core::js_number::number_string(*v)
            } else {
                "null".into()
            }
        }
        Value::String(v) => quote(v),
        Value::Array(v) => format!("[{}]", v.iter().map(json).collect::<Vec<_>>().join(",")),
        Value::Object(v) => format!(
            "{{{}}}",
            v.iter()
                .filter(|(_, v)| !matches!(v, Value::Undefined))
                .map(|(k, v)| format!("{}:{}", quote(k), json(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Bytes(v) => format!(
            "[{}]",
            v.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
        ),
    }
}
fn fault(f: Fault) -> String {
    match f {
        Fault::Command(e) => format!(
            "{}: {}",
            e.code().as_str(),
            String::from_utf16_lossy(e.message_units())
        ),
        Fault::Error { message, .. } | Fault::Text(message) => String::from_utf16_lossy(&message),
        Fault::Host(e) => format!("native host limit/error: {e:?}"),
        Fault::Other(_) => "native command failed".into(),
    }
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        return Err("Usage: scanruler-rs info <scan.stl|scan.ply> [--units mm|cm|m|in]".into());
    };
    if command == "--help" || command == "-h" {
        println!("Usage: scanruler-rs info <scan.stl|scan.ply> [--units mm|cm|m|in]");
        return Ok(());
    }
    if command != "info" {
        return Err(format!("Unknown command {command:?}. Use --help."));
    }
    let path = args.next().ok_or("info requires a scan path")?;
    let mut units = "mm".to_string();
    while let Some(arg) = args.next() {
        if arg == "--units" {
            units = args.next().ok_or("--units requires mm, cm, m or in")?
        } else {
            return Err(format!("Unknown argument {arg:?}"));
        }
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&path)
        .map_err(|e| format!("{path}: {e}"))?
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err("Scan exceeds the native input byte budget.".into());
    }
    let name = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Scan filename must be UTF-8")?;
    let engine = Engine::new().map_err(fault)?;
    let result = engine
        .execute(
            "scan.open",
            Value::Object(vec![
                (text("bytes"), Value::Bytes(bytes)),
                (text("name"), Value::String(text(name))),
                (text("units"), Value::String(text(&units))),
            ]),
        )
        .map_err(fault)?;
    let Value::Object(fields) = result else {
        return Err("Engine returned an invalid scan result".into());
    };
    let scan = fields
        .into_iter()
        .find(|(k, _)| *k == text("scan"))
        .map(|(_, v)| v)
        .ok_or("Engine returned no scan summary")?;
    let Value::Object(mut summary) = scan else {
        return Err("Engine returned invalid scan metadata".into());
    };
    summary.push((
        text("components"),
        Value::Number(engine.components() as f64),
    ));
    println!("{}", json(&Value::Object(summary)));
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}
