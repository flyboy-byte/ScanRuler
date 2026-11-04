<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# M3a public scan.open and info — 2026-11-04

PR105 is now the actual main-based code step, following main b37206d. Extract the
real runtime/task/import-queue/document/worker composition and thin filesystem
info frontend from the saved workbench. Keep the later process-wide producer
budget: a dropped waiter cannot release the slot or source assets while its
thread still runs. Its regression also verifies an admitted scan survives a
replacement refused at the native producer limit, with no history entry.

All 33 earlier registered-import controls remain as an independent fixture;
no tolerance, seed, source result or pinned TypeScript is changed. The source
comparison and its negative native-state control are re-run on this extracted
code. Two CLI black-box tests cover all PLY encodings, real original summary
values, malformed input and schema refusal with nonzero exit/no partial stdout.

The decision for the existing internal port-engine -> port-io edge is included;
there are no third-party dependencies, and port-io codec/archive files are not
edited. Native OS threads/files are adapters; the owned core/session still builds
for Wasm. Rendering is the public main implementation from PR106.

Scope remaining: M3b query/marks/box/plane and fit-plane CLI, M3c global undo/redo,
M3d compatible project save/read. Open those sharing-file PRs only after the prior
step merges. The full parent engine.measurement.slice stays planned, with explicit
M3a progress; numerical promotion/roster integration follows roadmap section 0.
The existing all-tests oracle invocation executes the added import comparison;
no native roster/discovery gate is removed or weakened.

Requested planning cleanup is included: geometry.mesh.crease is assigned to lane
D, and the historical ci.integration.nov03 batch is integrated/closed against
PR95 merge ac60301 (an ancestor of main). Its 11 required exact-head checks were
verified green on GitHub; the recorded closure does not claim new numerical or
feature acceptance. Generated files are rendered by the repo tools.


Validated source checkpoint: `326b9989a7f8c3c6ff463d33a69f19af3168519d`. All 33 registered import comparisons pass;
the actual vertexCount=-99 mutation rejects all 33, followed by a clean unchanged
committed-source rerun. Full `cargo xtask ci` and `cargo xtask oracle` pass. Exact
artifact completeness/identity passes for all 62 required suites on that same
source commit; no metadata/source commit occurred during the full run.

Fresh full raw records, original recapture, hashes and development failure/gate
logs: `evidence/2026-11-04-m3a-public-import-red-green.json.gz`.
Full required-roster artifact hash inventory:
`evidence/2026-11-04-m3a-native-artifact-inventory.json.gz`
(SHA256 of the uncompressed report: `e00db45eaad30842cfe9c51d2f1480f17304e1be5008bfeeb2734d17987fd732`).
Final evidence/docs metadata is separate from the tested source checkpoint;
hosted checks independently validate the final PR head before it becomes ready.
