// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen element.fit_marked public metadata.
use super::super::{object, value_text as text};
use crate::schema_value::Value;
pub(super) fn definition() -> Value {
    object(vec![
        ("name", text("fit_marked")),
        ("title", text("Fit an element to a marked surface")),
        (
            "description",
            text(
                "Fit a plane, sphere, cylinder, cone or torus to exactly the scan vertices given, with no search for the surface — vertices, or marked: true for the points the last element.fit { mark: true }, scan.segment { mark } or scan.query { mark: true } left marked, or box (with normal: only those facing a way) for every vertex inside it. The element keeps the marking as its recipe. Returns the element, and with box how many points it took. One undo step.",
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
                            "kind",
                            object(vec![
                                ("type", text("string")),
                                (
                                    "enum",
                                    Value::Array(vec![
                                        text("plane"),
                                        text("sphere"),
                                        text("cylinder"),
                                        text("cone"),
                                        text("torus"),
                                    ]),
                                ),
                                ("description", text("What to fit.")),
                            ]),
                        ),
                        (
                            "vertices",
                            object(vec![
                                ("type", text("array")),
                                (
                                    "items",
                                    object(vec![
                                        ("type", text("integer")),
                                        ("minimum", Value::Number(0.0)),
                                    ]),
                                ),
                                ("description", text("The scan vertices of the surface.")),
                                ("minItems", Value::Number(3.0)),
                            ]),
                        ),
                        (
                            "marked",
                            object(vec![
                                ("type", text("boolean")),
                                (
                                    "description",
                                    text("The points left marked, in place of vertices."),
                                ),
                            ]),
                        ),
                        (
                            "box",
                            object(vec![
                                ("type", text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "min",
                                            object(vec![
                                                ("type", text("array")),
                                                ("items", object(vec![("type", text("number"))])),
                                                ("description", text("Lowest corner, mm.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                        (
                                            "max",
                                            object(vec![
                                                ("type", text("array")),
                                                ("items", object(vec![("type", text("number"))])),
                                                ("description", text("Highest corner, mm.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                    ]),
                                ),
                                ("required", Value::Array(vec![text("min"), text("max")])),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    text("The scan inside a box, in place of vertices."),
                                ),
                            ]),
                        ),
                        (
                            "normal",
                            object(vec![
                                ("type", text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "dir",
                                            object(vec![
                                                ("type", text("array")),
                                                ("items", object(vec![("type", text("number"))])),
                                                ("description", text("A direction, any length.")),
                                                ("minItems", Value::Number(3.0)),
                                                ("maxItems", Value::Number(3.0)),
                                            ]),
                                        ),
                                        (
                                            "withinDeg",
                                            object(vec![
                                                ("type", text("number")),
                                                ("description", text("Degrees; 30 by default.")),
                                                ("exclusiveMinimum", Value::Number(0.0)),
                                                ("maximum", Value::Number(180.0)),
                                            ]),
                                        ),
                                    ]),
                                ),
                                ("required", Value::Array(vec![text("dir")])),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    text("With box: only the vertices facing this way."),
                                ),
                            ]),
                        ),
                        (
                            "sigma",
                            object(vec![
                                ("type", text("integer")),
                                (
                                    "enum",
                                    Value::Array(vec![
                                        Value::Number(3.0),
                                        Value::Number(2.0),
                                        Value::Number(1.0),
                                        Value::Number(0.0),
                                    ]),
                                ),
                                (
                                    "description",
                                    text(
                                        "Outlier cut-off, sigma: 3 (the default), 2, 1, or 0 for all points.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "knownDiameter",
                            object(vec![
                                ("type", text("number")),
                                (
                                    "description",
                                    text(
                                        "Sphere: a known diameter held, mm (a gauge ball); the centre alone is fitted.",
                                    ),
                                ),
                                ("exclusiveMinimum", Value::Number(0.0)),
                            ]),
                        ),
                        (
                            "assumed",
                            object(vec![
                                ("type", text("number")),
                                (
                                    "description",
                                    text(
                                        "The designed diameter, mm: drawn and exported at it, reported as measured.",
                                    ),
                                ),
                                ("exclusiveMinimum", Value::Number(0.0)),
                            ]),
                        ),
                        (
                            "orient",
                            object(vec![
                                ("type", text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "to",
                                            object(vec![
                                                (
                                                    "oneOf",
                                                    Value::Array(vec![
                                                        object(vec![
                                                            ("type", text("integer")),
                                                            ("description", text("Its id.")),
                                                            ("minimum", Value::Number(1.0)),
                                                        ]),
                                                        object(vec![
                                                            ("type", text("string")),
                                                            (
                                                                "description",
                                                                text("Its name — \"Sphere 1\"."),
                                                            ),
                                                            ("minLength", Value::Number(1.0)),
                                                        ]),
                                                    ]),
                                                ),
                                                (
                                                    "description",
                                                    text(
                                                        "A plane element, a section, or \"XY\", \"YZ\", \"XZ\".",
                                                    ),
                                                ),
                                            ]),
                                        ),
                                        (
                                            "relation",
                                            object(vec![
                                                ("type", text("string")),
                                                (
                                                    "enum",
                                                    Value::Array(vec![
                                                        text("normal"),
                                                        text("inPlane"),
                                                    ]),
                                                ),
                                                (
                                                    "description",
                                                    text(
                                                        "normal (the default): the direction is the plane’s normal. inPlane: it lies in the plane.",
                                                    ),
                                                ),
                                            ]),
                                        ),
                                    ]),
                                ),
                                ("required", Value::Array(vec![text("to")])),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    text(
                                        "Align the direction to a plane, as designed; the measured fit is kept beside it.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "extend",
                            object(vec![
                                ("type", text("object")),
                                (
                                    "properties",
                                    object(vec![
                                        (
                                            "start",
                                            object(vec![
                                                ("type", text("number")),
                                                (
                                                    "description",
                                                    text(
                                                        "Cylinder: mm past the start (negative pulls it in).",
                                                    ),
                                                ),
                                            ]),
                                        ),
                                        (
                                            "end",
                                            object(vec![
                                                ("type", text("number")),
                                                ("description", text("Cylinder: mm past the end.")),
                                            ]),
                                        ),
                                        (
                                            "uMin",
                                            object(vec![
                                                ("type", text("number")),
                                                (
                                                    "description",
                                                    text(
                                                        "Plane: mm past the edge along its first axis.",
                                                    ),
                                                ),
                                            ]),
                                        ),
                                        (
                                            "uMax",
                                            object(vec![
                                                ("type", text("number")),
                                                ("description", text("Plane: the opposite edge.")),
                                            ]),
                                        ),
                                        (
                                            "vMin",
                                            object(vec![
                                                ("type", text("number")),
                                                (
                                                    "description",
                                                    text("Plane: along its second axis."),
                                                ),
                                            ]),
                                        ),
                                        (
                                            "vMax",
                                            object(vec![
                                                ("type", text("number")),
                                                ("description", text("Plane: the opposite edge.")),
                                            ]),
                                        ),
                                    ]),
                                ),
                                ("additionalProperties", Value::Boolean(false)),
                                (
                                    "description",
                                    text(
                                        "Mm a cylinder or plane is drawn and exported past the measured surface.",
                                    ),
                                ),
                            ]),
                        ),
                        (
                            "name",
                            object(vec![
                                ("type", text("string")),
                                (
                                    "description",
                                    text("A name; else numbered by kind — \"Sphere 2\"."),
                                ),
                                ("minLength", Value::Number(1.0)),
                            ]),
                        ),
                    ]),
                ),
                ("required", Value::Array(vec![text("kind")])),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
    ])
}
