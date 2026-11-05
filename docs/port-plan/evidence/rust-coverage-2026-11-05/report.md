<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust differential corpus coverage — 2026-11-02

Candidate `c2966f3bd4d8bf9e676de3bb5b2ae3267aee3822`; original `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

Only mandatory differential tests: 69 suites /54040 identities. Ordinary unit tests are excluded from execution. File-level records can include their unexecuted inline bodies. LLVM explicit DA line counts are retained separately from LLVM LF/LH summary counters; discrepancies are reported, never normalized away.

| Rust source | Instrumented lines reached / total | Unreached explicit DA lines | LLVM LF / LH | DA/summary discrepancy |
| --- | ---: | --- | ---: | --- |
| rust/crates/port-core/src/adjacency.rs | 46 / 48 | 41, 55 | 49 / 47 | True |
| rust/crates/port-core/src/bounds.rs | 56 / 59 | 38, 51, 93 | 60 / 57 | True |
| rust/crates/port-core/src/centroid.rs | 86 / 87 | 24 | 88 / 87 | True |
| rust/crates/port-core/src/circle2d.rs | 93 / 100 | 33, 77, 78, 79, 95, 103, 120 | 102 / 93 | True |
| rust/crates/port-core/src/clip.rs | 96 / 98 | 48, 141 | 100 / 96 | True |
| rust/crates/port-core/src/closed_shells.rs | 308 / 335 | 87, 88, 89, 90, 91, 93, 107, 114, 325, 326, 327, 328, 329, 333, 334, 337, 338, 339, 340, 341, 344, 345, 346, 347, 348, 350, 406 | 351 / 324 | True |
| rust/crates/port-core/src/components.rs | 70 / 77 | 29, 36, 45, 63, 71, 87, 106 | 79 / 72 | True |
| rust/crates/port-core/src/crease.rs | 344 / 355 | 53, 61, 73, 81, 102, 107, 119, 256, 347, 358, 455 | 362 / 351 | True |
| rust/crates/port-core/src/curvature_gate.rs | 38 / 39 | 8 | 41 / 40 | True |
| rust/crates/port-core/src/cylinder.rs | 358 / 380 | 74, 75, 76, 80, 121, 155, 225, 261, 277, 278, 279, 281, 282, 299, 300, 358, 364, 402, 411, 445, 486, 501 | 388 / 364 | True |
| rust/crates/port-core/src/ids.rs | 0 / 12 | 21, 22, 23, 26, 27, 28, 55, 56, 57, 58, 59, 61 | 12 / 0 | False |
| rust/crates/port-core/src/js_date.rs | 63 / 64 | 19 | 64 / 63 | False |
| rust/crates/port-core/src/js_fixed.rs | 28 / 28 | none measured | 28 / 28 | False |
| rust/crates/port-core/src/js_number.rs | 64 / 70 | 30, 37, 38, 39, 85, 86 | 71 / 65 | True |
| rust/crates/port-core/src/js_string.rs | 37 / 41 | 24, 41, 44, 57 | 42 / 38 | True |
| rust/crates/port-core/src/lib.rs | 23 / 26 | 76, 79, 83 | 28 / 25 | True |
| rust/crates/port-core/src/linear.rs | 111 / 112 | 17 | 112 / 111 | False |
| rust/crates/port-core/src/marked_cylinder.rs | 140 / 179 | 67, 68, 69, 70, 72, 104, 117, 142, 210, 211, 212, 215, 216, 232, 236, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 252, 253, 254, 255, 256, 259, 263, 264, 265, 266, 269, 270 | 186 / 142 | True |
| rust/crates/port-core/src/marked_plane.rs | 199 / 208 | 55, 66, 113, 188, 223, 243, 244, 245, 246 | 215 / 203 | True |
| rust/crates/port-core/src/marked_sphere.rs | 60 / 63 | 120, 121, 122 | 70 / 65 | True |
| rust/crates/port-core/src/mesh_graph.rs | 86 / 104 | 56, 57, 58, 59, 60, 62, 106, 124, 125, 126, 136, 137, 138, 139, 143, 144, 145, 146 | 112 / 90 | True |
| rust/crates/port-core/src/mesh_selection.rs | 57 / 60 | 31, 54, 72 | 65 / 62 | True |
| rust/crates/port-core/src/normals.rs | 47 / 49 | 28, 31 | 50 / 48 | True |
| rust/crates/port-core/src/numeric.rs | 151 / 156 | 11, 36, 39, 45, 110 | 158 / 153 | True |
| rust/crates/port-core/src/orientation.rs | 60 / 61 | 38 | 63 / 62 | True |
| rust/crates/port-core/src/plane.rs | 96 / 99 | 26, 86, 122 | 103 / 100 | True |
| rust/crates/port-core/src/ransac.rs | 183 / 194 | 32, 33, 34, 35, 36, 37, 127, 130, 188, 284, 287 | 197 / 183 | True |
| rust/crates/port-core/src/selection.rs | 7 / 7 | none measured | 7 / 7 | False |
| rust/crates/port-core/src/sphere.rs | 212 / 220 | 40, 41, 42, 127, 149, 169, 212, 261 | 222 / 213 | True |
| rust/crates/port-core/src/transform.rs | 0 / 38 | 23, 24, 25, 26, 27, 28, 29, 30, 33, 34, 35, 36, 37, 38, 39, 42, 43, 44, 45, 46, 53, 54, 55, 56, 57, 58, 61, 62, 63, 64, 67, 68, 69, 70, 71, 73, 74, 75 | 39 / 0 | True |
| rust/crates/port-core/src/units.rs | 0 / 10 | 19, 20, 21, 26, 27, 28, 36, 37, 38, 39 | 10 / 0 | False |
| rust/crates/port-core/src/weld.rs | 73 / 76 | 51, 55, 105 | 77 / 74 | True |
| rust/crates/port-io/src/drop_files.rs | 80 / 86 | 80, 88, 106, 126, 146, 156 | 87 / 81 | True |
| rust/crates/port-io/src/inflate.rs | 351 / 395 | 37, 38, 39, 40, 42, 45, 55, 133, 222, 330, 390, 391, 392, 455, 456, 457, 458, 459, 460, 463, 479, 480, 481, 482, 483, 484, 485, 486, 488, 490, 491, 492, 494, 497, 498, 499, 500, 501, 502, 503, 505, 506, 507, 525 | 402 / 358 | True |
| rust/crates/port-io/src/lib.rs | 0 / 0 | unknown: no emitted coverage | None / None | False |
| rust/crates/port-io/src/obj.rs | 288 / 316 | 59, 60, 61, 131, 134, 155, 214, 246, 255, 279, 299, 307, 310, 311, 315, 325, 396, 397, 398, 399, 400, 401, 402, 403, 404, 405, 406, 407 | 324 / 296 | True |
| rust/crates/port-io/src/obj_texture.rs | 411 / 426 | 110, 116, 172, 199, 252, 292, 299, 348, 385, 402, 410, 416, 420, 427, 471 | 449 / 434 | True |
| rust/crates/port-io/src/ply.rs | 10 / 10 | none measured | 10 / 10 | False |
| rust/crates/port-io/src/ply_ascii.rs | 243 / 274 | 52, 59, 60, 61, 180, 187, 228, 241, 315, 347, 365, 400, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 421 | 286 / 255 | True |
| rust/crates/port-io/src/ply_binary.rs | 212 / 218 | 56, 74, 75, 76, 189, 193 | 221 / 215 | True |
| rust/crates/port-io/src/png.rs | 750 / 790 | 112, 631, 740, 798, 836, 840, 841, 842, 843, 844, 845, 846, 847, 888, 914, 918, 1022, 1029, 1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1049, 1051, 1052, 1057, 1061, 1062, 1082, 1088, 1097, 1108, 1127, 1138, 1146, 1152 | 804 / 764 | True |
| rust/crates/port-io/src/scan_files.rs | 150 / 179 | 62, 88, 149, 157, 183, 191, 212, 216, 247, 269, 270, 271, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 283, 284, 285, 288, 289, 290 | 182 / 153 | True |
| rust/crates/port-io/src/stl.rs | 37 / 38 | 35 | 39 / 38 | True |
| rust/crates/port-io/src/stl_dispatch.rs | 143 / 156 | 38, 39, 40, 41, 49, 50, 51, 52, 53, 55, 71, 146, 167 | 157 / 144 | True |
| rust/crates/port-io/src/threemf.rs | 936 / 952 | 88, 89, 90, 201, 202, 203, 204, 375, 679, 695, 1055, 1155, 1158, 1231, 1272, 1298 | 1010 / 994 | True |
| rust/crates/port-engine/src/activity_guard.rs | 36 / 50 | 34, 35, 36, 37, 38, 45, 46, 47, 48, 49, 67, 71, 72, 73 | 52 / 38 | True |
| rust/crates/port-engine/src/activity_state.rs | 93 / 100 | 97, 129, 130, 131, 144, 169, 194 | 102 / 95 | True |
| rust/crates/port-engine/src/command_deadline.rs | 130 / 148 | 87, 114, 115, 116, 120, 129, 130, 131, 136, 155, 178, 205, 227, 242, 243, 244, 246, 248 | 152 / 132 | True |
| rust/crates/port-engine/src/command_label.rs | 25 / 26 | 50 | 26 / 25 | False |
| rust/crates/port-engine/src/error.rs | 126 / 137 | 95, 119, 217, 218, 219, 220, 221, 222, 224, 225, 226 | 139 / 128 | True |
| rust/crates/port-engine/src/error_normalize.rs | 46 / 49 | 34, 85, 87 | 52 / 47 | True |
| rust/crates/port-engine/src/execute.rs | 110 / 128 | 26, 27, 28, 53, 75, 76, 77, 78, 82, 96, 157, 167, 168, 169, 213, 228, 236, 243 | 132 / 114 | True |
| rust/crates/port-engine/src/guarded.rs | 602 / 689 | 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 63, 64, 65, 243, 256, 266, 267, 305, 312, 315, 316, 317, 318, 320, 325, 326, 327, 393, 394, 395, 399, 400, 401, 411, 431, 432, 433, 434, 435, 436, 437, 530, 537, 567, 569, 576, 582, 609, 615, 642, 652, 661, 673, 697, 703, 710, 711, 721, 722, 723, 725, 726, 727, 728, 731, 732, 733, 734, 753, 760, 764, 854, 883, 889, 895, 926, 931, 940, 941, 942, 944, 945, 949, 955, 959 | 722 / 633 | True |
| rust/crates/port-engine/src/history_guards.rs | 42 / 43 | 104 | 46 / 45 | True |
| rust/crates/port-engine/src/history_local.rs | 29 / 31 | 46, 72 | 31 / 29 | False |
| rust/crates/port-engine/src/history_record/core.rs | 200 / 220 | 140, 149, 152, 165, 178, 198, 226, 227, 228, 229, 230, 236, 237, 238, 246, 261, 272, 291, 315, 318 | 232 / 210 | True |
| rust/crates/port-engine/src/history_record/restore.rs | 132 / 133 | 145 | 136 / 135 | True |
| rust/crates/port-engine/src/history_record.rs | 337 / 359 | 45, 47, 85, 213, 214, 215, 216, 217, 218, 219, 221, 285, 292, 299, 302, 308, 351, 352, 353, 448, 491, 518 | 372 / 349 | True |
| rust/crates/port-engine/src/import_queue.rs | 149 / 190 | 92, 103, 134, 135, 136, 137, 138, 142, 143, 144, 154, 155, 156, 157, 158, 159, 194, 195, 196, 198, 199, 200, 201, 202, 212, 237, 246, 278, 279, 292, 295, 296, 297, 307, 308, 309, 310, 317, 318, 319, 320 | 192 / 150 | True |
| rust/crates/port-engine/src/lib.rs | 0 / 0 | unknown: no emitted coverage | None / None | False |
| rust/crates/port-engine/src/native_engine.rs | 678 / 838 | 65, 75, 159, 200, 201, 202, 225, 226, 227, 228, 229, 230, 231, 232, 233, 244, 251, 288, 289, 314, 330, 351, 388, 430, 441, 465, 523, 549, 550, 551, 570, 579, 611, 615, 642, 676, 686, 720, 732, 733, 738, 770, 807, 810, 814, 846, 847, 848, 850, 851, 853, 860, 867, 869, 880, 948, 949, 950, 951, 954, 955, 956, 957, 958, 960, 961, 962, 964, 966, 967, 968, 988, 989, 990, 991, 993, 994, 995, 997, 998, 999, 1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1028, 1029, 1031, 1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039, 1040, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1062, 1063, 1064, 1065, 1066, 1067, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1088, 1089, 1092, 1093 | 892 / 713 | True |
| rust/crates/port-engine/src/native_history.rs | 169 / 213 | 15, 16, 17, 18, 19, 21, 42, 43, 44, 47, 61, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 91, 131, 168, 169, 170, 172, 177, 178, 179, 180, 189, 190, 191, 192, 211, 227, 228, 229, 230, 232, 234, 235, 236 | 224 / 177 | True |
| rust/crates/port-engine/src/native_history_definition.rs | 78 / 78 | none measured | 78 / 78 | False |
| rust/crates/port-engine/src/native_plane.rs | 453 / 518 | 12, 17, 46, 57, 65, 66, 77, 84, 85, 86, 87, 90, 91, 93, 95, 96, 97, 105, 112, 113, 114, 117, 120, 123, 128, 129, 130, 131, 132, 133, 134, 193, 212, 218, 233, 267, 268, 269, 270, 296, 302, 341, 348, 373, 391, 396, 397, 398, 399, 400, 401, 403, 404, 405, 443, 450, 451, 452, 453, 465, 488, 495, 545, 557, 576 | 546 / 475 | True |
| rust/crates/port-engine/src/native_plane_definition.rs | 337 / 337 | none measured | 337 / 337 | False |
| rust/crates/port-engine/src/native_plane_finish.rs | 295 / 360 | 21, 27, 32, 35, 40, 52, 107, 111, 131, 138, 142, 143, 145, 146, 147, 148, 149, 150, 154, 170, 180, 181, 183, 184, 185, 186, 187, 188, 194, 195, 196, 198, 199, 200, 201, 202, 203, 205, 206, 207, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 224, 275, 276, 277, 278, 280, 310, 354, 356, 357, 359, 372, 394, 398, 404 | 375 / 308 | True |
| rust/crates/port-engine/src/native_project.rs | 747 / 917 | 28, 33, 56, 113, 119, 120, 122, 123, 126, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 184, 185, 186, 225, 228, 284, 293, 294, 295, 296, 317, 318, 320, 442, 464, 492, 523, 530, 548, 549, 550, 551, 561, 562, 563, 564, 567, 568, 569, 570, 576, 577, 578, 579, 588, 589, 590, 591, 646, 647, 648, 661, 662, 663, 664, 667, 668, 669, 671, 672, 673, 674, 695, 699, 700, 701, 704, 705, 706, 708, 735, 738, 741, 742, 743, 747, 751, 757, 873, 874, 875, 876, 877, 883, 886, 902, 907, 941, 979, 980, 981, 982, 983, 984, 986, 988, 994, 1010, 1017, 1021, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1056, 1057, 1058, 1059, 1060, 1062, 1063, 1065, 1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1085, 1087, 1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1101, 1102, 1104, 1105, 1106, 1107, 1108, 1110, 1111 | 976 / 790 | True |
| rust/crates/port-engine/src/native_project_state.rs | 310 / 343 | 26, 51, 55, 64, 76, 78, 84, 89, 94, 228, 243, 247, 248, 250, 251, 252, 254, 257, 258, 260, 261, 262, 263, 286, 321, 322, 323, 324, 325, 326, 335, 343, 414 | 351 / 317 | True |
| rust/crates/port-engine/src/native_query_definition.rs | 201 / 201 | none measured | 201 / 201 | False |
| rust/crates/port-engine/src/native_scan_definition.rs | 88 / 88 | none measured | 89 / 89 | True |
| rust/crates/port-engine/src/native_selection.rs | 401 / 423 | 18, 21, 26, 34, 143, 144, 145, 241, 257, 258, 259, 260, 266, 289, 344, 391, 392, 393, 394, 408, 452, 493 | 455 / 421 | True |
| rust/crates/port-engine/src/page_stale.rs | 28 / 29 | 35 | 30 / 29 | True |
| rust/crates/port-engine/src/plugin_busy.rs | 98 / 108 | 27, 81, 82, 83, 84, 85, 106, 133, 161, 184 | 111 / 101 | True |
| rust/crates/port-engine/src/plugin_stop.rs | 49 / 52 | 59, 84, 85 | 53 / 50 | True |
| rust/crates/port-engine/src/project.rs | 181 / 192 | 32, 57, 76, 118, 120, 121, 135, 181, 184, 198, 241 | 200 / 187 | True |
| rust/crates/port-engine/src/project_json.rs | 203 / 259 | 15, 71, 72, 73, 74, 75, 76, 77, 78, 84, 87, 93, 98, 105, 108, 123, 127, 136, 153, 169, 184, 193, 197, 198, 199, 200, 201, 202, 203, 205, 206, 207, 212, 213, 215, 223, 224, 225, 226, 227, 228, 229, 251, 293, 295, 296, 297, 298, 299, 300, 301, 302, 303, 305, 306, 307 | 269 / 207 | True |
| rust/crates/port-engine/src/project_validation.rs | 247 / 337 | 11, 26, 35, 43, 52, 60, 82, 87, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 124, 126, 127, 128, 129, 130, 131, 132, 134, 135, 137, 139, 140, 142, 143, 144, 146, 147, 148, 149, 152, 153, 154, 155, 158, 159, 160, 161, 162, 163, 172, 176, 182, 187, 188, 189, 193, 198, 232, 233, 254, 258, 260, 261, 265, 272, 278, 287, 291, 292, 293, 296, 297, 298, 308, 314, 320, 323, 329, 333, 334, 335, 336, 337, 347, 353, 366 | 363 / 257 | True |
| rust/crates/port-engine/src/reactions.rs | 140 / 149 | 49, 101, 163, 166, 179, 197, 200, 237, 244 | 151 / 142 | True |
| rust/crates/port-engine/src/recent_result.rs | 126 / 131 | 29, 67, 74, 85, 93 | 131 / 126 | False |
| rust/crates/port-engine/src/recent_store.rs | 149 / 157 | 125, 161, 169, 178, 182, 228, 232, 233 | 162 / 154 | True |
| rust/crates/port-engine/src/registry.rs | 270 / 281 | 116, 123, 127, 143, 155, 173, 195, 226, 229, 241, 350 | 298 / 286 | True |
| rust/crates/port-engine/src/scan_session.rs | 521 / 595 | 92, 93, 94, 111, 138, 139, 140, 141, 142, 164, 173, 184, 199, 212, 216, 225, 230, 250, 254, 261, 270, 274, 289, 510, 511, 512, 529, 568, 585, 586, 587, 588, 589, 630, 635, 638, 641, 665, 672, 682, 694, 695, 696, 697, 699, 700, 701, 702, 703, 718, 746, 753, 754, 755, 756, 757, 758, 759, 760, 761, 762, 766, 774, 794, 826, 852, 889, 904, 907, 912, 921, 922, 923, 927 | 629 / 543 | True |
| rust/crates/port-engine/src/scan_worker.rs | 167 / 222 | 25, 26, 27, 84, 127, 154, 155, 156, 158, 159, 160, 162, 163, 164, 200, 208, 219, 221, 222, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 244, 245, 247, 248, 249, 250, 251, 252, 256, 265, 266, 267, 268, 270, 271, 272, 273, 302, 310, 320, 321, 327 | 244 / 176 | True |
| rust/crates/port-engine/src/scene_queue.rs | 34 / 34 | none measured | 37 / 37 | True |
| rust/crates/port-engine/src/schema.rs | 736 / 794 | 77, 78, 79, 117, 135, 140, 175, 176, 177, 178, 180, 198, 208, 219, 224, 227, 231, 235, 241, 245, 251, 268, 279, 282, 311, 349, 379, 380, 381, 382, 388, 389, 396, 403, 460, 487, 506, 517, 537, 564, 667, 678, 708, 717, 718, 719, 737, 744, 771, 801, 817, 859, 865, 913, 914, 915, 917, 918 | 823 / 762 | True |
| rust/crates/port-engine/src/schema_value.rs | 50 / 50 | none measured | 53 / 53 | True |
| rust/crates/port-engine/src/settle.rs | 36 / 37 | 76 | 37 / 36 | False |
| rust/crates/port-engine/src/settle_registry.rs | 84 / 98 | 55, 56, 57, 58, 59, 66, 67, 68, 69, 70, 93, 130, 155, 158 | 100 / 86 | True |
| rust/crates/port-engine/src/stale_error.rs | 29 / 30 | 42 | 32 / 30 | True |
| rust/crates/port-engine/src/stale_import.rs | 17 / 18 | 31 | 21 / 20 | True |
| rust/crates/port-engine/src/state_sections.rs | 132 / 143 | 117, 118, 119, 120, 121, 142, 180, 186, 210, 214, 240 | 150 / 139 | True |
| rust/crates/port-engine/src/tasks.rs | 192 / 208 | 93, 94, 193, 226, 229, 235, 276, 279, 282, 296, 313, 320, 332, 336, 353, 357 | 212 / 196 | True |
| rust/crates/port-engine/src/watch_session.rs | 43 / 44 | 59 | 46 / 45 | True |
| rust/crates/port-engine/src/watch_stores.rs | 24 / 33 | 17, 18, 19, 20, 21, 38, 42, 43, 44 | 33 / 24 | False |
| rust/crates/port-engine/src/workspaces.rs | 32 / 35 | 47, 55, 59 | 36 / 33 | True |

Unavailable: stable -C instrument-coverage emits regions/lines, no branch records in this run. Branch reach is unknown, not 100%.

LLVM LF/LH counters can disagree with explicit DA records for these multi-object exports. Both are preserved, with unresolved interpretation flagged per file; no denominator or pass percentage is inferred. Inline cfg(test) section markers and their instrumented lines are also explicit (some differential harnesses live there). Uninstrumented lines (comments, declarations, code not emitted/instantiated) are listed separately in report.json, never presumed covered or executable. Every measured source is hashed and mapped to canonical owning units. Missing owners/files remain explicit. Host allocation/depth/work guards may be unexecuted by the source comparison corpus; ordinary host tests are deliberately not blended into this report.

The system toolchain is recorded, not an immutable release pin. No tests, tolerances, seeds, outputs or source code were weakened. No acceptance or exact-base ratchet claim.
