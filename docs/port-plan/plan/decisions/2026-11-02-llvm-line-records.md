<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Preserve LLVM line-record discrepancies — 2026-11-02

Unit `harness.coverage.rust`, review owner/integrator PR #44.

MEASURED: the complete instrumented original/Rust gate passes all26 suites/2015
cases at e2c3d0010700899a47c6ec6c454d5cb3299b1141. Installed rustc1.99.1 uses
LLVM23.1.3; llvm-profdata/cov23.1.4 accepts and exports every profile. No tests,
seeds or numerical tolerance changed. Raw exact-candidate inventory is complete.

MEASURED: llvm-cov's multi-object LCOV DA records disagree with its LF/LH totals
on29 of35 owned source files. Example adjacency.rs:48 explicit DA lines,46
positive records, LF49/LH47. The complete JSON file summary repeats49/47; these
are not numbers invented by our parser. ransac.rs even has183 positive DA lines
but summary181, so an implicit extra-zero-line interpretation is unjustified.
The original LCOV and complete region JSON are retained, with both metrics.
Root cause is UNKNOWN; duplicate compiler/library instantiations may contribute
(INFERRED), but no claim of an LLVM bug or a resolved interpretation is made.

The initial parser required DA counts == LF/LH and stopped. That assumption was
ours, not the port's numerical contract. DESIGN: report explicit line IDs/counts
from DA, retain LF/LH as separate values and flag every discrepancy. Cross-check
those summary values independently against raw LLVM JSON. Do not silently repair
the export, choose a preferred percentage, hide zero lines or lower a coverage
threshold. No threshold or acceptance percentage exists for this diagnostic.
A discrepancy remains an open coverage-interpretation item in affected units.

MEASURED: three lib.rs module files have no emitted executable line records. Inline
cfg(test) sections occur in seven src files; they contain unexecuted ordinary
host tests or the inline source-comparison harness. Their markers/line records
remain explicit, not claimed as production functionality. Uninstrumented code
is not presumed reachable or unreachable. Stable flags produce no branch records;
branch reach stays unknown. Full static branch/condition coverage, browser/fuzz
campaigns and immutable toolchain pinning remain open.

This changes the reporter's interpretation of external coverage metadata only.
The successful comparison corpus, all raw original/Rust observations, profile
bytes, emitted code and source formulas remain unchanged. Fresh runs use the same
one-command collector; historical metrics regenerate from the archived exports.
