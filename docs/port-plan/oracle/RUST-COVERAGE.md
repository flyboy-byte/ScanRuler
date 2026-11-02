<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust comparison-only LLVM coverage

Unit `harness.coverage.rust`, PR #44,2026-11-02. System Arch Rust/LLVM; no rustup,
no added package. A real instrumented program/profile merge/export smoke passed
with rustc1.99.1 (LLVM23.1.3), llvm-profdata/llvm-cov23.1.4.

From a clean committed repo root:

```
python3 docs/port-plan/tools/rust_coverage.py run
# The command prints its fresh rust/target/rust-coverage/run-N directory.
python3 docs/port-plan/tools/rust_coverage.py report --out rust/target/rust-coverage/run-N
```

The runner checks LLVM tool major compatibility, builds every test executable
with `-C instrument-coverage -C link-dead-code` into a separate target, then runs
**cargo xtask oracle**, all required comparison tests. Ordinary tests are never
executed in that profile directory. `%m-%p.profraw` avoids collisions; a new run
refuses a pre-existing directory. Inherited Rust flags are refused rather than
silently mixed. Git must remain clean and at the same commit until execution
finishes. No machine toolchain setup, third-party dependency or source edit.

The normal comparison assertions remain authoritative. Exact-candidate raw
inventory is validated after the instrumented run. All profiles merge with the
installed llvm-profdata; llvm-cov exports both complete JSON regions and LCOV
line counts. Report line/count sets come directly from LCOV DA records and are
retained separately from LF/LH totals; the observed multi-object export
disagrees on29 owned files, so both counts and a discrepancy flag remain
explicit. No denominator/percentage is inferred; see the LLVM line-record
decision. Source text hashes, owning implemented units,
unreached lines and uninstrumented lines are reported per owned production file.
Inline cfg(test) sections are marked separately because file-level exports
include unexecuted ordinary test bodies and the inline differential harness.
The source inventory includes files with no emitted records as explicitly
unknown. Comments/declarations/uninstantiated code are not inferred covered.

Stable region instrumentation does not emit branch records in this setup.
**Branch reach is unmeasured, never 100%.** Static branch instrumentation and
platform runtime remain open. Successful line coverage does not establish value,
state, branch, command or acceptance completeness. Unreached host guards should
be interpreted alongside their separate ordinary tests, not filled by blending
those tests into this comparison-only diagnostic. The system toolchain is
recorded; immutable compiler pinning remains the existing P00 gap.

The retained bundle contains compressed complete LLVM JSON/LCOV/source data,
run/tool/executable/profile identities, native case/input/output hashes and logs,
plus generated JSON/Markdown. Binary executables/profraw/profdata remain in target;
reproduce them with the command, not from regenerated expected values. The CI
fast gate replays committed line data and rejects malformed/changed evidence.
Independent fuzz, browser profiles, static branch data and exact-base ratchet
remain separate. No source formula, tolerance, seed or passing case changes.

Observed attempt: the first complete instrumented26-suite/2015-case run passed
all assertions and exact-SHA raw inventory at e2c3d00. Report parsing initially
refused LLVM’s DA-versus-LF/LH discrepancy. Both original exports and the failure
are retained; the reporting-only correction records both metrics and preserves
the unknown interpretation, without rerunning/changing a comparison or source
result. Report regeneration and independent raw-JSON summary checks verify it.
