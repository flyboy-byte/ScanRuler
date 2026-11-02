# Typed engine error envelope — 2026-11-02

Unit: `engine.error.envelope`; implementation:
`f509ae2b667560a9a3532b50df14c149f3b2f562`.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7::src/commands/types.ts`.

The complete typed CommandError constructor/toJSON contract is ported: all twelve
codes, initial name/message, initial Error.toString and the exact two-field JSON
representation. **52 complete original/native field comparisons pass**, covering
every code with empty, escaping/control, Unicode and lone/paired surrogate text,
plus seeds 1/7/99/1039. The fixed inputs and original observations precede Rust
implementation. Every code/name/message/display/JSON byte is compared, not error
status. Changed code and missing JSON data are rejected against actual source
output. Inputs and full original/Rust outputs/hashes/identities are retained in
the adjacent JSON report and fixed fixture directory.

The original class executes from the frozen Git object after only TypeScript
annotation erasure. Source output never enters Rust. Rust owns UTF-16 units so
JSON-supplied lone surrogates survive unchanged as well-formed JSON escapes.
Valid pairs retain the original Unicode scalar, with no normalization. Constructor
message limits and allocation failures are distinct host errors; no truncation or
replacement text can silently enter the wire message. UTF-8 budget validation
stops after at most the caller's limit plus one UTF-16 unit.

`display_units` preserves initial String(error) losslessly. Standard terminal
Display uses Unicode replacement only where UTF-8 cannot represent a lone
surrogate. **Do not use Display as the wire message or as the unported source
errorMessage helper:** the latter reads Error.message without a name prefix.
The wire path is `to_json`, and the raw message path is `message_units`.

These are 52 constructed error-object observations, **not working commands**.
The registry's asCommandError normalization, arbitrary thrown-value coercion,
stale-page detection, JS Error object mutation/prototype/stack state, jobs,
history and session ownership remain outside this bounded unit. In particular,
source unknown faults normalize to failed unless stale; no internal-code mapping
or rollback guarantee is invented here. Timeout continues to mean partial
changes may remain, without implementing timeout execution itself.

Native ordinary ownership/cap tests and Wasm compilation pass. Actual browser
envelope runtime and public session/command integration remain open; no accepted
state or G2 completion. The shared required native gate preserves all earlier
suites and adds this one. No dependency or original source change.
