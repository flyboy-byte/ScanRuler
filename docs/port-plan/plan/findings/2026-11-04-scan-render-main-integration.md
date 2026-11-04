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
