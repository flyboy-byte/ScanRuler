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
const USAGE: &str =
    "Usage: scanruler-rs <info|fit-plane> <scan.stl|scan.ply|scan.obj> [--units mm|cm|m|in]
Plane options: [--sigma 0|1|2|3] [--box xmin ymin zmin xmax ymax zmax] [--marked]
[--normal x y z] [--within-deg degrees] [--orient XY|YZ|XZ] [--in-plane]
[--extend uMin uMax vMin vMax] [--undo] [--redo]
Box coordinates and extensions are in millimetres. --marked uses scan.query. --redo first undoes.";
fn numbers<const N: usize>(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<[f64; N], String> {
    let mut out = [0.; N];
    for v in &mut out {
        *v = args
            .next()
            .ok_or_else(|| format!("{flag} requires {N} numbers"))?
            .parse()
            .map_err(|_| format!("{flag} requires numbers"))?;
    }
    Ok(out)
}
fn vector(v: [f64; 3]) -> Value {
    Value::Array(v.into_iter().map(Value::Number).collect())
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(USAGE.into());
    };
    if command == "--help" || command == "-h" {
        println!("{USAGE}");
        return Ok(());
    }
    if command != "info" && command != "fit-plane" {
        return Err(format!("Unknown command {command:?}. Use --help."));
    }
    let path = args.next().ok_or("Command requires a scan path")?;
    let mut units = "mm".to_string();
    let mut sigma = None;
    let mut undo = false;
    let mut redo = false;
    let mut bounds = None;
    let mut marked = false;
    let mut normal = None;
    let mut within = None;
    let mut orient = None;
    let mut in_plane = false;
    let mut extend = None;
    while let Some(arg) = args.next() {
        if arg == "--units" {
            units = args.next().ok_or("--units requires mm, cm, m or in")?
        } else if command == "fit-plane" && arg == "--sigma" {
            sigma = Some(
                args.next()
                    .ok_or("--sigma requires 0, 1, 2 or 3")?
                    .parse::<f64>()
                    .map_err(|_| "--sigma requires a number")?,
            );
        } else if command == "fit-plane" && arg == "--box" {
            let v = numbers::<6>(&mut args, "--box")?;
            bounds = Some(Value::Object(vec![
                (text("min"), vector([v[0], v[1], v[2]])),
                (text("max"), vector([v[3], v[4], v[5]])),
            ]));
        } else if command == "fit-plane" && arg == "--marked" {
            marked = true;
        } else if command == "fit-plane" && arg == "--normal" {
            normal = Some(numbers::<3>(&mut args, "--normal")?);
        } else if command == "fit-plane" && arg == "--within-deg" {
            within = Some(numbers::<1>(&mut args, "--within-deg")?[0]);
        } else if command == "fit-plane" && arg == "--orient" {
            orient = Some(args.next().ok_or("--orient requires a plane reference")?);
        } else if command == "fit-plane" && arg == "--in-plane" {
            in_plane = true;
        } else if command == "fit-plane" && arg == "--extend" {
            extend = Some(numbers::<4>(&mut args, "--extend")?);
        } else if command == "fit-plane" && arg == "--undo" {
            undo = true;
        } else if command == "fit-plane" && arg == "--redo" {
            redo = true;
            undo = true;
        } else {
            return Err(format!("Unknown argument {arg:?}"));
        }
    }
    if in_plane && orient.is_none() {
        return Err("--in-plane requires --orient".into());
    }
    if within.is_some() && normal.is_none() {
        return Err("--within-deg requires --normal".into());
    }
    if normal.is_some() && bounds.is_none() {
        return Err("--normal requires --box".into());
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
    if command == "fit-plane" {
        let vertices = summary
            .iter()
            .find(|(k, _)| *k == text("vertices"))
            .and_then(|(_, v)| {
                if let Value::Number(n) = v {
                    Some(*n as usize)
                } else {
                    None
                }
            })
            .ok_or("Engine returned no vertex count")?;
        let mut input = vec![(text("kind"), Value::String(text("plane")))];
        let normal = normal.map(|v| {
            let mut fields = vec![(text("dir"), vector(v))];
            if let Some(within) = within {
                fields.push((text("withinDeg"), Value::Number(within)));
            }
            Value::Object(fields)
        });
        let mut selection = None;
        if marked {
            let bounds = bounds
                .or_else(|| {
                    summary
                        .iter()
                        .find(|(k, _)| *k == text("bounds"))
                        .map(|(_, v)| v.clone())
                })
                .ok_or("Engine returned no scan bounds")?;
            let mut query = vec![
                (text("box"), bounds),
                (text("mark"), Value::Boolean(true)),
                (text("fields"), Value::String(text("count"))),
            ];
            if let Some(normal) = normal {
                query.push((text("normal"), normal));
            }
            selection = Some(
                engine
                    .execute("scan.query", Value::Object(query))
                    .map_err(fault)?,
            );
            input.push((text("marked"), Value::Boolean(true)));
        } else if let Some(bounds) = bounds {
            input.push((text("box"), bounds));
            if let Some(normal) = normal {
                input.push((text("normal"), normal));
            }
        } else {
            if vertices > 1_000_000 {
                return Err("Scan exceeds the native marked-plane input budget.".into());
            }
            input.push((
                text("vertices"),
                Value::Array((0..vertices).map(|v| Value::Number(v as f64)).collect()),
            ));
        }
        if let Some(orient) = orient {
            let mut fields = vec![(text("to"), Value::String(text(&orient)))];
            if in_plane {
                fields.push((text("relation"), Value::String(text("inPlane"))));
            }
            input.push((text("orient"), Value::Object(fields)));
        }
        if let Some(extend) = extend {
            input.push((
                text("extend"),
                Value::Object(
                    ["uMin", "uMax", "vMin", "vMax"]
                        .into_iter()
                        .zip(extend)
                        .map(|(k, v)| (text(k), Value::Number(v)))
                        .collect(),
                ),
            ));
        }
        if let Some(sigma) = sigma {
            input.push((text("sigma"), Value::Number(sigma)))
        }
        let plane = engine
            .execute("element.fit_marked", Value::Object(input))
            .map_err(fault)?;
        let mut output = vec![(text("scan"), Value::Object(summary)), (text("fit"), plane)];
        if let Some(selection) = selection {
            output.push((text("selection"), selection));
        }
        if undo {
            output.push((
                text("undo"),
                engine
                    .execute("history.undo", Value::Object(vec![]))
                    .map_err(fault)?,
            ))
        }
        if redo {
            output.push((
                text("redo"),
                engine
                    .execute("history.redo", Value::Object(vec![]))
                    .map_err(fault)?,
            ))
        }
        println!("{}", json(&Value::Object(output)));
    } else {
        println!("{}", json(&Value::Object(summary)));
    }
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
