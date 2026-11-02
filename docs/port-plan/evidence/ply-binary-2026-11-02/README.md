# Complete binary PLY body — 2026-11-02

Unit: `io.ply.binary`; implementation:
`98d39fc4cb54ae1c523da619e54164f5d99f9105`.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, app 0.9.2.

The original `parseBinaryBody` and `readScalar` are ported through their complete
return. `ply::read_ply` dispatches ASCII and both binary byte orders through one
factored header. The ASCII-only convenience function remains available with a
distinct binary-boundary error; it is no longer the all-encoding entrypoint.

**168 independent original/native comparisons pass: 152 complete outputs and 16
source refusals.** All 16 source scalar aliases are exercised in vertex,
list-count and index roles in both endians. The initial 166 inputs were committed
before implementation and source-captured before the first Rust comparison; two
original-captured double IEEE cases extend them without replacing old inputs.
Seeds 1/7/99/1039 remain in both byte orders. Both million-record progress cases
execute at their real counts. Every coordinate/index/count/winding byte and full
progress/refusal string is exact, including IEEE NaN payloads and zero signs.
No numerical tolerance, normalization or changed seed is used.

The complete scalar conversion, fixed-stride and vertex-list branches, unknown
element skipping, face sizing/fill passes, progress and final result are present.
Source subtleties retained include unconditional first/previous reads for short
lists, replacing previous binary face arrays (ASCII appends), fractional counts
producing partial arrays with ignored out-of-range writes, and NaN offsets from
unknown ignored sizes. Invalid parser output is not repaired or called valid
geometry. Downstream graph validation remains separate.

`report.json` contains exact identities, each input hash and every source/Rust
output and progress byte. Small outputs are inline hex; the million-vertex
outputs are separate deterministic gzip with compressed and decoded hashes.
Inputs, including the generated progress case, are committed under
`fixtures/ply-binary/`, with generator/license/seed/hash manifest. No source output
is fed into Rust. Modified-coordinate and missing-word comparator witnesses run
against the actual original float triangle. Failed raw outputs are written before
assertions. The shared header change also strengthens all 60 existing ASCII
source comparisons to check the full dispatcher values and progress; their new
sidecars are mandatory in the raw artifact inventory.

Caller byte/vertex/triangle/work limits and fallible storage are host safeguards,
not alternative source formulas. Huge hostile declarations are tested against
Rust limits without running unbounded original allocations. No foreign kernel,
third-party crate or new npm dependency is introduced. The source TypeScript
compiler remains reference tooling only.

Native/Wasm compilation and ordinary host guards are checked; real browser-Wasm
PLY execution, public scan/session/history/project commands and broader P03
acceptance remain open. The umbrella `io.ply` remains planned for that broader
acceptance. No unit is promoted to accepted or whole-application parity.
