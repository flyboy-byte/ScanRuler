# Image audit result — 2026-11-02

PR #39, io.obj.companions.image_audit. The manifest links exact source/candidate/
dependency/build/fixture/code hashes and the clean tested checkpoint. Source
capture and independent encoded inputs preceded the candidate code. A fresh
Chrome instance reproduced all 56 complete original pixel/omission observations.

The candidate's exact 22-package graph has allowlisted declared license options,
inspected shipped license texts, Rust implementations and reviewed build/FFI
paths. Native and linked Wasm build; the Wasm module imports nothing. Candidate
raw library pixels are **not** a faithful replacement: 8 exact decoded images,
28 pixel mismatches, 18 agreed omissions and 2 presence mismatches. Full bytes,
errors, identities, CPU/compiler/browser and difference positions/counts remain.
No normalization, tolerance change, source-answer table or counterexample removal.

Application dependencies stay unchanged; no image decoder port is claimed. The
actual source bitmap/canvas continuation, JPEG bytes, 16-bit conversion and
malformed omission still need faithful Rust work. Wasm evidence is compile/link,
not decoder execution. Four audit regression tests check full artifacts, strict
frames, mutation sensitivity, preserved negative verdicts and exact graph scope.
Existing application native/runtime inventories remain 24 suites /1,278 cases
and 651 Chrome cases; these 56 audit observations are not added as passing cases.

Reproduce using the isolated spike README. New source recaptures write to a new
path, preserving immutable evidence. The host's system Rust policy remains.
No CLA, deployment, upstream contact, dependency adoption or accepted-unit claim.
