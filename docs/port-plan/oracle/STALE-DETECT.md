# Stale-import classification

2026-11-02. PR #60 ports only `isStaleImport` at pinned 2cd89c6.

The four complete alternatives retain the original substring and legacy `/i`
semantics. ASCII case folds; non-ASCII lookalikes do not become ASCII. Actual
CommandError values return false before examining their message. Non-error,
non-string values do not coerce their message property or array contents.

The host classifies runtime categories and supplies borrowed owned UTF-16 text.
A separate text budget bounds scanning, without turning host refusal into false.
Arbitrary mutable JS getters/non-string Error.message properties are outside
this typed boundary. No page clock, staleError normalization, browser preload
listener or actual command integration is claimed.

The source-only capture precedes Rust. Both complete original modules are
compiled with pinned TypeScript; runtime type imports erase naturally. There are
no behavioral dependency stubs or rewritten bodies. The 308 full source traces
include boolean result and unchanged null page timestamp. Inputs cover four
patterns, mixed case, prefixes/suffixes, malformed surrogates, non-ASCII spaces,
missing characters and six runtime categories, plus ten empty/unrelated types.
All source/native outputs match exactly. Full modules/maps/provenance and paired
raw input/original/Rust artifacts are retained. No old suite is removed.

Run `cargo xtask ci` and `cargo xtask oracle`; the latter invokes the live pinned
comparison, not committed reference answers. This branch's native roster is
32 suites /3,403 identities; stacked units not yet on its main base are separate.
Wasm compilation is required; actual browser predicate execution remains open.
