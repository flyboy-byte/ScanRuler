<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Prepared scan render continuation — 2026-11-04

Independent main-based implementation candidate: `7f2a6de0136e28bf1567bfda56ddb5b12f6d7287`. Reference
`2cd89c6769a10c27ee7754b477ada521c9ef5cb7` remains app 0.9.2. No shared-file PR is open for this branch
while #100 owns those files.

The complete prepared-array bodies now port wireSlots/wireConflicts, face normals,
CSR vertex-to-triangle fans, doubled fan-edge coplanarShare, tessellation decision,
creaseBudget, two-pass splitCreases, creaseFor and scanRender. Fan traversal and
union-find root/copy order match the source, including repeated corners and
non-manifold edges. Each side's area-weighted normals use the existing faithful
normal body. Skipped rendering copies the source graph arrays; completed splits
retain the logical vertex prefix and triangle order and append source-ordered
positions, normals, corner slots and copyOf. The default source budget remains
max(250000, vertexCount). Native input/allocation policy is a distinct typed error,
not a source budget report. No dependency or foreign algorithm was added.

The frozen oracle loads complete original modules using git show and TypeScript
compiler type erasure; no body is sliced, behavior stubbed or output fed into the
native generator. Before implementation it captured 1,056 full controls. Those
rows remain unchanged; 252 append K9 slot exhaustion and empty/isolated arrays,
and the last exercises an actual default-budget refusal on 40,000 disconnected
tetrahedra / 160,000 vertices. Other controls cover cubes, flat/folded quads, odd
and high-valence fans, degenerate/repeated corners, non-manifold faces, noisy
seeded grids, on/off/auto, seven angles and four explicit split budgets. Each side
constructs independent f32 geometry/indices, adjacency and normals from controls.

All 1,309 complete output buffers, slot/conflict counts, exact flat-edge ratios,
copy identities and reports match byte for byte. An actual native output bit
change produces 1,309 mismatches and exit 101, retaining all raw passing/red
input/original/native/stderr. Per-case wrong/missing-output checks remain. Native
admission tests distinguish invalid CSR/indices and output caps from source copy
budget refusal. The first lint/compile invocations caught missing public docs,
Rust float-literal syntax and one misplaced admission guard; those full logs are
retained as failed invocations, not comparison evidence. No positive numerical
mismatch was hidden or fixed by changing a control or tolerance.

Because the raw inventory selects the latest run, the complete positive render
corpus was rerun after the canary, keeping both full runs. The final manifest
therefore points to positive source/native values, not merely complete red files.

On the clean candidate, cargo xtask ci and cargo xtask oracle passed all 60 suites /
26,231 declared identities. This is an independent branch from main #99, not the
combined formatter/model inventory. Every inherited suite, case, seed, tolerance
and refusal remains. The exact raw inventory passed. Wasm probe, complete catalog/
browser negative contracts and real Chrome passed all 4,431 existing comparisons
and 36 negative protocol checks. Wasm compiles this render code; existing Chrome
probes remain libraries and do not claim actual render/viewer execution. Hosted
checks are still required on the final public PR head when it opens.

Full passing and deliberately wrong-value controls plus hashes/module identities
are in `evidence/2026-11-04-scan-render.json.gz`. Original-first source output and
failed development invocations are preserved separately with hashes. No accepted,
fuzzed or coverage-reviewed label is inferred.

The next source continuation is worker postScan plus versioned preparation/import
binding into the real session model. UI/command unknown setting-value coercion is
an explicit caller-boundary continuation, not claimed by typed prepared-array
functions. The pipeline must retain default crease-on behavior. Real import/plane
commands, global history apply/undo, project-compatible read/write and the native
CLI remain open. M3 still cannot be run.

Review after #100 and the formatter branch; this branch is independently based on
main, so it does not make a four-deep stack. Keep the model as the last session PR
with its single session CHANGELOG entry. When combining, retain this tested unit
record, merge its source with the model document and regenerate status/backlog via
tools. No main push/merge, original TypeScript edit, pin/dependency/threshold change
or numerical seed/case/tolerance reduction occurred.

| Artifact | SHA-256 |
| --- | --- |
| `scan-render-committed-ci.log` | `ba5ce1502d092ef8111b1ec7b33afde4f114dd1ea7f44d9c1fafdfc87930b051` |
| `scan-render-committed-oracle.log` | `a8858579d0c4937baf2ee7eb66518977630184a5949ddc238d2b213609e327c5` |
| `scan-render-committed-evidence.log` | `7851d040079e72633c5fcbcc4fe4c5deb3c86fcd691042de931b2aef3de71259` |
| `scan-render-committed-wasm.log` | `db07a3d0b3063c18f57be252f3df2daf79765acad46fd02d7f4cc37eecfaaa87` |
| `scan-render-committed-contracts.log` | `481b904e93b2b6ee7c44ad99c5e662e895c3e6b8882848419e094ba89a8ddb10` |
| `scan-render-committed-browser.log` | `04f3ed167098bd605f50bfa105f1578e8ecbf819ca760e1a64874cc4bc13bd09` |
| `scan-render-full-canary-red.log` | `c0f1f5daf3951989dff733c22099c6f662d921601400bd9c4b447fd63fc6bf1b` |
| `scan-render-post-canary-green.log` | `6e7d396e09570b56dfff79d8717f40b9974b4a5bdb3e898da9175248d921caf2` |
| `2026-11-04-scan-render.json.gz` | `c5e891b029cf56f9479cd18cfebaffcfe8caf874215c446a079edff91473b551` |
