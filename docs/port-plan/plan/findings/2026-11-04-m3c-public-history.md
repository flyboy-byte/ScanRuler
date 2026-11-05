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


Validated source: `e6e90b20abb7c621556208a19969f0b3dabf6621`. All **212/212** master controls pass together (205 prior
cases plus every one of the seven global-history cases); no skip/relabel or input
change. Six CLI tests pass. Full cargo xtask ci, full cargo xtask oracle and exact
artifact identity/completeness for all 62 required suites pass on that source;
no commit or source/metadata edit occurred during the full run.

Undo wrong-state red: all 212 exercised, five affected scenarios rejected after
seven actual successful undo operations. Redo wrong-state red: all 212 exercised,
two affected scenarios rejected after three actual successful redo operations.
Both returned exit 101 with document-elements mismatches; both complete red logs
were read. Command results and recorder bookkeeping remain intact in the test
mutation, proving that the checker examines the actual restored state. The clean
full-oracle run follows both red runs on unchanged source and passes all 212.

Raw original/native/case records, canary counts, source hashes and complete logs:
`evidence/2026-11-04-m3c-history-red-green.json.gz`. Required artifact hash report:
`evidence/2026-11-04-m3c-native-artifact-inventory.json.gz`
(uncompressed SHA256 `5f9078deb6c273f8c8a614c1a66fc63dc18e43290b7c5affd52490571cd649ee`). Final metadata/tool
checks and final-head hosted checks are separate; no feature acceptance inferred.
No roster, fuzz, crease-promotion, project or image files were changed.
