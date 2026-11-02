# OBJ browser-Wasm comparison contract

Specified 2026-11-02, unit `harness.wasm.obj`. Preserve every existing 341 browser
case, input, comparator and numerical budget. This extends the safe private test
ABI, not the product/session interface. No dependency or PLY implementation edit.

Mode 5 receives the exact authored OBJ/decoded-companion input packet and calls
the complete Rust parser. Mode 6 receives the authored helper-input packet and
calls complete Rust material/sampler/bake helpers. Page/Rust receive input only;
the original's decoded outputs or intermediate answers never enter Rust. Given
RGBA bytes and initial sampler buffers are independently authored fixture inputs.

Compare complete OBJ1 and OTX1 outputs byte for byte: geometry/kind/indices,
colors, ordered filenames/maps/texture notes, progress, source refusals and the
sampler's complete modified/untouched buffer. Host resource/codec/allocation
failures fail the run; they must not become paired source refusals or absent notes.

Run all 144 helper cases and 79/81 parser cases. The two **new** runtime exclusions
are `io.obj.mesh/line-progress` and `io.obj.mesh/short-unicode-line-skips-progress`,
whose full-size inputs exceed the existing private ABI's 1 MiB cap. Retain both in
native comparisons, with exact exclusion reasons; do not reduce their sizes or
count them as browser passes. The old PLY vertex workload exclusion is unchanged.

Expected current roster: seven suites / six codecs / **564 browser cases**,
482 successful output/helper states and 82 source refusals. The original 341
cases remain mandatory. Historical proof uses its immutable v2 roster; the live
runner must require v3, never silently accept the smaller historical roster.

Keep 1 MiB input/output and 64 MiB linear memory caps. Bound strings, arrays,
files, vertices, faces and work before allocation/iteration. Validate output
shapes and retain failing raw output before comparison assertions. Require exact
candidate/reference/native-file/module hashes, no module imports and actual
Chrome evidence. A Node-Wasm diagnostic is insufficient. Image-file decoding,
file-set ingress, public commands/UI/projects and larger browser workloads remain
open; no accepted-unit or whole-product parity claim.
