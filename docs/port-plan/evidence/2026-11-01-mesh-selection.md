# Ordered mesh selection preservation — 2026-11-01

Implementation: 3f56d3ee7683b5700446300382a87022cab558bc; frozen reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7. Exact corpus preregistration: 68696a6, plan/decisions/2026-11-01-mesh-selection-comparison.md.

Complete region.ts::trianglesWithin and remap.ts::remapVertices. All three triangle corners must be marked; out-of-range marks are ignored and out-of-range corners excluded. Incomplete trailing triples are skipped, not refused. Remaps drop missing/negative entries. Both functions retain source order/duplicates and perform the original two passes. Rust only adds fallible storage and explicit vertex/output budgets, without adding malformed-input rejection that would change the admitted source filtering rules.

62 independent cases: six fixed remaps, eight fixed triangle filters and forty-eight generated cases. Seeds 1/7/99/1039, vertices/map lengths 0/1/17, selections 0/31, wrapping LCG 1664525/1013904223. Fixtures include nonmonotone signed maps, i32 extremes, u32::MAX missing IDs, repeated marks/triangles, tails and empty results. Complete output length and every index compare byte-for-byte: 23 nonempty outputs, with deliberate empty outputs also checked. No tolerance, sorting, deduplication or reference-fed Rust result. Raw inputs/original/Rust plus candidate: target/mesh-selection-diff/run-*; CSV retains every hash and CI retains raw outputs.

At the implementation revision, cargo xtask ci and cargo xtask oracle pass: 25 ordinary tests plus fifteen native source suites (521 case identities including refusals). Formatting/lints, Wasm compilation and plugin-boundary check pass. Tests cover budget failures and remap-then-filter composition. Local Node 20.20.2, hosted 22. Original-app CI remains separate; display-unit checker lacks local TypeScript dependency and no displayed length changed.

Implemented, not accepted. No browser-Wasm execution, mesh edit/undo/project/session command integration, source/render mapping, G0/P05 or general P36 exact-base ratchet acceptance. PR #19 stacks on #18. No original source/tests, frozen pin or dependencies changed. See AUDIT-QUEUE.md.
