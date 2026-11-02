# PLY browser-Wasm comparison contract

Specified 2026-11-02, unit `harness.wasm.ply`; extends the existing safe private
test ABI, not a product interface. Existing 114 plane/graph/marked-plane browser
inputs and their comparators remain mandatory and unchanged.

## Fixed scope before runtime implementation

Mode 4 receives only raw PLY bytes. Rust runs the complete all-encoding parser
with bounded byte/vertex/triangle/work limits. The page never receives original
or native expected outputs. Node compares independently produced source/native
stdout and progress/error bytes against the Rust result after execution.

The output envelope is status (u8: 0 success, 2 source refusal), payload byte
length (u32 LE), complete PLY1 output, progress/error UTF-8 length (u32 LE), and
complete progress/error bytes. A source refusal has an empty payload and explicit
error text. Host allocation/resource/codec failures fail the run rather than
becoming successful paired source refusals. Every envelope byte is exact;
nonfinite source parser values and zero signs are compared as raw float32 bits.

Run all 60 ASCII cases and 167/168 binary cases. The **only new runtime exclusion**
is `io.ply.binary/le-vertex-progress`: its 3 MiB input/12 MiB output exceeds the
private ABI's existing 1 MiB limits. Keep it in the native suite, record the exact
exclusion and reason, and leave broader browser memory/vertex-progress coverage
open. The million-face progress case stays in the browser corpus. No previously
passing browser case is excluded or changed.

Expected combined inventory: five suites / four codecs / **341 browser cases**,
287 successful outputs and 54 source refusals, plus protocol checks. Numerical
budgets for the three original codecs remain unchanged. Require validated
current-SHA native inputs, original/native outputs and progress sidecars; a
missing/stale/corrupt file or unlisted exclusion fails before execution.

Add mutation/shape/sidecar validation tests and retain all failing actual bytes
before comparison assertions. Actual Chrome, compiler/module hash, memory bound
and exact tested SHA are required evidence. Node-Wasm smoke remains diagnostic.
No UI/session/project integration or accepted-unit/parity claim follows from this
bounded harness extension.
