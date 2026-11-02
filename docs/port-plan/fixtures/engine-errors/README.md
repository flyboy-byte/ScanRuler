# Original typed command-error inputs

Authored/generated under AGPL-3.0-only. No private data. Generate with
`python3 docs/port-plan/tools/generate_engine_error_cases.py`.
The 52 inputs were committed and source-captured before Rust implementation.

Each test input contains a source-union code index (u8), UTF-16 unit count (u32
little-endian) and exact message units (u16 little-endian). Every one of the twelve
codes is constructed with empty, escaping/control, Unicode and lone/paired
surrogate messages; four additional seeded messages retain seeds 1/7/99/1039.
The hash manifest and compiled Rust schedule must retain all cases unchanged.

The independent adapter executes the entire pinned CommandError class after
TypeScript annotation erasure, and serializes code, initial name, message, initial
String(error) and JSON.stringify(toJSON()). The ERR1 test frame length-prefixes
code UTF-8, name/message/display UTF-16 and JSON UTF-8. Compare every field byte
exactly; constructor success/error status alone is insufficient.

These are **52 constructor/serialization observations, not 52 working commands**.
The source errorMessage/asCommandError coercion and registry/session/job/history
execution are separate unported operations. Arbitrary mutation of JavaScript
Error objects, prototypes and stack traces is not the envelope contract.
