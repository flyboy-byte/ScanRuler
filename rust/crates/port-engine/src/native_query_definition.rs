// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen scan.query metadata.
use super::super::{object, value_text};
use crate::schema_value::Value;
pub(super) fn definition() -> Value {
    object(vec![
        ("name", value_text("query")),
        ("title", value_text("Read the scan’s points")),
        (
            "description",
            value_text(
                "The scan’s vertices inside a box, or within a radius of a point (near), with their outward normals — the points on a face, to measure a dimension or a draft off; normal keeps only those facing a direction. Returns matched, the vertex numbers, points and normals (mm, current frame), the box round them, centroid and mean normal; fields: count, ids or all (the default) says how much. Trap: at most limit come back (1000 by default, 20000 at most), every k-th spread evenly, step saying k — mark: true marks every matching vertex, not the thinned list, for element.fit_marked { marked: true }. Changes nothing on the part.",
            ),
        ),
        (
            "input",
            object(vec![
                ("type", value_text("object")),
                (
                    "properties",
                    object(vec![
                        (
                            "box",
                            object(vec![
                                ("type", value_text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "min",
                                            object(vec![
                                                ("type", value_text("array")),
                                                (
                                                    "items",
                                                    object(vec![("type", value_text("number"))]),
                                                ),
                                                ("description", value_text("Lowest corner, mm.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                        (
                                            "max",
                                            object(vec![
                                                ("type", value_text("array")),
                                                (
                                                    "items",
                                                    object(vec![("type", value_text("number"))]),
                                                ),
                                                ("description", value_text("Highest corner, mm.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                    ]),
                                ),
                                (
                                    "required",
                                    Value::Array(vec![value_text("min"), value_text("max")]),
                                ),
                                ("additionalProperties", Value::Boolean(false)),
                                ("description", value_text("A box in the current frame.")),
                            ]),
                        ),
                        (
                            "near",
                            object(vec![
                                ("type", value_text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "at",
                                            object(vec![
                                                ("type", value_text("array")),
                                                (
                                                    "items",
                                                    object(vec![("type", value_text("number"))]),
                                                ),
                                                ("description", value_text("The centre, mm.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                        (
                                            "radius",
                                            object(vec![
                                                ("type", value_text("number")),
                                                ("description", value_text("mm.")),
                                                ("exclusiveMinimum", Value::Number(0.0)),
                                            ]),
                                        ),
                                    ]),
                                ),
                                (
                                    "required",
                                    Value::Array(vec![value_text("at"), value_text("radius")]),
                                ),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    value_text("A ball about a point, in place of box."),
                                ),
                            ]),
                        ),
                        (
                            "normal",
                            object(vec![
                                ("type", value_text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "dir",
                                            object(vec![
                                                ("type", value_text("array")),
                                                (
                                                    "items",
                                                    object(vec![("type", value_text("number"))]),
                                                ),
                                                (
                                                    "description",
                                                    value_text("A direction, any length."),
                                                ),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                        (
                                            "withinDeg",
                                            object(vec![
                                                ("type", value_text("number")),
                                                (
                                                    "description",
                                                    value_text("Degrees; 30 by default."),
                                                ),
                                                ("exclusiveMinimum", Value::Number(0.0)),
                                                ("maximum", Value::Number(180.0)),
                                            ]),
                                        ),
                                    ]),
                                ),
                                ("required", Value::Array(vec![value_text("dir")])),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    value_text(
                                        "Only the vertices whose outward normal faces this way.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "limit",
                            object(vec![
                                ("type", value_text("integer")),
                                (
                                    "description",
                                    value_text(
                                        "How many vertices at most; 1000 by default, 20000 at most.",
                                    ),
                                ),
                                ("minimum", Value::Number(1.0)),
                                ("maximum", Value::Number(20000.0)),
                            ]),
                        ),
                        (
                            "mark",
                            object(vec![
                                ("type", value_text("boolean")),
                                (
                                    "description",
                                    value_text(
                                        "Leave every matching vertex marked, for element.fit_marked { marked: true }.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "fields",
                            object(vec![
                                ("type", value_text("string")),
                                (
                                    "enum",
                                    Value::Array(vec![
                                        value_text("count"),
                                        value_text("ids"),
                                        value_text("all"),
                                    ]),
                                ),
                                (
                                    "description",
                                    value_text(
                                        "count: the numbers alone (the default with mark); ids: with the vertices; all: with points and normals (the default).",
                                    ),
                                ),
                            ]),
                        ),
                    ]),
                ),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
        ("readOnly", Value::Boolean(true)),
    ])
}
