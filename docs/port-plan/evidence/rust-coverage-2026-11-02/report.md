<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust differential corpus coverage — 2026-11-02

Candidate `e2c3d0010700899a47c6ec6c454d5cb3299b1141`; original `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

Only mandatory differential tests: 26 suites /2015 identities. Ordinary unit tests are excluded from execution. File-level records can include their unexecuted inline bodies. LLVM explicit DA line counts are retained separately from LLVM LF/LH summary counters; discrepancies are reported, never normalized away.

| Rust source | Instrumented lines reached / total | Unreached explicit DA lines | LLVM LF / LH | DA/summary discrepancy |
| --- | ---: | --- | ---: | --- |
| rust/crates/port-core/src/adjacency.rs | 46 / 48 | 41, 55 | 49 / 47 | True |
| rust/crates/port-core/src/bounds.rs | 56 / 59 | 38, 51, 93 | 60 / 57 | True |
| rust/crates/port-core/src/centroid.rs | 86 / 87 | 24 | 88 / 87 | True |
| rust/crates/port-core/src/clip.rs | 96 / 98 | 48, 141 | 100 / 96 | True |
| rust/crates/port-core/src/closed_shells.rs | 308 / 335 | 87, 88, 89, 90, 91, 93, 107, 114, 325, 326, 327, 328, 329, 333, 334, 337, 338, 339, 340, 341, 344, 345, 346, 347, 348, 350, 406 | 351 / 324 | True |
| rust/crates/port-core/src/components.rs | 70 / 77 | 29, 36, 45, 63, 71, 87, 106 | 79 / 72 | True |
| rust/crates/port-core/src/curvature_gate.rs | 38 / 39 | 8 | 41 / 40 | True |
| rust/crates/port-core/src/ids.rs | 0 / 12 | 21, 22, 23, 26, 27, 28, 55, 56, 57, 58, 59, 61 | 12 / 0 | False |
| rust/crates/port-core/src/lib.rs | 0 / 0 | unknown: no emitted coverage | None / None | False |
| rust/crates/port-core/src/linear.rs | 108 / 112 | 17, 92, 94, 100 | 112 / 108 | False |
| rust/crates/port-core/src/marked_plane.rs | 203 / 212 | 55, 66, 113, 188, 227, 247, 248, 249, 250 | 219 / 207 | True |
| rust/crates/port-core/src/mesh_graph.rs | 86 / 104 | 56, 57, 58, 59, 60, 62, 106, 124, 125, 126, 136, 137, 138, 139, 143, 144, 145, 146 | 112 / 90 | True |
| rust/crates/port-core/src/mesh_selection.rs | 57 / 60 | 31, 54, 72 | 65 / 62 | True |
| rust/crates/port-core/src/normals.rs | 47 / 49 | 28, 31 | 50 / 48 | True |
| rust/crates/port-core/src/numeric.rs | 18 / 20 | 11, 13 | 21 / 19 | True |
| rust/crates/port-core/src/orientation.rs | 60 / 61 | 38 | 63 / 62 | True |
| rust/crates/port-core/src/plane.rs | 96 / 99 | 26, 86, 122 | 103 / 100 | True |
| rust/crates/port-core/src/ransac.rs | 183 / 194 | 32, 33, 34, 35, 36, 37, 127, 130, 188, 284, 287 | 197 / 181 | True |
| rust/crates/port-core/src/transform.rs | 0 / 38 | 23, 24, 25, 26, 27, 28, 29, 30, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 45, 46, 53, 54, 55, 56, 57, 58, 61, 62, 63, 64, 67, 68, 69, 70, 71, 73, 74, 75 | 39 / 0 | True |
| rust/crates/port-core/src/units.rs | 0 / 10 | 19, 20, 21, 26, 27, 28, 36, 37, 38, 39 | 10 / 0 | False |
| rust/crates/port-core/src/weld.rs | 73 / 76 | 51, 55, 105 | 77 / 74 | True |
| rust/crates/port-io/src/drop_files.rs | 80 / 86 | 80, 88, 106, 126, 146, 156 | 87 / 81 | True |
| rust/crates/port-io/src/lib.rs | 0 / 0 | unknown: no emitted coverage | None / None | False |
| rust/crates/port-io/src/obj.rs | 285 / 316 | 59, 60, 61, 111, 130, 131, 134, 155, 166, 214, 246, 255, 279, 299, 307, 310, 311, 315, 325, 396, 397, 398, 399, 400, 401, 402, 403, 404, 405, 406, 407 | 324 / 293 | True |
| rust/crates/port-io/src/obj_texture.rs | 411 / 426 | 110, 116, 172, 199, 252, 292, 299, 348, 385, 402, 410, 416, 420, 427, 471 | 449 / 434 | True |
| rust/crates/port-io/src/ply.rs | 10 / 10 | none measured | 10 / 9 | True |
| rust/crates/port-io/src/ply_ascii.rs | 257 / 302 | 52, 59, 60, 61, 180, 187, 228, 241, 315, 347, 365, 394, 397, 400, 411, 412, 413, 414, 416, 417, 418, 420, 422, 423, 425, 435, 437, 438, 439, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 456 | 315 / 270 | True |
| rust/crates/port-io/src/ply_binary.rs | 210 / 218 | 23, 56, 68, 74, 75, 76, 189, 193 | 221 / 213 | True |
| rust/crates/port-io/src/scan_files.rs | 150 / 179 | 62, 88, 149, 157, 183, 191, 212, 216, 247, 269, 270, 271, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 283, 284, 285, 288, 289, 290 | 182 / 153 | True |
| rust/crates/port-io/src/stl.rs | 37 / 38 | 35 | 39 / 38 | True |
| rust/crates/port-io/src/stl_dispatch.rs | 143 / 156 | 38, 39, 40, 41, 49, 50, 51, 52, 53, 55, 71, 146, 167 | 157 / 144 | True |
| rust/crates/port-engine/src/error.rs | 105 / 137 | 95, 108, 109, 110, 111, 112, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 217, 218, 219, 220, 221, 222, 224, 225, 226 | 139 / 107 | True |
| rust/crates/port-engine/src/lib.rs | 0 / 0 | unknown: no emitted coverage | None / None | False |
| rust/crates/port-engine/src/schema.rs | 736 / 794 | 77, 78, 79, 117, 135, 140, 175, 176, 177, 178, 180, 198, 208, 219, 224, 227, 231, 235, 241, 245, 251, 268, 279, 282, 311, 349, 379, 380, 381, 382, 388, 389, 396, 403, 460, 487, 506, 517, 537, 564, 667, 678, 708, 717, 718, 719, 737, 744, 771, 801, 817, 859, 865, 913, 914, 915, 917, 918 | 823 / 762 | True |
| rust/crates/port-engine/src/schema_value.rs | 49 / 50 | 71 | 53 / 52 | True |

Unavailable: stable -C instrument-coverage emits regions/lines, no branch records in this run. Branch reach is unknown, not 100%.

LLVM LF/LH counters can disagree with explicit DA records for these multi-object exports. Both are preserved, with unresolved interpretation flagged per file; no denominator or pass percentage is inferred. Inline cfg(test) section markers and their instrumented lines are also explicit (some differential harnesses live there). Uninstrumented lines (comments, declarations, code not emitted/instantiated) are listed separately in report.json, never presumed covered or executable. Every measured source is hashed and mapped to canonical owning units. Missing owners/files remain explicit. Host allocation/depth/work guards may be unexecuted by the source comparison corpus; ordinary host tests are deliberately not blended into this report.

The system toolchain is recorded, not an immutable release pin. No tests, tolerances, seeds, outputs or source code were weakened. No acceptance or exact-base ratchet claim.
