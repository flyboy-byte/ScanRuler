<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Original corpus coverage — 2026-11-02

Reference `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`; tool checkpoint `96c776c2abe15e184b1dd39c21a13387c88d59fa`; native input/results checkpoint `fa711afd2275c22279b287ede832c165c51b4a1e`.

1422 complete output/sidecar replays under v22.23.4, TypeScript5.9.3; all identical to the native-gate original artifacts. Coverage is diagnostic, not parity or acceptance.

| Original source | Mapped lines reached / total | Unreached lines | Unreached observed blocks / total |
| --- | ---: | --- | ---: |
| src/core/parsers/stl.ts | 52 / 52 | none | 0 / 17 |
| src/core/parsers/ply.ts | 265 / 265 | none | 2 / 136 |
| src/core/parsers/objTexture.ts | 127 / 127 | none | 1 / 66 |
| src/core/parsers/obj.ts | 105 / 105 | none | 1 / 63 |
| src/core/formats.ts | 54 / 54 | none | 1 / 2 |
| src/app/scanFiles.ts | 71 / 109 | 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126 | 2 / 33 |
| src/commands/types.ts | 19 / 19 | none | 0 / 0 |
| src/commands/schema.ts | 184 / 201 | 66, 77, 169, 246, 247, 248, 251, 252, 253, 254, 260, 261, 262, 263, 264, 265, 266 | 11 / 144 |

Compiler source-map segments classified by smallest enclosing V8 function/block range; line reached if any mapped segment executes; partial if some also unexecuted.

V8 observed detailed block-range union; uncalled functions may hide internal branches. Not a static condition/edge or MC/DC denominator.

## Sources not mapped by this report

- `tests/headlessEngine.test.ts`
- `src/core/fit/plane.ts`
- `src/core/fit/linalg.ts`
- `src/core/fit/clip.ts`
- `src/core/fit/ransac.ts`
- `src/core/fit/fitPlaneFromSeed.ts`
- `src/core/fit/selection.ts`
- `src/core/vec.ts`
- `src/core/geometry/adjacency.ts`
- `src/core/geometry/bounds.ts`
- `src/core/geometry/centroid.ts`
- `src/core/geometry/closedShells.ts`
- `src/core/geometry/unionFind.ts`
- `src/core/geometry/buildGraph.ts`
- `src/core/geometry/normals.ts`
- `src/core/geometry/region.ts`
- `src/core/geometry/remap.ts`
- `src/core/geometry/weld.ts`

Those source/helper comparisons still run in the mandatory native gate. Their source locations are unknown here, never zero uncovered or 100% reached. Each affected implemented unit retains an explicit coverage follow-up. Unreached lines and zero-count block/function records are generated in report.json; do not edit the generated lists.

Independent fuzz, actual browser execution and static branch/condition coverage remain open. No thresholds, original source code, fixtures, seeds or comparisons changed.
