# Actual Chrome PLY runtime — 2026-11-02

Tested implementation: `8bc73636b40a095f67a0e31f1cf4839daf13c789`.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.
Chrome **156.0.8078.79**, Node **22.23.4**, system Rust **1.99.1**.
Module SHA256: `c27544516eddfd1c0598e09472e6c3d7cd675382e2b731028d303636e5f085dc`.

Actual Chrome executed **341 original/native/browser-Wasm comparisons**:
287 complete successful outputs / 54 source refusals, plus **13 protocol checks**.
The module has no imports and the page had no errors. Browser-observed module hash
matches the built module. The current-SHA native artifact covered all 20 suites /
914 identities; its hash and the compiler identity are in the report.

All original 114 TLS/graph/marked-plane browser inputs remain, with their prior
budgets and strict fields unchanged. Added: all 60 ASCII PLY cases and 167 binary
PLY cases. Source/native stdout plus progress/error sidecars are verified against
the native manifest before the browser starts. Only raw input bytes and codec
number enter the page/Rust instance; original/native outputs are never served.

Every PLY coordinate/index/count/winding/progress/error byte matches exactly,
including nonfinite IEEE values and zero signs. Host budget/allocation/codec
failures cannot become passing source refusals. The million-face progress case
passes at its full original count in Chrome. The **only new runtime exclusion** is
`io.ply.binary/le-vertex-progress`, whose 3 MiB input/12 MiB output exceeds the
existing private ABI's 1 MiB caps. It stays fully compared in native, is recorded
as `not-run` in the browser report and remains an explicit runtime follow-up.
No previously passing browser case is excluded or changed.

`report.json` is the actual runtime report, `corpus.json` the exact roster snapshot,
`cases.csv` the identity index, and `manifest.json` their hashes and frozen case
identities. **All raw input/original/native/Wasm bytes** are retained in
`raw.json.gz`; PLY entries also retain both raw progress files and independent
source/native protocol envelopes. The comparator regression replays every raw
comparison and validates hashes. The compiled debug module is reproducible from
the tested source/toolchain and retained by hosted CI rather than committed as
application code. Debug-path differences can change module hashes across hosts;
each run must still prove its own browser/built hash identity.

Reproduction: `cargo xtask ci`, `cargo xtask oracle`, exact-current-SHA
`ci_report.py evidence`, `cargo xtask wasm-probe`, then Node 22 running
`run-browser-wasm.mjs` with `CHROME` set. Node-Wasm smoke cannot substitute for the
actual Chrome result. The private test ABI remains bounded to 64 MiB linear memory
and imports only existing owned Rust crates; no new third-party dependency.

This is bounded runtime evidence. Public session/commands/history/projects/UI,
the large vertex-progress browser workload, wider environments, the P37 exact-base
ratchet and P39 tooling replacement remain open. No numerical unit is accepted.
