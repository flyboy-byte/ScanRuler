# Port PR audit queue

Updated 2026-11-01. Nothing here authorizes merging. Review source behavior against frozen 2cd89c6; do not relax original math or comparison expectations.

| Order | PR | Scope | Review evidence |
| --- | --- | --- | --- |
| independent | #1 | Stale handoff/CI guidance | Plan checks; documentation only |
| 1 | #2 | Binary STL helper | evidence/2026-11-01-binary-stl.md; ten successful native cases |
| 2 | #3 (base #2 branch) | Full STL dispatch/ASCII | evidence/2026-11-01-stl-dispatch.md; 82 cases; hosted Node 22 passed |
| 3 | #4 (base #3 branch) | Exact-bit welding/filtering | evidence/2026-11-01-weld.md; 15 native cases |
| 4 | #5 (base #4 branch) | Area-weighted normals | evidence/2026-11-01-normals.md; ten native cases |
| 5 | #6 (base #5 branch) | Outward normal/winding correction | evidence/2026-11-01-orientation.md; 15 state comparisons |
| 6 | #7 (base #6 branch) | Cholesky and elimination | evidence/2026-11-01-linear.md; 22 value/state cases |
| 7 | #8 (base #7 branch) | Fixed-selection plane fitting | evidence/2026-11-01-plane.md; 13 paired cases |
| 8 | #9 (base #8 branch) | Shared clipping and clipped plane fits | evidence/2026-11-01-clip.md; 19 value/selection cases |
| 9 | #10 (base #9 branch) | Complete extents/bounds, shared helpers | evidence/2026-11-01-bounds.md; 12 cases + 106 core regressions |
| 10 | #11 (base #10 branch) | Area, surface/volume centroids and closure | evidence/2026-11-01-centroid.md; 23 paired cases |
| 11 | #12 (base #11 branch) | PRNG, median consensus and robust plane fit | evidence/2026-11-01-ransac.md; 50 draw/value/state cases |
| 12 | #15 (base #12 branch) | Discovery-guarded native oracle gate | evidence/2026-11-01-native-oracle-gate.md |
| 13 | #16 (base #15 branch) | Curved-fit versus plane gate and residual RMS | evidence/2026-11-01-curvature-gate.md; 80 exact value/state cases |
| 14 | #17 (base #16 branch) | Ordered vertex adjacency | evidence/2026-11-01-adjacency.md; 56 complete-array cases |
| 15 | #18 (base #17 branch) | Directed disjoint sets and mesh components | evidence/2026-11-01-components.md; 51 cases / 1557 complete state snapshots |
| 16 | #19 (base #18 branch) | Ordered mesh selection filtering/remapping | evidence/2026-11-01-mesh-selection.md; 62 full-output cases |
| 17 | #20 (base #19 branch) | Complete parsed mesh graph pipeline | evidence/2026-11-01-mesh-graph.md; 36 full states + 14 exact refusals |
| 18 | #21 (base #20 branch) | Bit-welded closed-shell parity | evidence/2026-11-01-closed-shells.md; 64 flag/private-ID/hash cases |
| independent | [#13](https://github.com/flyboy-byte/ScanRuler/pull/13) | Frozen original headless catalog | Actual built-engine core/all evidence; hosted capture passed |
| after #13 | [#14](https://github.com/flyboy-byte/ScanRuler/pull/14) | Browser catalog capture | Actual original Chrome core/all captured; evidence and immutable schema guards on #14 |

Audit/merge in dependency order. After a parent merges, retarget the child to main and inspect the resulting diff before review; preserve each unit's tested SHA/evidence. Never merge a child against an unmerged parent branch as a substitute for landing on main. Regenerate backlog.json from canonical units when resolving generated-index conflicts; preserve independent P39 documentation updates from #1.

For each code PR: cargo xtask ci plus its explicit ignored oracle test(s), and inspect raw paired outputs. Node is oracle-only, system Rust on Logan's host, no rustup. Browser-Wasm runtime and P36 general harness remain missing; units stay implemented, not accepted. No new dependencies or upstream source changes.

Known separate original-app CI issue: reNetworkCreases.test.ts:209 exceeds its unchanged 1500 ms timing threshold (1516.53 ms, then 1894.01 ms on rerun). Do not hide the failure or increase its threshold to merge these ports. See STL dispatch evidence for run links. All original source/tests remain unchanged.

Hosted Rust gates observed passing: #4 run 41347942247; #5 run 41348126410. Both include Node 22 native oracle tests; this does not imply browser-Wasm runtime or original-app CI passed.

Hosted Rust gates observed passing: #6 run 41348317934; #7 run 41348595278; #8 run 41348902435. Original-app CI remains a separate check; #6 run 41348317870 failed and must be inspected independently.

Run the current complete local audit gate from rust/:

```bash
cargo xtask ci
cargo xtask oracle
```

Oracle tests require Node and the frozen Git object. Inspect each raw target/*-diff run and its candidate identity, not only test counts. Shared-helper changes in #10 refresh plane/clipping/orientation evidence while retaining historical unit evidence. Mesh surface/volume centroid and closure behavior are now implemented in the follow-on unit; no engine/session or browser-Wasm runtime feature exists yet.

Hosted Rust gates observed passing: #9 run 41349387428; #10 run 41349751575. Targeted native source comparisons are distinct from unresolved original-app timing CI and browser-Wasm execution.

Hosted Rust gate observed passing: #11 run 41350575050. Confirmed #6 original-app failure is the same unchanged reNetworkCreases.test.ts:209 timing gate, 1996.27 ms vs 1500 ms (run 41348317870); no source expectation was relaxed.

Submission checkpoint 2026-11-01: #13 current-head checks all passed (original-app CI 41354547773 and headless capture 41354547806). #14 actual Chrome capture passed run 41356099428 after real original MCP pairing; snapshots committed and current recapture pending. #15 hosted Rust gate passed 41356040853 (all eleven native suites); original-app CI still running at this checkpoint. #12 Rust gate passed 41351565751; its separate original-app failure is confirmed reNetworkCreases.test.ts:209, 1872.05 ms vs 1500 ms (41351565764). #9 has the same failure at 1536.11 ms (41349387418). No thresholds changed and no merges performed.

Follow-on checkpoint: #14 final immutable browser recapture passed 41356255484; its separate original-app test fails the same reNetworkCreases.test.ts:209 threshold (1553.30 ms vs 1500 ms; 41356255460). #15 current-head Rust gate passed 41356313339 and complete original-app CI passed 41356313288. #16 adds 80 exact curvature/RMS cases to the discovery roster (twelve suites, 352 identities including refusals). No original source threshold changes.

#16 hosted Rust gate passed run 41365187936 (twelve source suites); original-app CI was pending at the checkpoint. #17 adds complete mesh adjacency, 56 exact array cases and a weld-to-adjacency integration check; local thirteen-suite gate passes, 408 case identities including refusals. Original-app CI must be checked independently.

#17 hosted Rust gate passed 41365449719; original-app CI pending at the checkpoint. #18 adds 51 exact parent/query cases (1557 complete state snapshots); all fourteen native suites pass locally, 459 case identities including refusals. PRs remain unmerged and units unaccepted.

#18 hosted Rust gate passed 41365788747; original-app CI pending at the checkpoint. #19 adds 62 exact selection cases; local fast and fifteen-suite source gates pass (521 case identities including refusals). Original source/tests remain unchanged and no PRs merged.

#19 hosted Rust gate passed 41366118494; original-app CI pending at the checkpoint. #20 integrates existing helpers without changes and passes the full local sixteen-suite gate (571 identities including refusals), plus 26 ordinary tests. Audit adjacency-before-winding order, source validation/error/progress and explicit host ownership/budgets. P06 still requires mapping/queries/runtime/session work.

#20 hosted Rust gate passed 41366668479; original-app CI pending at the checkpoint. #21 adds 64 exact closure/private-state cases; local fast and seventeen-suite source gates pass (635 identities including refusals). Inspect signed-zero distinction, ignored tails and original even-edge probabilistic semantics; no manifold guarantee or acceptance.
