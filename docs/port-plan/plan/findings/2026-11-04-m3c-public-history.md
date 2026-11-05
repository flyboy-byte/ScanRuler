<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# M3c public global history — 2026-11-04

Claim engine.measurement.slice M3c from main ed5170c after M3a/M3b merged.
Bring only the actual runtime-recorder restoration adapter and global history
handlers from codex/m3-plane-finishing; bring only the CLI undo/redo switches
and black-box assertions from codex/m3-cli-selections. Preserve current main's
import/query/plane/optional-input fixes rather than merge old branches wholesale.

Compare all 212 master controls together: 205 already landed plus the seven
history-travel controls previously deferred. Add an opt-in wrong-restore-state
control that must fail after an actual undo/redo; retain the red and clean records.
No new ignored local-tool/Chrome tests; mutation controls require an explicit env
flag. No project IO, roster/fuzz registration, crease promotion or image changes.
Logan merges. Source/reference, seeds, tolerances and timing gates remain fixed.
