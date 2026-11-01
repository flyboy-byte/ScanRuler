# Port PR audit queue

Updated 2026-11-01. Nothing here authorizes merging. Review source behavior against frozen 2cd89c6; do not relax original math or comparison expectations.

| Order | PR | Scope | Review evidence |
| --- | --- | --- | --- |
| independent | #1 | Stale handoff/CI guidance | Plan checks; documentation only |
| 1 | #2 | Binary STL helper | evidence/2026-11-01-binary-stl.md; ten successful native cases |
| 2 | #3 (base #2 branch) | Full STL dispatch/ASCII | evidence/2026-11-01-stl-dispatch.md; 82 cases; hosted Node 22 passed |
| 3 | welding follow-on (base #3 branch) | Exact-bit welding/filtering | evidence/2026-11-01-weld.md; 15 native cases |

Audit/merge in dependency order. After a parent merges, retarget the child to main and inspect the resulting diff before review; preserve each unit's tested SHA/evidence. Never merge a child against an unmerged parent branch as a substitute for landing on main. Regenerate backlog.json from canonical units when resolving generated-index conflicts; preserve independent P39 documentation updates from #1.

For each code PR: cargo xtask ci plus its explicit ignored oracle test(s), and inspect raw paired outputs. Node is oracle-only, system Rust on Logan's host, no rustup. Browser-Wasm runtime and P36 general harness remain missing; units stay implemented, not accepted. No new dependencies or upstream source changes.

Known separate original-app CI issue: reNetworkCreases.test.ts:209 exceeds its unchanged 1500 ms timing threshold (1516.53 ms, then 1894.01 ms on rerun). Do not hide the failure or increase its threshold to merge these ports. See STL dispatch evidence for run links. All original source/tests remain unchanged.
