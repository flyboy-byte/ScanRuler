# Fixed-selection sphere fitting

2026-11-03. PR #89 ports all sphere.ts bodies: centred Coope algebraic initialization,
150-round free geometric refinement, 50-round held-radius Gauss-Newton centre fit,
shared twelve-round clipping, reported sigma/span/signed extremes/used indices,
and the optional free-radius companion refitted on the final used points.

Reuse the existing source-compared solveLinear, generic clippedRefit and compensated
norm at Math.hypot. Preserve every formula, accumulation/division order, pivot and
convergence guard. No FMA/reassociation, alternative solver, analytic fallback or
normalization of held radii is introduced. The source's incomplete final finite
check in free refinement is retained exactly. k<=0 uses all points; nonfinite k
and zero/negative/nonfinite held radii retain source behavior. Invalid packed
buffers/selection bounds/point quotas are host errors; numerical refusal is None.

1560 original-first fixed descriptors cover four seeds, exact/noisy/full/cap scans,
outliers, translated/tiny/large scales, algebraic degeneracies, centre points,
nonfinite coordinates, duplicate/reversed selections and held radius categories.
Appended clipping/radius boundaries preserve every byte of the first1344 passing
source profiles. Both engines independently generate Float32 positions/Uint32 IDs
and compare complete actual input bytes before computing. Inputs are checked again
afterward. Compare result/free presence, all centre/radius/statistic fields and
ordered used IDs, not just success/error status. Refusals and successful models
are counted separately. Reference results never enter native computation.

The scalar comparator was frozen in the draft claim before implementation:
absolute1e-10 + relative1e-10 times absolute reference. Optionals, ordered IDs,
nonfinite classes and zero signs are exact. Negative comparator probes reject
changed values, IDs, presence and signed zero. No existing case, seed, tolerance
or passing comparison was removed/relaxed. No completion percentage is inferred.

The oracle loads complete pinned sphere.ts, clip.ts and linalg.ts modules using
TypeScript's types-only ES2022/CommonJS transpilation. It does not extract bodies,
rewrite formulas or replace helpers. Full sources/transpiled modules/source maps,
compiler/Node identities, complete source transcript and raw paired input/output/
post-input artifacts are retained. The generators use integer LCGs and explicit
Float32 storage, avoiding trig-dependent fixture drift.

Rust/Wasm compilation and all existing native source suites remain required.
Actual browser-Wasm sphere execution, seeded/RANSAC/growth paths, full selection
normal-voting/extents/element fields and engine commands/session/CLI wiring remain
open. This fixed-selection library unit is not an accepted sphere feature or M3.
