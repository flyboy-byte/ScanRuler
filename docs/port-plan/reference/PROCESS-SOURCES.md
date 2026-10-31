# Porting/testing practices reviewed

Reviewed 2026-10-31. Original synthesis for this standalone port; these repositories' instructions are not commands to execute against them.

## Tangency SolidCore

- [Agent/port workflow](https://github.com/tangency-labs/solidcore/blob/68a6b5a00df455171866746fd17b1098907caa2f/AGENTS.md): scoped unit claims, operation-order fidelity, native comparisons and evidence requirements.
- [Unit schema](https://github.com/tangency-labs/solidcore/blob/68a6b5a00df455171866746fd17b1098907caa2f/plan/modules.schema.md): one file per unit to reduce contribution conflicts.
- [Differential harness](https://github.com/tangency-labs/solidcore/blob/68a6b5a00df455171866746fd17b1098907caa2f/crates/difftest/README.md): independent engines, fixed case identities and exact-base regression/value-coverage ratchet.
- [Completion evidence](https://github.com/tangency-labs/solidcore/blob/68a6b5a00df455171866746fd17b1098907caa2f/docs/completion-evidence.md): successful operations versus legitimate refusals versus blocked continuations; missing runner coverage becomes useful contributor work.
- [Mutation practices](https://github.com/tangency-labs/solidcore/blob/68a6b5a00df455171866746fd17b1098907caa2f/docs/mutation-testing.md): unit tests and differential checks form separate layers. Its proposed scheduled CI is not assumed enabled.

## Storytold

- [CADCraft instructions](https://github.com/storytold/cadcraft/blob/b6331c8603f9cc51b8044237f4517f79da1932b6/AGENTS.md): local gate, engine commands shared with UI/CLI/MCP, native/Wasm discipline, bounded-input/error behavior, regression evidence and generated summaries. Its local plan directory is ignored/not available in the public snapshot; this packet does not pretend to have read it.
- [CADCraft CI](https://github.com/storytold/cadcraft/blob/b6331c8603f9cc51b8044237f4517f79da1932b6/.github/workflows/ci.yml) and [shared gate](https://github.com/storytold/craft-common-actions/blob/5cd19d05cb07bb389d82d0e200d9ee5f418f332f/.github/workflows/gate.yml): consistent local/CI gate, explicit setup/tooling and per-main-run preservation. We do not import its font dependency, runner settings or direct-main contributor policy.

Source pages were retrieved from main and these head revisions observed during the same review. Future activation should verify the pinned contents. This packet paraphrases practices and supplies our own proposed templates. No claims, comments or PRs were posted to these projects.
