<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Exact shortest primitive Number text — 2026-11-04

Complete combined implementation candidate: `dfb6d63a454ebd3d48be91634e156a4df47835a2`.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (app 0.9.2). Parent #100 final `8789795`.
The feature branch stays separate while #100 owns shared public PR files.

The independent actual scan-document comparison found 24 mismatching source-file
cases, all in generated PLY bytes: the exact f32-derived value
-141.142852783203125 printed as -141.14285278320313 in Rust, versus
-141.14285278320312 in the original. Those original controls and bytes remain.
The formatter is corrected rather than changing inputs or allowing byte tolerance.

Rust's existing shortest scientific formatter supplies the same shortest round-trip
candidate, but can choose an odd decimal coefficient at an exact decimal midpoint.
The source selects the even one. The correction tests the two adjacent candidates
using the exact dyadic mantissa/power and decimal powers of five. Odd normalization
bounds the integer work: a large power cannot fit/divide the 53-bit mantissa or
at-most-17-digit coefficient. No float subtraction or rounded decimal scaling is
used to decide the tie; no dependency or foreign formatter is introduced. Existing
special values, signed zero and exponent thresholds retain their source behavior.

## Independent observations and failed runs

The actual frozen registry asCommandError numeric-throw branch provides primitive
String conversion; Node22 JSON.stringify supplies JSON number text. 6,241 independent
IEEE controls compare input bits, primitive category, full message and JSON text,
without tolerance. They include authored counterexamples and threshold neighbours,
plus 1,024 raw f64 and 1,024 promoted f32 samples for each fixed seed 7/1039/65537.
Original outputs never enter the native run.

The unchanged first run found 11 mismatches. Exact integer midpoint correction
matches all 6,241. The actual wrong-text canary still fails with exit 101 and
positive divergences; changed/missing-output controls remain for every case.
Direct native regression values use exact IEEE bits, preserving the original
midpoints while satisfying Clippy's unchanged precision lint.

Changing the shared helper correctly invalidated the inherited PLY/STL fuzz source
hashes. Those checks were not bypassed or downgraded: on clean source candidate
`1a67f50200779e43128db617a4a5c516c8c9980d`, both parsers reran seeds 1/7/99 with
1,000 cases each, plus seed-1 300-case selfchecks and budget-128 minimizers. All
6,000 normal inputs were attempted with zero divergences; 26 host limits and 16
original timeouts remain separate from comparisons. Both canaries retain actual
successful-value minimized mismatches (108 PLY, 194 STL). Eight logs, raw reduced
reproductions and current dependency hashes are in
`evidence/verification-fuzz-2026-11-04-number-text/report.json`; the entire older
proof remains intact. The four parser units retain their prior fuzzed grade.

One attempted fast-gate command used the wrong working directory and could not
find the xtask alias; it is not gate evidence. The first correctly scoped full
fast gate caught three long float test literals; replacing them by the same exact
bit patterns resolved the lint without changing any test values/cases.

## Clean candidate validation

On the complete immutable candidate above, cargo xtask ci and cargo xtask oracle
passed. All 61 mandatory suites / 33,754 declared case identities remain, including
all prior refusals and every earlier seed/tolerance/gate. ci_report.py evidence
validated all raw candidate/reference/case inventories. Wasm probe, complete browser
contract tests and real Chrome passed all 4,431 previous comparisons plus 36 negative
protocol checks. These remain library probes, not browser/session acceptance.

Full raw passing Number inputs/original/native/stderr, source/generator hashes,
provenance and both actual red reproductions are archived in
`evidence/2026-11-04-number-shortest.json.gz`. Final docs/metadata point to this tested
implementation. Hosted checks apply to the final PR head once its bounded PR opens.

The formatter is fixed-corpus verified, not accepted/fuzzed/coverage-reviewed.
M3 still needs the actual scan model and prepared commit integration, complete
worker/render/import/plane commands, undo/apply, compatible projects and native CLI.
No original TypeScript, dependency, pin, numerical tolerance, existing case/seed or
1500 ms threshold changed. No main push, PR merge or legal/upstream action occurred.

| Artifact | SHA-256 |
| --- | --- |
| `number-string-before-fix-red.log` | `cc299bde92543f32aa958b281f314efe73778a00c43310c0013149c086e4b3dc` |
| `number-string-canary-red.log` | `a7170f52abdab64d514af08847ba12e7492c3dc6a6a13508b78f83e42aee5329` |
| `number-string-canary-exit.txt` | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |
| `number-string-final-committed-ci.log` | `504ac92ab030f82d0fe3ad7cee4c08df46f15a7f3fa82e2a31d18a7b343006b2` |
| `number-string-final-committed-oracle.log` | `4b90aa9b7745ca32318e28db2848d4ec2161dfd6d3163af7fb1200eab234cd94` |
| `number-string-final-committed-evidence.log` | `048c7e4e493245b603fbf660114d58ce3781c35d7ca61b5088a8bc7a696de631` |
| `number-string-final-committed-wasm.log` | `2619f92e1169db2884e50d65f4c2983a88e4feeca87dea3cf2f92c0b72dd818f` |
| `number-string-final-committed-browser-contract.log` | `a70328dc4ba61d049c260699d037b9ef2cbac861590e2e5866ce8aed0a1cd651` |
| `number-string-final-committed-browser.log` | `04f3ed167098bd605f50bfa105f1578e8ecbf819ca760e1a64874cc4bc13bd09` |
| `2026-11-04-number-shortest.json.gz` | `7fc9962ae3518a5a0ccc2318e23487039659a46b94c5fd2176c52095d02a3497` |
