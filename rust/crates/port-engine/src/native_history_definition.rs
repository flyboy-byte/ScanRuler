// SPDX-License-Identifier: AGPL-3.0-only
//! Frozen history command metadata.
use super::super::{object, value_text};
use crate::schema_value::Value;
pub(super) fn undo() -> Value {
    object(vec![
        ("name", value_text("undo")),
        ("title", value_text("Undo")),
        (
            "description",
            value_text(
                "Undo the last step — one of the person’s or one of these commands, each of which is one step. Returns the step’s name and the session afterwards in brief (detail \"full\": the whole of it).",
            ),
        ),
        (
            "input",
            object(vec![
                ("type", value_text("object")),
                (
                    "properties",
                    object(vec![(
                        "detail",
                        object(vec![
                            ("type", value_text("string")),
                            (
                                "enum",
                                Value::Array(vec![value_text("brief"), value_text("full")]),
                            ),
                            (
                                "description",
                                value_text(
                                    "brief (the default): what the steps change, in brief, as after; full: the whole session, as state.",
                                ),
                            ),
                        ]),
                    )]),
                ),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
        ("history", Value::Boolean(false)),
        ("headless", Value::Boolean(true)),
    ])
}
pub(super) fn redo() -> Value {
    object(vec![
        ("name", value_text("redo")),
        ("title", value_text("Redo")),
        (
            "description",
            value_text(
                "Redo the step undone last. Returns the step’s name and the session afterwards in brief (detail \"full\": the whole of it).",
            ),
        ),
        (
            "input",
            object(vec![
                ("type", value_text("object")),
                (
                    "properties",
                    object(vec![(
                        "detail",
                        object(vec![
                            ("type", value_text("string")),
                            (
                                "enum",
                                Value::Array(vec![value_text("brief"), value_text("full")]),
                            ),
                            (
                                "description",
                                value_text(
                                    "brief (the default): what the steps change, in brief, as after; full: the whole session, as state.",
                                ),
                            ),
                        ]),
                    )]),
                ),
                ("additionalProperties", Value::Boolean(false)),
            ]),
        ),
        ("history", Value::Boolean(false)),
        ("headless", Value::Boolean(true)),
    ])
}
