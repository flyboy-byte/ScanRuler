# Contributing to the Rust port

Help is welcome with source/contract analysis, harnesses, fixtures, geometry ports, native/Wasm UI, formats, documentation and regression coverage. We aim to preserve the pinned original ScanRuler behavior in a pure-Rust implementation.

Read `docs/port-plan/PORTING-PROCESS.md` for the complete process. Pick a bounded unclaimed operation or follow-up, reproduce its gap, and open a scoped draft PR. State unit IDs, current base SHA, source symbols, intended files/interfaces, related PRs and how success will be measured. Check both task IDs and files/symbols for overlap.

Implement according to the unit and architecture; refine steps/open items as evidence develops. Source code, fixtures and guides retain their required attribution/licenses. The pending licensing enquiry is not a grant of new rights, and no contributor CLA is assumed here.

Before requesting review:

- Compare the original and Rust on the same independently supplied inputs, using the pinned baseline and fixed cases/seeds.
- Include successful geometry/state measurements, relevant refusals and boundary inputs; preserve all prior comparisons.
- Run the defined local/CI gate, native/Wasm checks and affected callers. Add a minimized fixture for each fixed divergence.
- Update the unit's evidence/open items and identify follow-up work. Use actual screenshots for visible behavior.
- Record both tested SHAs, reference/fixture/comparator identities and known skips or blockers. Missing required evidence means partial work, even when compilation passes.

Run `cd rust && cargo xtask ci && cargo xtask oracle`, plus affected original/browser checks. The fast gate validates generated backlog/status as well as tests/lints and Wasm compilation. The native oracle gate executes all registered source suites; browser-Wasm runtime and generic `diff`/`diff-ci`/exact-base ratchet remain open. See [CI contracts](docs/port-plan/CI-AND-REGRESSIONS.md) and [current inventory](docs/port-plan/STATUS.md).

After canonical edits, regenerate both `backlog.json` (`port_plan.py --root docs/port-plan render`) and status (`ci_report.py status`). Do not edit generated counts by hand. Maintainers review and merge contributor PRs; each approval is for a particular commit. Preserve partial/open scope and historical evidence. The [implementation plan](docs/port-plan/IMPLEMENTATION-PLAN.md) names bounded next work and exit evidence.
