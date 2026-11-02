<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust differential coverage targets — 2026-11-02

MEASURED: all26 native original/Rust suites /2015 identities pass under isolated
LLVM instrumentation at e2c3d00. Full raw inventory is exact-candidate complete.
No ordinary host/unit test was executed in this profile campaign. Report-only
correction at be43ab8 preserves the initial export/parser failure and both raw
representations. [Full generated locations](../../evidence/rust-coverage-2026-11-02/report.json).

- 35 owned src files are inventoried;32 have explicit emitted DA records. Three
 lib.rs files contain module declarations with no emitted executable line records.
 They are unknown/non-emitted in this metric, not100% covered functions.
- 398 explicit DA line records are zero-count. These include allocation/host
 refusal guards, unexercised port paths, unused scaffolding helpers and inline
 ordinary test bodies. Every owning implemented unit has a coverage open item;
 do not erase zero lines because ordinary tests happen to cover them elsewhere.
 Source-profile and ordinary-test evidence answer different questions.
- schema.rs has58 explicit zero-count lines; schema_value.rs has1. Inspect those
 paths alongside the source coverage targets before claiming exhaustive validator
 verification. Body implementation and697 successful full comparisons do not
 close all branch/host paths.
- ids.rs,transform.rs,units.rs and the three module files have no canonical
 implemented function-owner mapping. They are retained as unowned scaffolding
 records. Session integration is still reserved to the owner's other agent;
 this diagnostic does not claim those helpers have comparison contracts.
- Seven src files contain inline cfg(test) sections. Their markers and DA line
 records are explicit. No production-only percentage is inferred from these
 whole-file exports; closed_shells includes its actual differential harness there.
- 29 owned files have DA-versus-LF/LH summary discrepancies. Keep both raw metrics
 and their independent JSON-summary cross-check; root cause/interpretation is
 still UNKNOWN. See [decision](../decisions/2026-11-02-llvm-line-records.md).
- No branch records were emitted by this stable instrumentation setup. Branch
 reach remains unknown. Static branch/condition data, browser profiles,
 independent fuzz campaigns, immutable toolchain pinning and exact-base ratchet
 remain separate follow-ups. No acceptance threshold is introduced.

The report is generated from raw LLVM exports, not edited by hand. Existing
numeric comparators, source inputs, seeds, values/state comparisons and budgets
remain unchanged. Runtime errors and unexplained diagnostics stay visible.
