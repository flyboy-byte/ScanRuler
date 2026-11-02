# Fixed ASCII PLY inputs

Authored/generated for this port under AGPL-3.0-only; not third-party scan data.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7::src/core/parsers/ply.ts`.
`manifest.json` records input hashes, seed schedule and source-success/refusal
classification. No expected coordinate/index output is generated from Rust.

Regenerate inputs with `python3 docs/port-plan/tools/generate_ply_ascii.py`.
Changes to a passing case, seed or input hash require explicit review; regeneration
is not permission to rebaseline comparisons. The Rust compiled schedule mirrors
the manifest, and the Python fast-gate test verifies files, hashes and schedule.

The initial 56 cases were captured against the original before the first Rust
comparison (43 values / 13 refusals). Four cases were then added, retaining every
old input: successful repeated vertex elements, appending repeated face elements,
an ignored negative-count element, and a missing-format refusal. Final corpus:
60 cases, 46 successful values / 14 source refusals, seeds 1/7/99/1039.

Comparisons are exact bytes for every float32 coordinate (including zero sign and
NaN bits), uint32 fan index, array count, indexed kind and optional winding flag.
Progress and refusal messages are exact UTF-8. There is no adjustable numerical
tolerance. Invalid coordinates/indices are source parser observations; downstream
mesh validation is a separate operation.

Coverage includes CRLF/multibyte/replacement/BOM text, ECMAScript whitespace and
numeric prefixes, reordered/duplicate/list coordinate properties, ignored colors
and normals, unknown elements and types, face aliases/fans/short lists, missing
tokens, negative/NaN cursor and count behavior, repeated elements and the source's
64 KiB header search. Huge declared workloads have separate Rust-only host guard
tests; the oracle is never asked to execute unbounded hostile loops.

Binary PLY is explicitly unported at this unit's boundary. Browser-Wasm runtime
and public scan command integration remain open.
