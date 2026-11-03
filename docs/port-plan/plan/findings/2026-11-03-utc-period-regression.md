# UTC negative-year period regression claim

2026-11-03. Follow-up to #68, owned by Codex. Expand only the fixed input/source
comparison corpus across one complete negative-year400-year Gregorian period.
Retain the existing311 input/source/Rust records byte-for-byte. Capture the
additional original results before changing the live Rust comparison's case
admission. No production calendar, clipping, timeout or page-state code change.

The added inputs cover January1 and March1, at the boundary and1ms before it,
for years-400..-1. Input construction uses Python date arithmetic for1600..1999
and an exact five-period day offset, without reading either engine's answers.
The real original Date converter remains the oracle. This is a fixed census,
not a fuzzing, coverage-review, browser or whole-session acceptance claim.

Result: the additional1600 source/native states match; all prior311 case records
are byte-identical across input/original/Rust files and hashes. The live corpus
now has1911 identities. Production code and existing comparison budgets remain
unchanged. Full-gate and hosted results are recorded in the PR body/Checks.
