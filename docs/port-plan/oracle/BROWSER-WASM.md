# Actual Rust execution in browser Wasm

2026-11-03: [sphere extension](SPHERE-WASM.md) adds all1560 sphere cases; the live v5 roster requires2211 cases while retaining every earlier651 case and all older budgets/exclusions. Full actual Chrome proof: evidence/browser-wasm-sphere-2026-11-03/manifest.json. This is library execution, not session/command acceptance. Historical checkpoints below retain their original scope.

Scope 2026-11-02: TLS plane (13 cases), complete mesh graph (50), marked-plane
elements (51), ASCII PLY (60), binary PLY (167), OBJ texture helpers (144) and
complete OBJ parser (79) and companion file states (87). All earlier 564 browser cases remain unchanged.
Original frozen Node outputs and native Rust outputs come from independent
source suites. The browser receives only mode plus authored fixture input bytes.
The local HTTP server serves only an empty page and compiled Wasm module, never
expected results. [PLY contract](PLY-WASM.md) and [OBJ contract](OBJ-WASM.md)
record three explicit native-only workload exclusions. The live v4 roster cannot
be replaced by smaller historical v2/v3 rosters.

This is a private test ABI, not a product/session interface. `port-wasm-probe` uses safe Mutex-owned buffers and five scalar functions (reset/push/run/length/word). It inherits unsafe_code=forbid and depends only on existing owned port-core/port-io crates. There are no raw pointer exchanges, no unsafe no_mangle attributes, wasm-bindgen, C/C++ kernel or JavaScript geometry implementation. A test-only linker export-all/link-dead-code build exposes unique v0 symbol suffixes; the runner fails if lookup changes. The module must import nothing. Future production ABI design remains under P05.

Limits: one MiB input/output, 30,000 geometry coordinate scalars, 10,000 geometry indices/selected points and 64 MiB linear memory. PLY additionally caps declared vertices/triangles at 10,000 and work at 4,000,000. Its million-face zero-property progress case fits these bounds; its million-vertex input/output does not and stays explicitly native-only. OBJ additionally caps vertices/UVs/face corners at 10,000, indices at 30,000, materials at 1,024, companion files at 64, names at 4,096 bytes and work at 4,000,000. Both full-size million-line OBJ progress inputs exceed the input cap and remain unchanged in native comparisons. Typed packet counts/UTF-8/trailing data are checked before execution. Valid source refusals are serialized as results; malformed codec, invalid mode, exhausted input and allocation errors return nonzero and clear stale output. Browser tests deliberately exercise bad mode, truncated/oversized declarations, poisoned writes, out-of-range reads, hard memory-growth refusal, clean reset/recovery and distinct PLY host-versus-source failures. No browser exception or missing result can become a pass.

From the fork root, system Rust/Wasm target, Node 22 and Chrome plus the original npm dependencies are required:

```bash
(cd rust && cargo xtask ci && cargo xtask oracle && cargo xtask wasm-probe)
python3 docs/port-plan/tools/ci_report.py evidence --candidate "$(git rev-parse HEAD)"
npm ci
node --test docs/port-plan/oracle/browser-wasm-contract.test.mjs
node docs/port-plan/oracle/run-browser-wasm.mjs
```

The CI browser-wasm lane downloads the current native artifact by exact checkout SHA, builds its own module, and runs Chrome. The final Rust gate requires this lane. Runs record source/candidate/input/native/original/Wasm/module hashes, browser identity, actual export symbols, compiler and raw values/state. Output is under rust/target/browser-wasm and retained 30 days. The module build candidate, native artifact candidate, expected reference and every input/output hash must match before execution. Full canonical case identities and counts must remain.

Local diagnostic mode `node .../run-browser-wasm.mjs --node-smoke` can execute the identical module without Chrome, but records diagnostic-node-wasm-not-browser in a separate directory and never substitutes for the required browser lane. Its result cannot establish browser runtime evidence.

Comparisons retain existing contracts: complete graph bytes exact; TLS/marked numerical fields within 1e-12*(1+abs(original)); exact presence/kind/error/count/region, and marked zero signs. All 45 marked-frame angle probes remain, with no alternate-frame normalization or original-derived constants. Negative comparator tests reject changed values, missing/nonfinite outputs, changed presence, graph bytes, region and signed zero. No tolerance, input, seed or existing passing comparison is removed.

Actual browser results must be recorded before claiming execution coverage; a compiled module or Node smoke is insufficient. The current local Chrome checkpoint covers 651 inputs (569 successful outputs/helper states and 82 source refusals), with three native-only workloads explicitly excluded. Helper observations are not working import commands. It does not provide UI/session/command/project acceptance, all-unit browser parity, the general P36 protocol, P37 exact-base coverage ratchet, P39 Rust tooling replacement or P35 whole-product purity. The JS launcher/comparator is temporary source-oracle test tooling and browser serialization only; algorithms execute in Rust.

## Companion file Chrome checkpoint — 2026-11-02

Clean tested candidate `dbe8fcc78b1b6efefe2e6eb11f8d05c5bf049d44`: Chrome 156.0.8078.79,
Node 22.23.4, Rust 1.99.1. All 651 comparisons and 24 protocol checks pass, with zero
module imports/page errors. All 87 new file states compare exactly: full
extension/classifiers/constants, selection and every identity/association after
each operation, ordered original names/bytes. Existing budgets/exclusions remain.
The owned input-only codec caps files/references 64, operations 64, names 4096 bytes,
links 4096, per-operation work 4m, and retains the 1 MiB input/output/64 MiB memory caps.
[Manifest and complete raw proof](../evidence/browser-wasm-files-2026-11-02/manifest.json).
24 comparator/proof tests replay every historical/current result and reject
mutations, malformed state, lost artifacts and trimmed rosters. Final hosted
head outcomes must be inspected separately. No public import command is claimed.

## OBJ local Chrome checkpoint — 2026-11-02

Clean tested checkpoint `239012c9648f645a3039856e5c511857fb62e691` (codec implementation
`6bcab1d1a436fb757ce2023a001f70d3972b5a05`): Chrome
156.0.8078.79 / Node 22.23.4 / Rust 1.99.1 passes 564 comparisons and 18 protocol
checks. Zero module imports or page errors. All OBJ values, float32 bits, colors,
ordered notes/filenames/maps, complete sampler buffer and progress/refusals match
exactly. Existing numerical budgets stay unchanged.
[Manifest, complete raw bytes and identities](../evidence/browser-wasm-obj-2026-11-02/manifest.json).
20 comparator/proof tests replay old and new raw proofs and reject mutations,
malformed outputs, missing/stale artifacts and trimmed rosters. Final hosted
head outcomes must still be inspected separately.

## Expanded local Chrome checkpoint — 2026-11-02

Implementation `8bc73636b40a095f67a0e31f1cf4839daf13c789`: Chrome
156.0.8078.79 / Node 22.23.4 / Rust 1.99.1 passes 341 comparisons and 13 protocol
checks, with no module imports or page errors. PLY bytes/progress/refusals match
exactly, including IEEE bits; the original numerical budgets are unchanged.
[Manifest and complete raw evidence](../evidence/browser-wasm-ply-2026-11-02/manifest.json).
Hosted final-head outcomes must still be inspected separately.

## Historical hosted Chrome checkpoint — 2026-11-01

Hosted run [41375102435](https://github.com/flyboy-byte/ScanRuler/actions/runs/41375102435) passed the actual browser lane and strict Rust aggregate at source head 43e397793d301143984243e05cca7f3a76e6937a, tested CI merge 00fd1a69af98c7bacc91da747483ac0ccbfb70c6. Chrome 156.0.8078.47, Node 22.23.4 and Rust 1.99.1 executed all 114 original/native/Wasm comparisons: 90 successful values and 24 source refusals. Ten protocol checks passed, including hard memory-growth refusal, poisoned output and recovery. No page errors or module imports. Browser-observed module SHA256 matches the compiled module, 68303a86412f3a142d1a807ea420060eae4b69573215a64830b60c64982f074a.

Full immutable report, 114-row identity CSV and report/module hash manifest: [evidence](../evidence/browser-wasm-2026-11-01/manifest.json). Worst absolute numeric difference 1.1102230246251565e-16; original budgets unchanged. Compiled module and every raw source/native/Wasm/input payload remain in the SHA-named CI artifact for thirty days and are reproducible from the pinned code/fixture generators. Coverage remains bounded to these cases/operations, not the whole Rust app.

Failure retention audit: raw input/original/native/Wasm files are written before any numerical assertion can throw, and the failing unit/case remains in the report. A deliberate changed-value regression proves the mismatching bytes survive. The original successful Chrome snapshot above remains immutable; later CI runs exercise this retention refinement as well.
