# Actual Rust execution in browser Wasm

Scope 2026-11-01: TLS plane (13 existing cases), complete mesh graph (50) and marked-plane elements (51). Original frozen Node outputs and native Rust outputs come from the existing independent source suites. The browser receives only mode plus fixture input bytes. The local HTTP server serves only an empty page and the compiled Wasm module, never expected results.

This is a private test ABI, not a product/session interface. `port-wasm-probe` uses safe Mutex-owned buffers and five scalar functions (reset/push/run/length/word). It inherits unsafe_code=forbid and adds no dependencies beyond port-core. There are no raw pointer exchanges, no unsafe no_mangle attributes, wasm-bindgen, C/C++ kernel or JavaScript geometry implementation. A test-only linker export-all/link-dead-code build exposes unique v0 symbol suffixes; the runner fails if lookup changes. The module must import nothing. Future production ABI design remains under P05.

Limits: one MiB input/output, 30,000 coordinate scalars, 10,000 indices/selected points and 64 MiB linear memory. Valid source refusal results are serialized as results; malformed codec, invalid mode, exhausted input and allocation errors return nonzero and clear stale output. Browser tests deliberately exercise bad mode, truncated/oversized declarations, poisoned writes, out-of-range reads, hard memory-growth refusal and clean reset/recovery. No browser exception or missing result can become a pass.

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

Actual hosted browser results must be recorded before claiming execution coverage; a compiled module or Node smoke is insufficient. Even a passing run covers only these 114 inputs (90 successful value cases and 24 source refusals). It does not provide UI/session/command/project acceptance, all-unit browser parity, the general P36 protocol, P37 exact-base coverage ratchet, P39 Rust tooling replacement or P35 whole-product purity. The JS launcher/comparator is temporary source-oracle test tooling and browser serialization only; algorithms execute in Rust.

## Verified Chrome checkpoint

Hosted run [41375102435](https://github.com/flyboy-byte/ScanRuler/actions/runs/41375102435) passed the actual browser lane and strict Rust aggregate at source head 43e397793d301143984243e05cca7f3a76e6937a, tested CI merge 00fd1a69af98c7bacc91da747483ac0ccbfb70c6. Chrome 156.0.8078.47, Node 22.23.4 and Rust 1.99.1 executed all 114 original/native/Wasm comparisons: 90 successful values and 24 source refusals. Ten protocol checks passed, including hard memory-growth refusal, poisoned output and recovery. No page errors or module imports. Browser-observed module SHA256 matches the compiled module, 68303a86412f3a142d1a807ea420060eae4b69573215a64830b60c64982f074a.

Full immutable report, 114-row identity CSV and report/module hash manifest: [evidence](../evidence/browser-wasm-2026-11-01/manifest.json). Worst absolute numeric difference 1.1102230246251565e-16; original budgets unchanged. Compiled module and every raw source/native/Wasm/input payload remain in the SHA-named CI artifact for thirty days and are reproducible from the pinned code/fixture generators. Coverage remains bounded to these cases/operations, not the whole Rust app.

Failure retention audit: raw input/original/native/Wasm files are written before any numerical assertion can throw, and the failing unit/case remains in the report. A deliberate changed-value regression proves the mismatching bytes survive. The original successful Chrome snapshot above remains immutable; later CI runs exercise this retention refinement as well.
