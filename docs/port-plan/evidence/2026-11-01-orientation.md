# Outward mesh orientation — 2026-11-01

Unit geometry.normals.orient; exact implementation SHA in its unit file/raw candidate.txt. Source 2cd89c6769a10c27ee7754b477ada521c9ef5cb7: normals.ts::orientNormalsOutward, centroid.ts::tetraVolume6 and bounds.ts::extentOf. Co-ported minimal extent reduction is internal; no complete bounds or centroid API is claimed. tetra_volume6's point-array boundary preserves the source arithmetic and is reusable by later centroid work.

Fifteen independent source/native cases compare the returned boolean, all triangle indices and all normal float32 bits (orientation.state.v1). Six genuine flips and nine no-ops: empty/isolated/sheet and tetrahedra with inward/outward winding, scales 1/0.001/10000 and translations 0/1039. No tolerance and no source results supplied to Rust. Normals are independently generated before either run. Host malformed-array/index errors are checked for no mutation.

The centroid accumulation, translated tetrahedral volume sum, extent/bbox order and -6e-3 threshold follow the frozen original. A narrowly scoped clippy allowance retains the source's NaN-sensitive negated comparison. Do not rewrite it as bbox <= 0. Both winding and normals change together; positions stay borrowed and immutable. Original heuristic limitations on open meshes remain.

Validation: cargo xtask ci passes (eleven ordinary tests, fmt/clippy, plan checks and wasm32 library compilation); explicit pinned_orientation_comparison test passes on local Node 20.20.2 / Rust 1.99.1. Raw run rust/target/orientation-diff/run-1793565735065996004; adjacent CSV contains input/output SHA-256 hashes. CI runs Node 22 and retains mesh-orientation-comparison seven days. Existing STL/weld/normal comparisons remain active.

Still implemented, not accepted: browser-Wasm runtime, threshold-adjacent/open/non-finite fuzz corpus, shared G0/P05 contracts, general P36 ratchet and engine/session integration. Caller bypass for settled file winding remains required when integrating. No source expectation or threshold changed.
