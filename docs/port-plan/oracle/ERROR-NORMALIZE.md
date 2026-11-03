# Command fault normalization

2026-11-02. PR #65 ports complete asCommandError at owned host boundaries.
It composes stale detection and stale envelope construction from #60/#61/#63.
Actual CommandError values retain their exact shared allocation identity and
bypass callbacks/limits. Error/string import failures mark the page and answer
stale_page. Ordinary errors and other thrown values answer failed. Other values
cannot become stale merely because their String conversion contains import text.

304 source-first cases cover all twelve typed codes, all four import alternatives,
empty/ordinary/UTF-16/non-ASCII lookalike messages, existing stale state, clock
failures and object/array/failing coercion. They compare complete name/code/text,
identity equality, clock/coercion order, exceptions and page state exactly.
The entire frozen registry graph is built without behavioral stubs. Its sole
private adapter appends an export; a source-map proof checks the complete pinned
registry body plus exactly that append. Full bundles/maps/provenance, independent
inputs/source-only capture and paired raw bytes are retained.

The host classifies caught categories, owns Error messages and supplies fallible
String conversion for other values and ISO UTC clock text. Mutable Error.message
getters/prototype mutation, general native String conversion, actual platform
clocks and full registry execution remain separate. Source outputs never become
Rust inputs. Non-Clone native coercion faults preserve payload; host storage
refusal stays separate from the source code. Arc allocation uses ordinary Rust
allocation behavior, not process-wide OOM isolation.

Native roster on this stack:35 suites /3,891 identities. Run cargo xtask ci and
cargo xtask oracle in rust/. Wasm compile is required, not actual browser
normalization execution or full session acceptance.
