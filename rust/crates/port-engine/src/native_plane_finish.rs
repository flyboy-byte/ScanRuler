// SPDX-License-Identifier: AGPL-3.0-only
//! Plane-only finishing from common refs, orientElement/orientFit and withSide.
use super::*;
fn whitespace(v: &u16) -> bool {
    matches!(*v,0x9..=0xd|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn trim(v: &[u16]) -> &[u16] {
    let a = v.iter().position(|v| !whitespace(v)).unwrap_or(v.len());
    let b = v.iter().rposition(|v| !whitespace(v)).map_or(a, |i| i + 1);
    &v[a..b]
}
fn lower(v: &[u16]) -> Result<Text, Fault> {
    port_core::js_string::lowercase(trim(v), 1_000_000, 2_000_000).map_err(Fault::host)
}
fn error(code: CommandErrorCode, prefix: &str, value: &[u16], suffix: &str) -> Fault {
    let mut message = text(prefix);
    message.extend(value);
    message.extend(text(suffix));
    match CommandError::from_utf16(code, &message, 1_000_000) {
        Ok(e) => Fault::Command(Arc::new(e)),
        Err(e) => Fault::host(e),
    }
}
fn string(c: &C) -> Result<Text, Fault> {
    match c {
        C::String(v) => Ok(v.clone()),
        _ => Err(Fault::host("native reference name")),
    }
}
fn vector(c: &C) -> Result<[f64; 3], Fault> {
    let C::Array(a) = c else {
        return Err(Fault::host("native fit vector"));
    };
    if a.len() != 3 {
        return Err(Fault::host("native fit vector length"));
    }
    let mut out = [0.; 3];
    for (i, v) in a.iter().enumerate() {
        let C::Number(n) = v else {
            return Err(Fault::host("native fit coordinate"));
        };
        out[i] = *n
    }
    Ok(out)
}
fn xyz(v: [f64; 3]) -> C {
    arr(v.into_iter().map(C::Number).collect())
}
fn list(doc: &Session, key: &str) -> Result<Vec<C>, Fault> {
    match doc.store(StoreId::App).get(key) {
        C::Array(a) => Ok((*a).clone()),
        _ => Err(Fault::host("native reference list")),
    }
}
fn base(name: &[u16]) -> Option<(f64, [f64; 3])> {
    let mut name = trim(name)
        .iter()
        .map(|&c| if (97..=122).contains(&c) { c - 32 } else { c })
        .collect::<Vec<_>>();
    let suffix = text("PLANE");
    if name.ends_with(&suffix) {
        name.truncate(name.len() - suffix.len());
        while name
            .last()
            .is_some_and(|c| whitespace(c) || *c == 45 || *c == 95)
        {
            name.pop();
        }
    }
    if name == text("XY") || name == text("YX") {
        Some((-1., [0., 0., 1.]))
    } else if name == text("YZ") || name == text("ZY") {
        Some((-2., [1., 0., 0.]))
    } else if name == text("XZ") || name == text("ZX") {
        Some((-3., [0., 1., 0.]))
    } else {
        None
    }
}
fn reference(doc: &Session, given: &Value) -> Result<(f64, [f64; 3]), Fault> {
    if let Value::String(name) = given
        && let Some(base) = base(name)
    {
        return Ok(base);
    }
    let elements = list(doc, "elements")?;
    let sections = list(doc, "sections")?;
    let mut by_number = match given {
        Value::Number(n) => Some(*n),
        Value::String(s) => {
            let trimmed = trim(s);
            let numeric = !trimmed.is_empty() && trimmed.iter().all(|v| (48..=57).contains(v));
            let named = elements
                .iter()
                .chain(&sections)
                .any(|e| matches!(field(e,"name"),C::String(name) if trim(&name)==trimmed));
            if numeric && !named {
                Some(
                    String::from_utf16_lossy(trimmed)
                        .parse::<f64>()
                        .map_err(Fault::host)?,
                )
            } else {
                None
            }
        }
        _ => return Err(Fault::host("admitted plane reference")),
    };
    let want = if by_number.is_none() {
        let Value::String(s) = given else {
            return Err(Fault::host("native reference text"));
        };
        Some(lower(s)?)
    } else {
        None
    };
    let matches = |c: &C| -> Result<bool, Fault> {
        if let Some(id) = by_number {
            Ok(matches!(field(c,"id"),C::Number(n)if n==id))
        } else {
            Ok(lower(&string(&field(c, "name"))?)?
                == *want
                    .as_ref()
                    .ok_or_else(|| Fault::host("native reference key"))?)
        }
    };
    let mut hits = Vec::new();
    for element in &elements {
        if matches(element)? {
            hits.push(element)
        }
    }
    if hits.is_empty() {
        let mut sections_found = Vec::new();
        for section in &sections {
            if matches(section)? {
                sections_found.push(section)
            }
        }
        if !sections_found.is_empty() {
            if sections_found.len() > 1 && by_number.is_none() {
                let Value::String(s) = given else {
                    return Err(Fault::host("section reference"));
                };
                return Err(error(
                    CommandErrorCode::InvalidInput,
                    &format!("{} sections are named \"", sections_found.len()),
                    s,
                    "\" — name it by its id.",
                ));
            }
            let section = sections_found[0];
            let C::Number(id) = field(section, "id") else {
                return Err(Fault::host("section id"));
            };
            return Ok((id, vector(&field(&field(section, "frame"), "normal"))?));
        }
    }
    if hits.is_empty() {
        if let Some(n) = by_number.take() {
            return Err(command_error(
                CommandErrorCode::NotFound,
                &format!(
                    "There is no element {}.",
                    port_core::js_number::number_string(n)
                ),
            ));
        }
        let Value::String(s) = given else {
            return Err(Fault::host("missing ref text"));
        };
        return Err(error(
            CommandErrorCode::NotFound,
            "There is no element named \"",
            s,
            "\".",
        ));
    }
    if hits.len() > 1 && by_number.is_none() {
        let Value::String(s) = given else {
            return Err(Fault::host("ambiguous reference"));
        };
        return Err(error(
            CommandErrorCode::InvalidInput,
            &format!("{} elements are named \"", hits.len()),
            s,
            "\" — name it by its id.",
        ));
    }
    let element = hits[0];
    let name = string(&field(element, "name"))?;
    let fit = field(element, "fit");
    if !truthy(&fit) {
        let mut suffix = text(" has no geometry");
        if let C::String(message) = field(element, "message")
            && !message.is_empty()
        {
            suffix.extend(text(" — "));
            suffix.extend(message)
        }
        suffix.extend(text("."));
        let mut message = name.clone();
        message.extend(suffix);
        return Err(
            match CommandError::from_utf16(CommandErrorCode::InvalidState, &message, 1_000_000) {
                Ok(e) => Fault::Command(Arc::new(e)),
                Err(e) => Fault::host(e),
            },
        );
    }
    if !matches!(field(element,"kind"),C::String(s)if s==text("plane")) {
        let kind = string(&field(element, "kind"))?;
        let mut suffix = text(" is a ");
        suffix.extend(kind);
        suffix.extend(text("; this takes a plane."));
        return Err(error(
            CommandErrorCode::InvalidInput,
            "",
            &name,
            &String::from_utf16_lossy(&suffix),
        ));
    }
    let C::Number(id) = field(element, "id") else {
        return Err(Fault::host("element id"));
    };
    Ok((id, vector(&field(&fit, "normal"))?))
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = port_core::hypot3(v);
    if len < 1e-12 {
        None
    } else {
        Some(v.map(|x| x / len))
    }
}
fn cylinder(fit: &C) -> bool {
    matches!(field(fit, "kind"), C::String(k) if k == text("cylinder"))
}
fn rotate_fit(fit: &C, normal: [f64; 3], in_plane: bool) -> Result<C, Fault> {
    // orient.ts directionOf: a plane's normal, a cylinder's axis.
    let key = if cylinder(fit) { "axis" } else { "normal" };
    let dir = vector(&field(fit, key))?;
    let target = if !in_plane {
        if dot(dir, normal) < 0. {
            normal.map(|x| -x)
        } else {
            normal
        }
    } else {
        let d = -dot(dir, normal);
        let projected = [
            dir[0] + normal[0] * d,
            dir[1] + normal[1] * d,
            dir[2] + normal[2] * d,
        ];
        let Some(target) = normalize(projected) else {
            return Ok(fit.clone());
        };
        target
    };
    let axis = cross(dir, target);
    let sin = port_core::hypot3(axis);
    let cos = dot(dir, target);
    let angle = port_core::js_atan2(sin, cos);
    let mut turned = fit.clone();
    // transformFit turns a cylinder's centre and axis only, and withDirection
    // then puts both back exactly: nothing of the rotation survives.
    if angle > 1e-12 && key == "normal" {
        let rot_axis = if sin > 1e-12 {
            axis
        } else {
            cross(
                dir,
                if dir[0].abs() < 0.9 {
                    [1., 0., 0.]
                } else {
                    [0., 1., 0.]
                },
            )
        };
        let len = port_core::hypot3(rot_axis);
        let mut r = [1., 0., 0., 0., 1., 0., 0., 0., 1.];
        if len > 1e-20 && angle.abs() > 1e-20 {
            let half = angle / 2.;
            let s = half.sin() / len;
            let (w, x, y, z) = (
                half.cos(),
                rot_axis[0] * s,
                rot_axis[1] * s,
                rot_axis[2] * s,
            );
            let n = port_core::hypot4([w, x, y, z]);
            if n > 1e-20 {
                let (w, x, y, z) = (w / n, x / n, y / n, z / n);
                r = [
                    1. - 2. * (y * y + z * z),
                    2. * (x * y - w * z),
                    2. * (x * z + w * y),
                    2. * (x * y + w * z),
                    1. - 2. * (x * x + z * z),
                    2. * (y * z - w * x),
                    2. * (x * z - w * y),
                    2. * (y * z + w * x),
                    1. - 2. * (x * x + y * y),
                ];
            }
        }
        let rotate = |v: [f64; 3]| {
            [
                r[0] * v[0] + r[1] * v[1] + r[2] * v[2],
                r[3] * v[0] + r[4] * v[1] + r[5] * v[2],
                r[6] * v[0] + r[7] * v[1] + r[8] * v[2],
            ]
        };
        for key in ["normal", "basisU", "basisV"] {
            let v = rotate(vector(&field(fit, key))?);
            turned = put(&turned, key, xyz(normalize(v).unwrap_or(v)));
        }
    }
    // Source pins the direction and restores the exact measured center after rotation.
    turned = put(&turned, key, xyz(target));
    turned = put(&turned, "center", field(fit, "center"));
    Ok(turned)
}
pub(super) fn orient(doc: &Session, input: Option<&Value>, fit: &C) -> Result<(C, C), Fault> {
    let Some(input) = input else {
        return Ok((fit.clone(), C::Undefined));
    };
    let (id, normal) = reference(
        doc,
        get(input, "to").ok_or_else(|| Fault::host("missing admitted orient reference"))?,
    )?;
    let relation = match get(input, "relation") {
        Some(Value::String(s)) => s.clone(),
        _ => text("normal"),
    };
    let recipe = obj(vec![
        ("ref", C::Number(id)),
        ("relation", C::String(relation.clone())),
    ]);
    let app = doc.store(StoreId::App);
    let d = put(&app.get("draft"), "orient", recipe.clone());
    patch(doc, vec![("draft", d)])?;
    Ok((
        rotate_fit(fit, normal, relation == text("inPlane"))?,
        recipe,
    ))
}
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0. && b == 0. {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.
        } else {
            -0.
        }
    } else if a > b {
        a
    } else {
        b
    }
}
pub(super) fn extension(doc: &Session, input: Option<&Value>, fit: &C) -> Result<C, Fault> {
    let Some(input) = input else {
        return Ok(C::Undefined);
    };
    let Value::Object(fields) = input else {
        return Err(Fault::host("admitted extension object"));
    };
    if fields.is_empty() {
        return Ok(C::Undefined);
    }
    let round = cylinder(fit);
    let sides: &[&str] = if round {
        &["start", "end"]
    } else {
        &["uMin", "uMax", "vMin", "vMax"]
    };
    let shape = |values: &[f64; 4]| {
        if round {
            obj(vec![
                ("kind", st("cylinder")),
                ("start", C::Number(values[0])),
                ("end", C::Number(values[1])),
            ])
        } else {
            obj(vec![
                ("kind", st("plane")),
                ("uMin", C::Number(values[0])),
                ("uMax", C::Number(values[1])),
                ("vMin", C::Number(values[2])),
                ("vMax", C::Number(values[3])),
            ])
        }
    };
    let mut values = [0.; 4];
    let mut changed = false;
    for i in crate::schema_value::order(fields) {
        let (key, value) = &fields[i];
        if matches!(value, Value::Undefined) {
            continue;
        }
        let Some(side) = sides.iter().position(|side| *key == text(side)) else {
            return Err(error(
                CommandErrorCode::InvalidInput,
                if round {
                    "A cylinder has no side \""
                } else {
                    "A plane has no side \""
                },
                key,
                if round {
                    "\" — it has start, end."
                } else {
                    "\" — it has uMin, uMax, vMin, vMax."
                },
            ));
        };
        let Value::Number(value) = value else {
            return Err(Fault::host("admitted extension number"));
        };
        // extend.ts fittedSpan: a cylinder's length, a plane's full width.
        let span = match field(
            fit,
            if round {
                "length"
            } else if side < 2 {
                "extentU"
            } else {
                "extentV"
            },
        ) {
            C::Number(n) if round => n,
            C::Number(n) => 2. * n,
            _ => return Err(Fault::host("extendable span")),
        };
        let min = 0.001 - span - values[side ^ 1];
        values[side] = if value.is_finite() {
            max(*value, min)
        } else {
            0.
        };
        changed = true;
        let d = put(
            &doc.store(StoreId::App).get("draft"),
            "extend",
            shape(&values),
        );
        patch(doc, vec![("draft", d)])?;
    }
    if changed && values.iter().any(|n| *n != 0.) {
        Ok(shape(&values))
    } else {
        Ok(C::Undefined)
    }
}
