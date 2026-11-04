<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Prospective combined scan integration: local validation — 2026-11-04

**This is a tested prospective tree, not merged main or a usable M3.**
Main remains `7a6ae9a`. Local/review-only branch `codex/nov04-combined-validation`
at `baeba66e74def1eeca3bbfe2b61ff4f8d5772001` combines #100 plus the queued formatter, real prepared-scan document
and prepared render branches. It is pushed for reproducibility, not proposed as
a multi-unit feature PR. The frozen reference remains `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`
(app 0.9.2); upstream 0.9.4 was not merged or rebaselined.

## Result on the exact clean candidate

- `cargo xtask ci` passed, including all tooling tests, exact inventory checks,
  formatting/lints, ordinary native tests and Wasm compilation.
- `cargo xtask oracle` passed **63 required suites / 35,863 declared identities**,
  including every earlier refusal/case/seed. Raw inventory validation passed on
  the exact candidate. No previous gate or comparison disappeared.
- The combined Number text (6,241), scan document (800) and render (1,309) raw
  outputs were additionally checked byte-identical to the independent originals.
  Each unit's complete actual wrong-value failure proof remains in its archive.
- Wasm probe, all 59 browser/catalog contract tests, and real Chrome passed
  **4,431 value/state comparisons + 36 negative protocol checks**.
- Original `src`, tests, plugins, MCP and dependency files have no diff against
  the pin. No dependency, tolerance, seed, input, source threshold or 1500 ms
  assertion was changed. None is accepted and M3 is still not runnable.

The compressed evidence contains every native identity/file hash, the complete
browser report, full gate logs and relevant source hashes:
`evidence/2026-11-04-prospective-combined-validation.json.gz` (SHA-256 `4c1673883358bd5821ca92e5d459fd9bfa3c0a2008b8f2b0bb635e85e003a0d9`). Complete raw combined runs
remain in the validation checkout's `rust/target/*-diff/`; individual new-unit
archives retain full passing and red raw values, not only hashes. The new public
feature heads must still run hosted checks after their bounded PRs open. This
snapshot does not substitute for that CI or implement the missing P37 ratchet.

## Review order and shared-file ownership

#100 and independent docs-only #101 are ready, each with all eleven required
checks green. #100 remains the only OPEN PR owning shared source/CI/plan files.
No PR was merged, no main push occurred and no stack exceeds three feature PRs.

1. Review #100, registered read-only/job execution, final head `8789795`.
2. Open the bounded `codex/number-shortest` PR (head `31badd7`, tested code
   `dfb6d63`). It includes the complete unchanged-budget parser campaign refresh.
3. Open independent main-based `codex/scan-render` (head `ff3f897`, tested code
   `7f2a6de`): full wire slots, fan/crease splitting and prepared render arrays,
   including a 160,000-vertex default-budget refusal. It is not stacked on model.
4. Open `codex/session-scan` last (code/proof head `8c87f16`, later plan-only
   refinement `b743b45`, tested implementation `63b5c1e`). It carries real owned
   mesh/source/store state and the complete prepared-scan commit consumer. It
   includes the formatter through ordinary merge and the sole extended session
   changelog entry. Source-audit #101 is independent of all shared-file work.

These are pushed queued branches, not already-open feature PRs. Final exact-head
hosted checks and owner review are still required. The prospective merge exists
to catch combination failures before integration, not to bypass unit review.

## Conflicts resolved without losing evidence

The source implementations combined without editing either body. Canonical suite
and xtask lists keep their full union (63), and the inventory assertion ratchets to
35,863. The render unit takes the tested implemented record over model's earlier
planned placeholder. Canonical lane ownership preserves both branches.

Generated status/backlog were regenerated with `port_plan.py render` and
`ci_report.py status`, never hand-merged. One initial regeneration failed because
backlog conflict markers were not valid metadata input; the complete existing
backlog was used only to bootstrap metadata, then tools rebuilt all aggregates.
The final clean candidate passed all gates. Keep the two source-audit wordings
consistent when #101 lands (proposed versus canonical children); that is source
prose, not a generated-file merge.

## First next implementation

On the combined plan, `next_task.py --offline -n 1` selects
**engine.session.scan.import**, now startable. The model branch's subsequent
plan-only refinement records that the render continuation is available. Port
actual worker preparation/staging/versioning and real scan command execution:

- `imports.prepareScan`: retain original source bytes/units/companions, copy the
  worker buffer, prepare actual parser/graph/render assets, and own view lifetime.
- `scanImport.openScan`: keep source unit selection, import queue/status behavior,
  worker commit before document commit, and the exact view-dispose/discard finally.
- `workerClient`/`meshWorker`: preserve staged IDs, model versions, stale/failure
  rules, scaling and default crease-on. A parser alone or forced crease-off branch
  is not the complete worker load path.

Then bind actual plane fitting, global history apply/undo, compatible projects and
native CLI. Existing typed scene/view/history providers are explicit boundaries,
not a renderer or production command session. UI unknown setting coercion and
MCP transport remain open; queued failures must answer only their requesting
connection as required by ledger D4/D5.
