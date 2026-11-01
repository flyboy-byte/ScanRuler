# Fixed-selection plane comparison contract — pre-registered 2026-11-01

Before implementing/running Rust plane math: fitPlaneTLS/planeResidual are the bounded target, with centroidOf/symmetricEigen3/normalizeOrZ as co-ported dependencies. No clipping, RANSAC or splitTwoPlanes claim. Keep accumulation, Jacobi pair/sweep order, sorting and refusal thresholds unchanged.

Compare presence/refusal and output lengths exactly; compare each of four plane fields and every selected residual with abs(error) <= 1e-12 * (1 + abs(original)). No sign normalization. This fixed budget derives from f64 representation/rounding (about 4500 eps; much stricter than source planeFit.test.ts micron-scale/noisy checks) and the source's 12-decimal internal consistency assertion. It is set before Rust mismatches; do not loosen it. Rust implements scaled compensated three-component norm using sqrt, avoiding native libc hypot and permitting only platform math rounding differences at the Math.hypot boundary. All fit formulas/iteration order remain pinned.

Require exact/tilted/noisy seeded finite planes, repeated/collinear/insufficient points, non-contiguous selection and raw independent original/native outputs. Browser-Wasm runtime, expanded extreme/nonfinite corpus and P36 ratchet remain missing until demonstrated. This is a targeted partial implementation, not acceptance of G0/G2 or full fitting.
