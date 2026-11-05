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


Provenance/scope: copied only native_history.rs, native_history_definition.rs and
the runtime-recorder travel bridge from codex/m3-plane-finishing. The bridge's
old native-only compile cfg was dropped because the current module is also
compile-checked for Wasm; the synchronous pure-Rust restore body itself is
unchanged. Its supported hooks complete in one poll without a RefCell borrow
across await. Unsupported asynchronous/family/full-session hooks remain explicit.
The failed-travel message now preserves the original lossless label and wording.
From codex/m3-cli-selections, only parsing and command-dispatch blocks for undo/
redo were copied; current main's other CLI/engine code and five prior CLI tests
are preserved, with an additional black-box history test.

Dropped from extraction: old copies of scan IO, query/plane/finishing bodies,
core helpers, input/default logic, other tests, docs, inventory and dependency
edits already landed in #105/#108. No old-branch merge. No project/roster/image
work is included. Current main's optional-undefined fix is retained.
