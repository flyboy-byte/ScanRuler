# Original local baseline: failures and exclusion-report discrepancy

DOCUMENTED — full default original run on 2026-11-02, Node 22.23.4; original source
unchanged against `2cd89c6`. See `evidence/baseline-2026-11-02/` for raw logs,
individual unit results, commands, identities and hashes.

Four failures and one unhandled worker timeout occurred: two 20 s test timeouts,
an over-budget shell bisection diagnostic assertion, and the 1,500 ms crease
timing assertion. The hosted original run at main `551b3a9` passed separately.
Do not collapse the four failures into the previously known crease-only failure,
change thresholds, or claim the local baseline is green.

Vitest's summary says 24 skipped; its individual JSON additionally has one nested
pending alignment/real-scan test. The inventory preserves all 25 excluded entries
and the original summary. Each entry matches a declared original guard. Future
unmapped exclusions or changed guards fail the baseline inventory checks.

UNKNOWN — whether another unchanged local run would pass, or how much scheduling
versus other environment factors explain the failures. No retries were used to
replace this observation. G0 and the full P03 corpus remain incomplete.
