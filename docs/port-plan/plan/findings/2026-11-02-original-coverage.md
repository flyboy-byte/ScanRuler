<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Original coverage targets — 2026-11-02

MEASURED at clean tool96c776c, input/result checkpointfa711af:1422 exact native
original replays, eight whole source maps. Full generated line/block/function
locations and counts: [report](../../evidence/original-coverage-2026-11-02/report.json).
This finding describes corpus reach, not correctness or acceptance.

- scanFiles.ts has38 unreached mapped lines in decodeImage/readCompanions/image
  ingress, not exercised by these native file/drop inputs. The separate Chrome
  image audit is not included in this report. Targets remain in
  io.obj.companions.images and ingress/session continuations.
- schema.ts has17 unreached mapped lines, including a names-slot guard, a type
  fallback, a single-character minimum refusal and unused builder functions.
  Eleven observed V8 block ranges are zero-count. Add small named source cases
  for each reachable validator guard before claiming exhaustive branch reach;
  schema builders are outside the implemented validate/weight scope.
- PLY, OBJ, OBJ-texture and format classifiers each retain zero-count block
  ranges despite every mapped line being reached. Generated block offsets are
  the reproducer targets; line-only percentages conceal these gaps.
- Eighteen source paths remain unmapped, mostly manually stripped/concatenated
  fit/geometry bodies. Every implemented owner has an explicit open item. Build
  trustworthy span maps or whole-module loaders, then replay its existing cases;
  do not guess locations by matching similar text. baseline's headless test path
  is included as an explicit unmapped diagnostic, not a product function port.

LIMIT: V8's observed block union does not enumerate all static condition edges,
particularly inside uncalled functions. Preserve that qualification. No fixtures,
seeds, tolerances, source algorithms or passing comparisons were removed. This
report adds no case identities to the native roster and no browser-runtime claim.
