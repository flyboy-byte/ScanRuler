# Port PR audit and integration queue

Updated 2026-11-02. **Historical: #1–#24 are all merged into `main` (`0940fd2`) and no PR is open.** The table records the base and hosted outcome each PR had when merged; for new PRs use `gh pr list -R flyboy-byte/scanruler` and AGENT-OPERATIONS.md. (Original text follows.) The owner will review from #1 upward. No row authorizes a merge. Current implementation state is generated in [STATUS.md](STATUS.md); the latest concrete next work is in [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md).

The immutable [observed CI snapshot](evidence/2026-11-01-review-ci.json) records exact heads, run links and capture time for #1–#21. It is not a live badge. Earlier attempt/run history is retained in [the prior queue](evidence/2026-11-01-audit-queue-before22.md).

| Review | Scope | Current PR base | Observed Rust / original CI |
| --- | --- | --- | --- |
| [#1](https://github.com/flyboy-byte/ScanRuler/pull/1) | Handoff docs | `main` | SUCCESS / SUCCESS |
| [#2](https://github.com/flyboy-byte/ScanRuler/pull/2) | Binary STL | `main` | SUCCESS / FAILURE |
| [#3](https://github.com/flyboy-byte/ScanRuler/pull/3) | Complete STL dispatch | `codex/port-binary-stl` | SUCCESS / SUCCESS |
| [#4](https://github.com/flyboy-byte/ScanRuler/pull/4) | Mesh welding | `codex/port-stl-dispatch` | SUCCESS / SUCCESS |
| [#5](https://github.com/flyboy-byte/ScanRuler/pull/5) | Vertex normals | `codex/port-mesh-weld` | SUCCESS / FAILURE |
| [#6](https://github.com/flyboy-byte/ScanRuler/pull/6) | Winding orientation | `codex/port-vertex-normals` | SUCCESS / FAILURE |
| [#7](https://github.com/flyboy-byte/ScanRuler/pull/7) | Dense solves | `codex/port-mesh-orientation` | SUCCESS / SUCCESS |
| [#8](https://github.com/flyboy-byte/ScanRuler/pull/8) | TLS planes | `codex/port-linear-solvers` | SUCCESS / SUCCESS |
| [#9](https://github.com/flyboy-byte/ScanRuler/pull/9) | Clipped plane continuation | `codex/port-plane-tls` | SUCCESS / FAILURE |
| [#10](https://github.com/flyboy-byte/ScanRuler/pull/10) | Bounds/shared primitives | `codex/port-plane-clipping` | SUCCESS / SUCCESS |
| [#11](https://github.com/flyboy-byte/ScanRuler/pull/11) | Mesh centroids | `codex/port-mesh-bounds` | SUCCESS / SUCCESS |
| [#12](https://github.com/flyboy-byte/ScanRuler/pull/12) | Robust planes/PRNG | `codex/port-mesh-centroid` | SUCCESS / FAILURE |
| [#13](https://github.com/flyboy-byte/ScanRuler/pull/13) | Original headless catalog | `main` | SUCCESS / SUCCESS |
| [#14](https://github.com/flyboy-byte/ScanRuler/pull/14) | Original browser catalog | `codex/capture-headless-catalog` | SUCCESS / FAILURE |
| [#15](https://github.com/flyboy-byte/ScanRuler/pull/15) | Native oracle discovery gate | `codex/port-ransac-plane` | SUCCESS / SUCCESS |
| [#16](https://github.com/flyboy-byte/ScanRuler/pull/16) | Curvature/RMS | `codex/native-oracle-gate` | SUCCESS / SUCCESS |
| [#17](https://github.com/flyboy-byte/ScanRuler/pull/17) | Ordered adjacency | `codex/port-curvature-gate` | SUCCESS / SUCCESS |
| [#18](https://github.com/flyboy-byte/ScanRuler/pull/18) | Directed components | `codex/port-mesh-adjacency` | SUCCESS / FAILURE |
| [#19](https://github.com/flyboy-byte/ScanRuler/pull/19) | Mesh selection preservation | `codex/port-mesh-components` | SUCCESS / FAILURE |
| [#20](https://github.com/flyboy-byte/ScanRuler/pull/20) | Complete mesh graph | `codex/port-mesh-selection` | SUCCESS / SUCCESS |
| [#21](https://github.com/flyboy-byte/ScanRuler/pull/21) | Closed-shell parity | `codex/port-mesh-graph` | SUCCESS / SUCCESS |
| [#22](https://github.com/flyboy-byte/ScanRuler/pull/22) | CI/status/docs and remaining implementation plan | #21 plus local joins of #1/#13/#14 | Full hosted gates passed; see continuation below |
| [#23](https://github.com/flyboy-byte/ScanRuler/pull/23) | Complete marked-selection plane element | #22 | 51 source cases; check current head |
| [#24](https://github.com/flyboy-byte/ScanRuler/pull/24) | Actual Rust geometry in Chrome Wasm | #23 | Hosted 114-case runtime / strict gate passed; immutable report retained |

## Merge order and evidence

Review #1–#21 in number order. #13 targets main and #14 targets #13; #15 resumes the numerical chain at #12. After each parent lands, retarget the child to main and inspect the resulting diff. Do not merge a child into an unmerged parent instead of trunk. #22 is the integration checkpoint joining all three histories; after prior merges its diff should contain only the CI/docs/status changes. No earlier PR is force-updated by this integration.

Every numerical unit JSON links its actual implementation SHA and report/case hashes; follow STATUS.md to those units. Run `cd rust && cargo xtask ci && cargo xtask oracle`, then the exact-run raw inventory and affected original/browser checks. Preserve fixed tolerances/seeds/outputs. Current metadata/raw checks do not provide the missing P37 exact-base numerical ratchet. No numerical child is accepted.

The original source unit timing test reNetworkCreases.test.ts:209 has an unchanged 1500 ms threshold. Known #18/#19 failures were verified as 1729.82/1753.32 ms (runs 41365788786/41366118457); #16/#17/#20/#21 full original CI passed at the captured heads. Older failures and raw links remain in the previous queue. Do not suppress the test or raise its threshold. #22 collects MCP/browser results independently while the strict original aggregate remains red on any failure.

For #22, validate all new lanes and negative tests, generated inventory freshness, exact SHA raw evidence, original snapshot recapture and successful/failing source diagnostics. Local system Rust policy remains; Node 22 is oracle-only. No upstream source/tests/locks, deploy, legal or repository settings are changed.

Checkpoint 2026-11-01: #22 final hosted Rust gate 41370201083, both original catalog recaptures 41370201138/41370201212 and every independent original lane/aggregate 41370201074 passed. The CI separation is exercised on hosted Node 22. #23 adds fit.plane.selection: 45 successful full elements, six source refusals, original-region retention after clipping, and fixed angle probes. Current generated inventory is eighteen suites / 686 case identities including refusals. No numerical child accepted and no PR merged.

Runtime checkpoint 2026-11-01: #24 actual Chrome-Wasm and strict Rust gate passed run 41375102435 at CI merge 00fd1a69af98c7bacc91da747483ac0ccbfb70c6 (head 43e3977). TLS/mesh graph/marked-plane inputs: 114, including 90 successful value cases and 24 source refusals; ten bounded protocol checks. Node 22.23.4, Chrome 156.0.8078.47, Rust 1.99.1. Full report/module/case hashes under evidence/browser-wasm-2026-11-01/. No imports, page errors, source-fed Rust outputs or numerical budget changes. All numerical units remain unaccepted; broader runtime, commands and P37 remain open.
