# Port PR audit and integration queue

2026-11-02: [#51](https://github.com/flyboy-byte/ScanRuler/pull/51) adds opt-in bounded PLY/STL reproducer reduction on main522f10c. Both real sabotage checks shrink inputs while preserving mismatches; seed7 real runs have zero divergences (PLY retains two source timeouts). Fast gate passes at32c45a9. See oracle/FUZZ-MINIMIZE.md and the lossless evidence archive.

2026-11-02 review continuation: [#45](https://github.com/flyboy-byte/ScanRuler/pull/45)
then [#46](https://github.com/flyboy-byte/ScanRuler/pull/46). #46 adds bounded
identity-aware history recording/group/clear and sync/async wrapper behavior,
with 110 exact full original/Rust scenarios and 1,095 observations. Native
inventory is 28 suites / 2,475 case identities. The parent session contract
remains specified. Undo/redo restoration, scheduler, actual commands and CLI
workflow are the next work; neither PR completes M3.

Session continuation from main4ff736f: [#45](https://github.com/flyboy-byte/ScanRuler/pull/45)
ports complete registration/catalog/headless and schema admission, with350 exact
full operation scenarios. Parent engine.session.contract is now owned by Codex
and remains specified; history/scheduling/real feature commands stay open.
Earlier #42–#44 were merged by the owner, all11 checks green on each.

Current review order: #42 → #43 → [#44](https://github.com/flyboy-byte/ScanRuler/pull/44).
#44 retains comparison-only LLVM coverage across all26 suites/2015 identities
at e2c3d00, with report-only correction be43ab8. Every explicit zero DA line
and unresolved LLVM metric discrepancy remains visible; no branch/acceptance
claim. The full original/ Rust value comparisons and native/browser inventories
are unchanged. Session implementation remains with the owner’s other agent.

Coverage continuation: review #42 → [#43](https://github.com/flyboy-byte/ScanRuler/pull/43).
#43 archives1422 exact instrumented original replays, eight complete module maps
and every unreached/unknown location. It changes no Rust bodies; full native
checkpointfa711af remains the input/result authority. Independent fuzz and
static condition coverage remain open.

Continuation 2026-11-02 from main `3d9b482` after owner integration through #41:
[#42](https://github.com/flyboy-byte/ScanRuler/pull/42) ports `engine.schema.validate`
with 697 complete native observations (26 suites /2015 identities). Session
contract ownership remains with the owner’s other agent. Earlier review-stack
entries below are historical; this continuation does not merge PRs.

Final continuation checkpoint 2026-11-02: review **#36 → #37 → #39 → #40**.
Latest #36 `ac02df4`, #37 `7dd1ace`, #39 `8bb0ae0` have successful Rust, plan,
license, original source/MCP/browser and aggregate checks; CLA is skipped, not
signed. [Exact heads and check links](evidence/2026-11-02-review-stack.json).
#40 passed the complete local 25-suite /1,318-identity source gate and raw
inventory at `a9f8826`, plus fast gate and existing browser proof regressions.
Its final hosted checks are still independent/pending. No PR was merged into
main by this continuation. Earlier source timing failure and harness attempt
remain documented; they were not hidden by dropping comparisons or thresholds.

[#40](https://github.com/flyboy-byte/ScanRuler/pull/40) continues after #39 with
complete bounded async drop traversal and 40 exact source traces. Review order
is #36 → #37 → #39 → #40; actual DOM adapter runtime remains a follow-up.

Current continuation queue: #36 → #37 → [#39](https://github.com/flyboy-byte/ScanRuler/pull/39).
#39 is the exact dependency/source-pixel audit, with direct decoder adoption
rejected and no application dependencies changed. Main’s #38 verification docs
are merged into both parents and this branch; derived files were regenerated.

Retained #36 hosted observation at head `8b3a208`: Rust/native/browser/plan/license,
MCP and original browser succeeded. The original source lane’s only failure was
reNetworkCreases.test.ts:209: 1517.556736 ms against 1500 ms. Read the full job log;
[exact observation](evidence/2026-11-02-pr36-original-timing.json) retains the
run/job/merge SHA. This was not waived, threshold-adjusted or manually retried.
New parent-integration heads trigger independent CI; inspect those final heads.

Continuation 2026-11-02 from clean main `cb0b480` after #35: new queue
[#36](https://github.com/flyboy-byte/ScanRuler/pull/36), complete companion file
selection/identity/byte state. Local 87-case source evidence is committed; inspect
final hosted results independently. Earlier numbered rows below are history.
[#37](https://github.com/flyboy-byte/ScanRuler/pull/37) stacks actual Chrome
file-state execution on #36:651 comparisons,24 protocol checks and complete
raw proof. #36 hosted Rust/plan/license/MCP checks passed at observed head `8b3a208`;
original source/browser were still running at that observation. Inspect final
heads independently. No PR is merged by this continuation.

Updated 2026-11-02 UTC. PRs #1–#34 are merged into `main` (`28dfeae`) and none is open. #26–#30
(baseline, PLY, error envelope), #31/#32/#34 (OBJ and its Chrome comparison) and #33 (the PLY
negative-count text fix from an independent 282,000-file fuzz) were merged bottom-up with merge commits
after reading the failing logs: the only red check was the known `reNetworkCreases.test.ts:209` timing
test on #26, #27, #30 and #31. The original tree had no diff against `reference/2cd89c6`. No line-by-line
review of the added Rust was done; the vetting was mechanical checks, hosted evidence and the fuzz.
Local baseline unit failures are preserved in `evidence/baseline-2026-11-02/`. Current implementation
state is generated in [STATUS.md](STATUS.md). Open audit findings are in `plan/findings/`.

The table below is the **historical first-batch checkpoint**, not current branch
bases or a live badge. Its earlier “no PR open/no PR merged” statements describe
that checkpoint. For live claims use `gh pr list -R flyboy-byte/scanruler` and
ROADMAP-TO-USABLE.md. No row authorizes a merge.

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
