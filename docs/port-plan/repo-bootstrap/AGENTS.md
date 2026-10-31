# Rust port contributor instructions

This is a faithful pure-Rust port of the pinned ScanRuler baseline. Stefan's behavior and mathematics are authoritative. Start with `docs/port-plan/HANDOFF.md`, `PURE-RUST.md`, `PORTING-PROCESS.md`, `CI-AND-REGRESSIONS.md`, then the relevant canonical `plan/units/<id>.json`.

- Work from current main in an isolated checkout/branch. Check active scoped PR claims by symbols and paths; coordinate overlap before editing shared contracts.
- Implement the unit's documented steps. Preserve formulas, operation order, defaults, input guards and output meaning. Document missing continuations/dependencies explicitly.
- Every numerical/kernel port has original-versus-Rust comparison with stable inputs, case IDs and seeds. Compare geometry/state values, not only error status. Missing original results and zero cases cannot pass.
- Never feed the original outputs into the Rust execution. Never weaken tolerances, remove passing comparisons, trim seed coverage or regenerate expected values from Rust to obtain green CI.
- Every reproducible divergence gets a fixture and open item. Plan adjustments go with evidence and affected unit/decision updates; they do not silently redefine the target.
- User-visible operations use engine commands; UI is thin. Native and Wasm support, bounded work, safe error handling, undo/recovery and project compatibility are part of the feature.
- Keep application/numerical/geometry implementations Rust. Foreign kernels are reference tools only. Follow dependency purity/source provenance rules; a Rust wrapper is not a Rust kernel.
- Edit only affected unit files. Add unique finding/decision files and small follow-up tasks. Aggregate backlog/status is generated, not another manually maintained truth.
- Readiness requires evidence for the tested commit and affected callers. Public actions by an agent need authorization from its user; these instructions do not authorize messages, pushes, deployments or legal signatures.

Bootstrap gate now: `python docs/port-plan/tools/port_plan.py --root docs/port-plan check` and `python docs/port-plan/tools/port_plan.py selftest`.

Product gate after implementation: `cargo xtask ci` plus the affected differential suites and exact-base ratchet defined in CI-AND-REGRESSIONS. These commands must be implemented before product CI is enabled. Do not claim that a plan-only check proves port parity.
