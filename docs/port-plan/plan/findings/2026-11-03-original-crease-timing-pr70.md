# Original crease timing failure on guard PR

2026-11-03. PR70 original run41592185438/job125771201971 failed only
reNetworkCreases.test.ts:209 at1578.9365049999997ms vs1500ms;370 files passed,
13skipped. All Rust/plan/browser/MCP/license lanes passed. One failed-lane rerun
was requested; no further retries are implied. The PR body retains the failure
link. Source tests and thresholds are unchanged. The independently reviewed #69
single-worker scheduling change is not yet in this branch and is not treated
as a waiver of the red timing check.
