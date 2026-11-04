<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# M3a registered native scan.open controls — 2026-11-04

These are the same 33 original-first controls already exercised by the saved
native-info workbench: first 12 original owned-session cases plus the 21 PLY/held
cases. Their IDs, inputs, seeds and exact wire comparison are unchanged; they now
have a separate input fixture so earlier owned-session fixtures are not enlarged
or filtered differently. Original and Rust independently generate each STL/PLY.

The corpus covers triangle/quad/malformed STL and every PLY encoding in all four
units, malformed PLY, held dimensions/sections/mixed/project state and explicit
discard. Compare complete result/error, stores/history/source bytes, logical and
render arrays, real worker resources, generations and request IDs, exactly.
The actual native vertexCount=-99 canary must fail all 33 before a clean rerun.
No Rust-derived goldens are used. Original-first development records remain at
saved checkpoint 049e873 in evidence/2026-11-04-native-info-development.json.gz.

This bounded step integrates native scan.open and CLI info only. The parent
measurement slice stays planned until selection/plane/history and its required
roster promotion are integrated. No full UI/project/nominal-kernel acceptance.
