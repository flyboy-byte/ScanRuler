# Decision: 2026-10-31-node-is-oracle-only

- Affected units, contracts and active contributions: P00, P01, P02, P03, P36-P38 (harness and CI).
- Existing plan/behavior: HANDOFF step 3 and VALIDATION section 1 require Node 22 for the upstream baseline.
- Proposed change and reason: no Node is needed to write or build the Rust port. Node is needed only to run the original TypeScript as the reference oracle (P01 baseline, P02 catalogs, differential comparisons). Install it when the first original-versus-Rust comparison is wanted, not before.
- Evidence, reproduction and tested commits: the workspace builds and passes `cargo xtask ci` with Node 20.20.2 on PATH and never invoking it. Upstream `package.json` says `engines.node >= 22`; whether the oracle runs on Node 20 is untested. Do not claim a baseline from Node 20 without recording it.
- Alternatives and compatibility impact: `fnm` or a distribution `nodejs-lts` package under the user's home keeps the system Node unchanged. At the end state, PURE-RUST.md permits the frozen JS only as a reference oracle.
- Required tests/migrations and follow-up units: P00 must record the Node version actually used for each baseline run.
- Owner/reviewer decision and date: Logan, 2026-10-31 ("unsure why we need node"); agreed after reading the plan.
