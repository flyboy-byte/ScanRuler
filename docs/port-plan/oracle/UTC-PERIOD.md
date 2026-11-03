# Negative Gregorian period regression

2026-11-03. PR #74 expands #68's fixed UTC corpus from311 to1911 cases.
The added1600 independent inputs cover January1 and March1, at the boundary and
1ms before it, for every year-400..-1. This exercises all negative leap/century
boundaries and era rollover over a complete Gregorian period. Original Date
results were captured before expanding the Rust comparison's case admission.

Production date/page code is unchanged. All311 earlier input/original/Rust
records remain byte-exact, including refusals and wide seeded values. The old
source and paired archives are retained; cases-311.tsv retains historical inputs
and generate() retains its311-input behavior. generate_period() appends only the
new authored census. Tests check full states, hashes, independent generation and
the exact prior prefix. No expected value is regenerated from Rust.

All1911 complete direct timestamp plus six page/error/clock states match the
real original Date module. Full new source-only and paired raw artifacts are
retained. The original private module proof remains unchanged and required.
Native roster on this stack:36 suites /5,802 identities. The input expansion is
a fixed census, not fuzz/coverage review or actual browser runtime acceptance.
OS clock acquisition, browser wiring and full session execution remain open.
Run cargo xtask ci and cargo xtask oracle from rust/.
