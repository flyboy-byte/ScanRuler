# CI and status consolidation — 2026-11-01

Owner requested the next PR rework CI/status/docs and update implementation plans for review from #1 upward. Preserve source/numerical expectations. Use independent fast/native/catalog and original unit/MCP/browser lanes with strict aggregate checks; main runs never cancel one another, superseded PR runs may cancel. Preserve existing aggregate IDs gate/test-and-build. Hash and require every current raw suite/case at exact checkout SHA before publishing evidence. Keep committed original catalog snapshots immutable and recapture on relevant PR/main changes.

Canonical JSON remains authoritative. Generate backlog plus human/machine status and fail CI on drift; generated counts are inventory, not acceptance. Python is temporary contributor bootstrap until P39, not product logic. Keep generic paired reports, exact-base ratchet, browser-Wasm execution and kernel purity unimplemented rather than aliasing them to weaker current tools. No numeric/seed/tolerance changes, repository settings, release activation or automatic merges.

Integration branch starts at #21 and joins #1/#13/#14 parent histories. #22 follows the owner's review/merge of #1–#21; retarget to main and inspect the final diff after parents land. Historical snapshots/reports are retained. No forced updates to earlier PR branches.
