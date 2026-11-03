# Plan source-proof tests need the frozen Git tree

2026-11-02. Hosted plan runs 41583681498 (#59) and 41584561973 (#60)
failed because actions/checkout defaults to depth1. The source-map proof tests
correctly require original source blobs at pinned 2cd89c6; that tree is absent
in the shallow checkout. The Rust validation workflow already fetches full history.

Reproduced with a fresh depth1 local clone of codex/stale-detect: git show of
2cd89c6:src/commands/types.ts fails. After git fetch --unshallow, the same
command succeeds. The fix sets fetch-depth:0 in plan-integrity.yml, preserving
credential persistence=false and the pinned checkout action. No assertion,
source map, case, timing threshold or parity tolerance changes.

This is CI checkout preparation, not a numerical or runtime parity finding.
Full generated-plan checks and Python tests validate the prepared checkout.
