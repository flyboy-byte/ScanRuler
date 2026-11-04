<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Actual scan/session ownership closure — 2026-11-03

Source pin `2cd89c6` / app 0.9.2; local follow-up starts from #100 head
`8789795`. #100 alone owns shared public PR files while it remains open.
No scan-model implementation or acceptance claim is made by this audit.

`scanImport.commitScan` is the first complete bounded consumer body. It owns
source/model replacement and publishes app/deviation/thickness/mark state, map
and surface invalidation, optional scene/view and plugin effects, then history
clear. Native application data must be real mesh/source assets and the single
business document; the current runtime owns only metadata/history. Immutable
geometry/source assets may be shared by snapshots; a second mutable document
or a fixture scalar is not a session.

Important source constraints:

- Element/dimension IDs and persistent settings survive beginLoad; per-scan
  numbering, elements/drafts, dimensions/sections and visibility restart.
- A file reference/source remains; a session-derived nominal is cleared before
  maps and alignment. Image/other workspaces are not implicitly reset.
- Preview epoch advances, all surface regions clear and their version advances,
  map buffers clear, marking gesture/erase/count reset while other tools persist.
- A prepared scan publishes beginLoad, scanColors, view commit and finishLoad,
  then brush/thickness sizing and loaded callbacks. No viewport uses boundsOf
  on the prepared render positions with the original logical vertex count.
- Null commit has a distinct clearScan/model-size branch. Both finish busy=true,
  then clear the actual runtime history. Import orchestration clears busy later.
- Store callbacks and scan listeners may throw after earlier publications. Those
  partial effects are authoritative; do not invent transaction rollback there.
  Failed preparation before a commit is a separate preservation guarantee.
- Prepared view, optional scene and plugin callbacks are trusted owned native
  services with explicit fault/lifetime boundaries, never hidden JS behavior stubs.

`prepareScan/openScan` is not just STL + buildMeshGraph. meshWorker.postScan calls
wireSlots and scanRender/creaseFor/splitCreases, transferring render buffers and
copyOf metadata distinct from the worker logical graph. Those are currently
missing Rust continuations. Do not force crease off or claim the full import
body without them. Worker staged IDs, commit/discard/version replacement, unit
scaling, source-byte copying, statuses and finally disposal are also required.

Proposed bounded children: engine.session.scan.commit (actual model and complete
prepared commit), geometry.mesh.crease (render continuation), then
engine.session.scan.import (actual preparation and scan.open). Marked-plane,
global undo/apply and original-compatible project/CLI save follow. Every earlier
case/seed/gate remains; none of these helpers alone closes M3.
