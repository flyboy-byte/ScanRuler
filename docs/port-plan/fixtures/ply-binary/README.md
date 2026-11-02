# Binary PLY input corpus

Authored/generated under AGPL-3.0-only for the frozen
`2cd89c6769a10c27ee7754b477ada521c9ef5cb7::src/core/parsers/ply.ts`.
No external/private scan data. Generate with
`python3 docs/port-plan/tools/generate_ply_binary.py`.

The initial **166 inputs were committed before binary implementation**, and all
source outcomes were captured before the first Rust comparison. That complete
run passed 150 outputs / 16 refusals. Two double-precision IEEE cases were then
captured from the original and added without changing/removing any old input.
Final schedule: **168 inputs, 152 outputs / 16 refusals**, four seeds
(1/7/99/1039) in both byte orders. Manifest hashes and the compiled Rust schedule
are checked by the fast gate. No seed, input or tolerance can be quietly reduced.

Every one of the 16 source scalar aliases appears in vertex, list-count and list
index roles in both byte orders. Additional cases cover multibyte/winding headers,
mixed/ignored properties, variable-length vertex records, list-named coordinates,
unknown fixed/list elements, repeated vertices/faces, face-before-vertex, polygon
fans, short-list unconditional reads, fractional/negative/NaN counts, malformed
NaN offsets, truncation/type refusals, Uint32 wrap, integer boundaries, signed zero,
infinity, subnormal and quiet/signaling float32/float64 values.

Both source progress boundaries run at **1,048,577 records**, not a reduced count
or injected callback. The vertex case uses zero char coordinates (3,145,731 body
bytes including one face); the face case needs no face property bytes, as in the
source. These are generated analytic input bytes, not large real scans.

Compare full PLY1 coordinate/index/count/winding bytes, all progress and all
source error text exactly. There is no numerical tolerance or NaN normalization.
Source outputs never enter the Rust parser. Host byte/vertex/triangle/work limits
are separate from the original semantics; unbounded hostile allocations are not
executed in the oracle. Browser-Wasm and public session integration remain open.
