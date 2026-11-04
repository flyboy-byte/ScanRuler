<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Main-based prepared-render prerequisite — 2026-11-04

Claim `geometry.mesh.crease` from main ac85ebd. This bounded prerequisite brings
forward the complete prepared render arrays, wire slots and crease continuation
needed by the native scan command. The source-first 1,309-case development corpus
and all red results are retained. The native roster and xtask remain owned by #104;
this PR does not edit either. The full oracle runs all ignored source comparisons,
including this added public production-module test, in addition to its required
roster. Mandatory inventory registration remains explicit follow-up work.

M3 still lacks integrated scan.open/CLI, plane fitting, history restoration and
original-compatible project save/read. The plane and CLI work is saved separately;
this PR contains only the renderer unit. Nothing is accepted or merged here.

## Reassessment checkpoint

Main-based renderer source: `d4514b1e2047ba86315d7c861673caaf1424fc81`.
Fresh full wrong-output/clean controls are retained in
`evidence/2026-11-04-scan-render-main-red-green.json.gz`, with all earlier red
records kept separately. No seed, case, exact comparison or threshold changed.

The existing status gate refused an attempted fixed-corpus promotion because the
new differential unit is not yet in the held native roster. That red setup result
is retained in the final gate archive. The unit therefore stays **planned/none**;
its open items carry the source SHA, evidence path and exact registration action.
No evidence kind was renamed to evade the gate. The source comparison still runs
as an added ignored test in the full native oracle, beyond the 61 required suites.
This PR does not edit xtask, the native roster, or PR104's engine module list.
Generated status/backlog are regenerated with the tools, never hand-merged.

Downstream work is saved separately for #105; none is silently in this PR:

- `codex/m3-selection-next` at `9565aff`: real scan.query/marks/box-plane paths,
  137 paired controls and full red/green records; full fast/oracle gates pass.
- `codex/m3-plane-finishing`: source `55eabd0`, saved evidence `435c109`;
  195 original-first plane/history/selection/orientation/extension controls,
  actual wrong nextId rejected all 195, clean committed-source rerun passed.
- `codex/m3-cli-selections` at `016a7b3`: thin real CLI selection/finishing and
  undo/redo, five passing black-box tests and full fast gate. Its branch starts
  at the finishing source; the separate finishing evidence branch holds the
  195-case proof archive. No compatible project save/read yet.

#104 is the ready scan-session PR. #105 remains a main-based **doc-only draft
claim**; its linked development branches contain the actual prototype code.
Review these branch scopes before claiming an integrated engine or M3. Logan
merges. Stop after this PR and let Claude reassess; do not begin another unit.
