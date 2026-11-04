<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Explicit undefined optional arguments — 2026-11-04

Original schema.ts skips undefined optional values without removing their keys.
Plane guards compare against undefined and query destructuring supplies defaults.
The native argument reader instead returned Some(Undefined), so presence tests
mistook those optional values for supplied inputs. This changed sigma memory,
triggered unwanted rename/diameter guards and failed otherwise valid queries.

Seventeen original-first controls are appended as 0195–0211. Every old row, ID,
seed and policy remains: 212 master rows, 205 M3b and the same 7 M3c rows. Source
was recaptured before fixing native behavior. Test-only checkpoint 1040700 on the
unchanged handler produces 13 real mismatches across all 205 cases; four new
controls already matched. Earlier checkpoint 30f8f5c stopped on an unexpected
host fault after recording the preceding cases. Preserve that panic/red record
as partial evidence, not a passing or complete run.

The comparison helper now records unexpected Host faults with a distinct
$nativeHostFailure / NativeHostFailure marker, which cannot match source command
outcomes; it does not map them to allowed source refusals. This lets all raw cases
be recorded. The comparer and numerical tolerances are unchanged.

The production fix is limited to the shared native optional-argument reader:
return absence for Undefined, preserving false, zero and all other defined values.
Input objects themselves, schema validation, own-key order and raw extension-key
iteration remain unchanged. Re-run the complete corpus, wrong-state control,
prior import cases, all required oracle suites and exact artifact identity before
readiness. No Rust-derived expected values or gate changes.
