# Source unit worker scheduling experiment

Pre-registered 2026-11-02 before the new hosted run. Existing source runs
41588180933 (PR55),41585589657 (PR61) and41585626487 (PR63) failed only the
unchanged reNetworkCreases.test.ts:209 wall-time assertion (1500ms). PR61's one
rerun failed again; PR63's passed. This is evidence of intermittent timing,
not a demonstrated numerical divergence or proof of the contention hypothesis.

Hypothesis: concurrent source test files compete for the runner's CPU during
this timing assertion. A2 schedules the entire unchanged Vitest suite using
maxWorkers=1,minWorkers=1,fileParallelism=false. Installed pinned Vitest CLI
help confirms all three flags. No source/config/lock/test input changes, test
filters, assertion changes, retries or extra timeout allowance are introduced.
The source-tests job retains its20-minute budget; MCP/browser and Rust lanes
stay independent and mandatory. This is one fixed scheduling configuration,
not a search over worker counts after seeing results.

Exit conditions fixed in advance: two independent complete hosted source-suite
runs at the same candidate/config must pass all384 discovered files (371pass,
13existing skips when private scans are absent), retain the1500ms assertion and
finish inside20minutes. All other required PR checks must also pass. If a new
failure or the same timing assertion persists, retain the diagnostics and leave
the experiment unpromoted. Two passing runs do not establish a statistical cure;
future failures stay red. Results will be recorded without changing these gates.

Results 2026-11-02: A2 met the registered gate at643bd5b. Independent PR run
41589811086/job125763689911 and workflow-dispatch run41589910621/job125764015730
both passed371 files/13existing skips (384total), inside the unchanged20-minute
job limit. Both ran the exact registered worker flags and same candidate. Full
job logs, workflow bytes, source-test hash and metadata are retained under
evidence/source-scheduling-2026-11-02 with a verified report. The artifact
regression checks hashes, actual CLI flags/counts, immutable1500ms source
assertion, both event identities and duration bounds. No test body changed.

Verdict: the scheduling candidate meets the pre-registered promotion conditions.
Two successes provide limited reliability evidence; the cause of the old flake
is unproven and future failures remain red. Source numerical parity is not
claimed from scheduling evidence. Final PR checks remain mandatory.
