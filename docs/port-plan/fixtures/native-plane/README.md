<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Native marked-plane command controls — 2026-11-04

57 independent original-first controls: flat and two fixed-seed 25-point grids,
four units and four original sigma cutoffs; short/reversed/duplicate/invalid
selections, forbidden known/assumed diameters, naming, missing selection and two
successive fits. Each side constructs its own file bytes. Retain the first 33
native import comparisons, 800 document controls and 49 worker controls.
Compare results/errors, every final store/history patch and source bytes,
logical/render geometry, worker identities/generations and each command step.
No native implementation or M3 acceptance follows from source-only captures.

Freeze the existing marked-plane numeric policy before native implementation:
fit geometry/residual floating values use 1e-12 * (1 + abs(original)), exactly
as port-core/tests/marked_plane.rs. Buffers, IDs, counts, source bytes, settings,
field presence/order, all error text and history metadata remain exact. Every
old passing comparison remains; this adds command state around the same kernel.

Append-only controls 0057–0063 cover actual global undo/redo, two identical-label
steps, empty history, branching after undo, and repeated travel. Controls
0064–0067 preserve lone UTF16 surrogates and exact JS BOM/blank-name trimming.
All earlier 57 complete original records replay byte-identically before the
appended controls. The native CLI uses the same registered body and sole history
recorder; `fit-plane scan.ply --sigma 0 --redo` runs fit, undo and redo.

Append-only controls 0068–0092 preregister native plane box selection: flat and
fixed-seed grids, four units, both default/specified clipping, normal facing,
empty/short boxes, inverted bounds, zero directions, 180 degrees and exclusive
selection guards. The original query/box helper runs before the native port;
retain all earlier 68 cases and the same numeric policy and exact state checks.

Controls 0093–0134 preregister the real scan.query-to-marked-plane path: box
and ball queries at limits 1/3/25, count/ids/all, direction/zero-direction, empty
queries, all-points marking despite thinning, stale marks after replacement and
missing marks. Query results and arrays remain exact; the unchanged fitter
tolerance applies only inside fit geometry. Every previous control remains.

Controls 0135–0136 check both plane and query commands on a real empty session.
All first 93 original observations replay byte-identically after the appended
query corpus. There are 137 controls in total. Deliberately changing the actual
native nextId rejects every control. Dropping a caller's waiter does not release
its native thread/resource charge before that producer finishes.

Controls 0137–0192 preregister plane finishing: growing/trimming and ordered
extension clamps over flat/seeded grids in every unit, empty/zero/invalid side
inputs and retained partial failures, coordinate-plane orientation and aliases,
in-plane degenerate geometry, existing-element references and lossless names.
The original is captured before the finishing body. The same established raw
marked-plane tolerance also applies to its retained measured fit; all prior
non-fit values, counts, extension values and errors stay exact. No earlier
numerical case is weakened: measured was absent in the earlier corpus.

Controls 0193–0194 resolve orientation by the actual section id and case-folded
section name. Both sessions independently contain the same section frame before
executing the command. All 195 cases remain, including every earlier 137 case.
The native body retains raw measured geometry, the orientation recipe and ordered
extension changes, including partial changes before a later side is refused.


## Public M3b partition

The complete 195-row master fixture remains unchanged. `cases-m3b.tsv` contains
188 select/fit controls; `cases-m3c.tsv` preserves exactly the seven global-history
controls 0057–0063 for the next ordered PR. They are deferred, not passing M3b
commands. No previously passing main comparison is removed: all 33 M3a import
controls and all 62 mandatory suites remain. M3b still compares actual history
record patches produced by plane creation; command-level travel is M3c.


Controls 0195–0211 preregister explicitly undefined optional plane/query inputs.
The original validator skips such fields and command guards/defaults treat them
as absent. Every previous row remains unchanged. The master now has 212 controls:
205 current M3b and the same seven deferred M3c controls. Original captures must
precede correction of the native presence/default mismatch; keep the real red.


## Public M3c global history

The comparison now consumes `cases.tsv` directly and reports all 212 master rows:
205 previously landed cases plus all seven history-travel cases, with no filters,
skips or relabeling. The master bytes, seeds and comparison policy are unchanged.
The older partition files remain historical inputs only, not current exclusions.

Manual wrong-state checks are opt-in through
`SCANRULER_NATIVE_HISTORY_CANARY=undo` or `=redo`. After a successful actual engine
travel, the test restores the pre-travel element payload while leaving recorder
bookkeeping and command results alone. The same state comparer must reject it.
Empty-history refusals are still compared normally and never mutated. No new
ignored Chrome/local-tool tests are added; the existing Node oracle entrypoint
continues to be invoked by the established oracle lane.
