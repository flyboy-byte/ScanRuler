// SPDX-License-Identifier: AGPL-3.0-only
//! Original scan.open public metadata; history:false comes from its source definition.
use crate::schema_value::Value;
fn text(s: &str) -> Value {
    Value::String(s.encode_utf16().collect())
}
fn object(v: Vec<(&str, Value)>) -> Value {
    Value::Object(
        v.into_iter()
            .map(|(k, v)| (k.encode_utf16().collect(), v))
            .collect(),
    )
}
pub(super) fn definition() -> Value {
    object(vec![
        ("name", text("open")),
        ("title", text("Open a scan")),
        (
            "description",
            text(
                "Open a mesh as the scan to measure — STL, PLY, OBJ or 3MF, or a STEP part cut into triangles — in place of the one there is; everything measured on the old one goes with it, so it refuses while the session holds measurements unless discard is true (the refusal names what is held). A reference part opened from a file stays, its alignment and maps gone (referenceKept); one made from the session’s own work goes (referenceDropped). Returns the scan: name, units, vertex and triangle counts, bounding box, centre and diagonal (size), mm. Not undoable: a new scan starts a new history.",
            ),
        ),
        (
            "input",
            object(vec![
                ("type", text("object")),
                (
                    "properties",
                    object(vec![
                        (
                            "bytes",
                            object(vec![
                                ("type", text("string")),
                                ("format", text("binary")),
                                ("description", text("The file’s contents.")),
                            ]),
                        ),
                        (
                            "name",
                            object(vec![
                                ("type", text("string")),
                                (
                                    "description",
                                    text("The file's name with its extension — \"ballbar.stl\"."),
                                ),
                                ("minLength", Value::Number(1.0)),
                            ]),
                        ),
                        (
                            "units",
                            object(vec![
                                ("type", text("string")),
                                (
                                    "enum",
                                    Value::Array(vec![
                                        text("mm"),
                                        text("cm"),
                                        text("m"),
                                        text("in"),
                                    ]),
                                ),
                                (
                                    "description",
                                    text(
                                        "The file’s units: mm (the default), cm, m or in. Everything is measured in mm once read.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "discard",
                            object(vec![
                                ("type", text("boolean")),
                                (
                                    "description",
                                    text(
                                        "true: replace the session though it holds measurements (lost unless saved as a project first). Refused otherwise when there is work to lose.",
                                    ),
                                ),
                            ]),
                        ),
                    ]),
                ),
                ("required", Value::Array(vec![text("bytes"), text("name")])),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
        ("readOnly", Value::Boolean(false)),
        ("returnsFile", Value::Boolean(false)),
        ("returnsImage", Value::Boolean(false)),
        ("history", Value::Boolean(false)),
    ])
}
