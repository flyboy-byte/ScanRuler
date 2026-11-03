# Sphere execution in browser Wasm

2026-11-03. PR #90 depends on #89. Its private input-only mode8 decodes the exact
native sphere input packet: Float32 coordinate/Uint32 selection counts, clipping
k, held-radius presence/value, all coordinates and selected indices. It calls the
owned sphere implementation and serializes complete result/free presence, every
sphere/statistic value and ordered used IDs. No geometry is implemented in JS.

The live v5 roster requires all1560 sphere cases plus every earlier651 browser
case (2211 total). Historical v2/v3/v4 proofs still replay under their original
rosters. The three old native-only workload exclusions and every old mode's
numerical budget remain unchanged. Sphere's mode-specific budget is the draft
#89 contract: absolute1e-10 + relative1e-10 times absolute reference, exact used
IDs/optionals/nonfinite classes/zero signs. Negative controls prove the old TLS
budget stays tighter, reject malformed sphere tags/counts/companion/trailing
state, and reject changed values/IDs/signs and missing/stale native sidecars.

The page receives only mode and actual fixture bytes, never source/native outputs
or expected constants. Reference/native values remain in the coordinator; compare
all three pairs after independent execution. Sphere native descriptor/post-input
sidecars are mandatory and hash-checked before the page is used. Host failures
poison output and never become a completed numerical refusal. Six new protocol
checks extend the earlier24 without removing any. Input/output1MiB, coordinate
scalars30000, selected points10000 and linear memory64MiB stay fixed. Oversized
counts, unknown held presence, trailing bytes and invalid selections fail before
producing values. Empty valid sphere input is a completed source refusal.

Actual clean Chrome evidence must be recorded before claiming execution coverage.
Build/run the full native gates and raw inventory at one clean candidate, then
cargo xtask wasm-probe and Node22 oracle/run-browser-wasm.mjs. Retain module/native/
reference/candidate/browser identities and every actual source/native/Wasm/input
byte. A Node smoke or compiled Wasm artifact is not actual browser evidence.

This is bounded library execution, not a production ABI or usable command. Seeded
sphere growth, complete selection element fields and session/history/command/UI/
CLI wiring remain open, as do broader platform profiles and the generic exact-base
numerical ratchet. No accepted unit or M3 claim.

## Actual Chrome checkpoint

Clean implementation c33011c1d24db4dc5b66459f70aada6ea8d02a13 passed all2211 original/native/Chrome comparisons and30 protocol checks:1193 successful model/helper outputs and1018 source refusals. Zero module imports/page errors. Complete source/native/input/Wasm bytes, hashes, corpus and browser/module/native identities are retained in evidence/browser-wasm-sphere-2026-11-03/. Historical proofs and the new full proof replay in the comparator regression suite. The initial default launcher was absent; its pre-comparison failure report is retained. The unchanged checkpoint passed via the installed google-chrome-stable CHROME override, with no case/budget changes.
