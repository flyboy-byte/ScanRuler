# Preserve original error-message UTF-16 units

The original CommandError takes a JavaScript string. A Rust String alone cannot
represent lone surrogates, which can occur in JSON-supplied names and then in a
source error message. The owned engine envelope therefore retains UTF-16 units
and emits well-formed JSON escaping unpaired surrogates exactly as the original
JSON.stringify does. Valid pairs remain their original Unicode scalar; no text
normalization is introduced. Human terminal Display may use replacement characters,
but wire serialization and the lossless display-unit accessor must not.

This is an owned type with explicit message-size/allocation limits, not a global
session or a general JavaScript runtime. Scope: the complete typed CommandError
constructor/toJSON and initial name/display observations for the twelve source
codes. Arbitrary JavaScript thrown-value coercion, mutable Error/prototype/stack
behavior, registry normalization/stale-page detection and job/history semantics
remain distinct follow-ups. Do not silently classify unknown faults as internal
or invent rollback guarantees under this envelope's name.

Use no new third-party dependency. Fixed inputs cover all twelve codes, empty,
control/escaping/Unicode/lone-surrogate messages and seeds 1/7/99/1039, with source
observations captured before Rust implementation. Future engine/platform callers
must use the lossless wire representation and retain the original code meanings.
