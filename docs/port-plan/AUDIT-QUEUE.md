# Port PR audit queue

Updated 2026-11-01. Nothing here authorizes merging. Review source behavior against frozen 2cd89c6; do not relax original math or comparison expectations.

| Order | PR | Scope | Review evidence |
| --- | --- | --- | --- |
| independent | #1 | Stale handoff/CI guidance | Plan checks; documentation only |
| 1 | #2 | Binary STL helper | evidence/2026-11-01-binary-stl.md; ten successful native cases |
| 2 | #3 (base #2 branch) | Full STL dispatch/ASCII | evidence/2026-11-01-stl-dispatch.md; 82 cases; hosted Node 22 passed |
| 4 | #5 (base #4 branch) | Area-weighted normals | evidence/2026-11-01-normals.md; ten native cases |
| 3 | #4 (base #3 branch) | Exact-bit welding/filtering | evidence/2026-11-01-weld.md; 15 native cases |

| 5 | #6 (base #5 branch) | Outward normal/winding correction | evidence/2026-11-01-orientation.md; 15 state comparisons |

| 6 | #7 (base #6 branch) | Cholesky and elimination | evidence/2026-11-01-linear.md; 22 value/state cases |

| 7 | plane TLS follow-on (base #7 branch) | Fixed-selection plane fitting | evidence/2026-11-01-plane.md; 13 paired cases |

Audit/merge in dependency order. After a parent merges, retarget the child to main and inspect the resulting diff before review; preserve each unit's tested SHA/evidence. Never merge a child against an unmerged parent branch as a substitute for landing on main. Regenerate backlog.json from canonical units when resolving generated-index conflicts; preserve independent P39 documentation updates from #1.

For each code PR: cargo xtask ci plus its explicit ignored oracle test(s), and inspect raw paired outputs. Node is oracle-only, system Rust on Logan's host, no rustup. Browser-Wasm runtime and P36 general harness remain missing; units stay implemented, not accepted. No new dependencies or upstream source changes.

Known separate original-app CI issue: reNetworkCreases.test.ts:209 exceeds its unchanged 1500 ms timing threshold (1516.53 ms, then 1894.01 ms on rerun). Do not hide the failure or increase its threshold to merge these ports. See STL dispatch evidence for run links. All original source/tests remain unchanged.

Hosted Rust gates observed passing: #4 run 41347942247; #5 run 41348126410. Both include Node 22 native oracle tests; this does not imply browser-Wasm runtime or original-app CI passed.
