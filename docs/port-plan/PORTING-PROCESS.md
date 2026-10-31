# Contributor process and living port plan

## Reference policy

Stefan's pinned ScanRuler code, mathematics, defaults, refusals and observable results are the authoritative behavior reference for this port. The user explicitly trusts that implementation; the project is a faithful Rust port, not a redesign or an audit of the original. Preserve formulas, operation/iteration order, clipping rules and rounding behavior before considering optimizations. Do not use an algebraically attractive rewrite, different solver or Rust FMA simply because it looks cleaner.

Run the original and Rust implementation on equivalent independent copies of the same input. Compare outputs; never feed reference results, shapes, intermediate state or expected answers into the Rust implementation. A discrepancy is an open port divergence until explained. Analytic fixtures and metamorphic checks diagnose the port or its harness and broaden coverage; they do not authorize replacing Stefan's output. If an apparent source problem is found, record the reproduction separately and retain the pinned reference contract. A reference update or intentional behavior change requires a separate reviewed decision, never a relaxed expectation in the fixing PR.

## Single source of work status

`plan/units/<id>.json` is authoritative for each work item. The initial P00–P39 files are roadmap workstreams; add smaller dotted child IDs for bounded tasks such as `fit.plane.residuals` with `parent: "P07"`. A child records hard/soft dependencies, source paths/symbols, intended Rust paths, claimed edit boundary, baseline/case IDs, acceptance, evidence, open items and owner/PR. Do not claim a whole kernel or workflow when working on one function group.

`backlog.json` is a generated snapshot/index, not a second manual status system. Generate it with `python tools/port_plan.py --root . render`; in a bootstrapped repo use `--root docs/port-plan`. Contributors edit only affected unit files. Maintainers regenerate aggregate summaries in integration batches. Reports, source/command/file inventories and documentation have different purposes: generated metrics are evidence, prose explains intent, unit files own implementation state. A green test suite does not itself mark all work complete.

Lifecycle: planned → specified → baseline-captured → implemented → compared → integrated → accepted. A blocked item retains its last achieved state and describes `open_items`/blocker rather than claiming completed code. Accepted units require a tested implementation commit, actual evidence, resolved open items and accepted hard dependencies. A unit with missing successful branches, deferred calls, unknown platform coverage or status-only comparisons cannot be accepted for that scope.

## Start a contribution

1. Read HANDOFF, PURE-RUST, the relevant architecture/validation sections, the unit file and its open items. Verify current main/source baseline; check open PRs by symbols and files as well as task number.
2. Reproduce the existing behavior and port gap before editing. Identify the source's successful path, refusals, defaults and boundary cases. Use a named case, not a screenshot-only guess.
3. Prepare a scoped draft PR claim: unit/symbols, current base SHA, branch, intended edit boundary, dependencies, evidence to produce and independent follow-ups. The user/maintainer authorizes actual public actions for sessions acting on their behalf; this document is not permission to post automatically. A contributor submitting their own PR follows the repository's published process.
4. Use an isolated branch/checkout. Coordinate before changing a shared schema/registry/arena; no competing version of a live claimed function. The earliest active overlapping scoped claim has priority; disjoint work is welcome. While actively implementing, update at least daily. If inactive for 72 hours, a maintainer may reassign after a check; release independent scope when blocked rather than reserving a whole subsystem.

## Port one bounded unit

Specify contract and fixed inputs → capture reference → implement Rust faithfully → compare native and Wasm → exercise successful and refusal paths → integrate callers → run affected regressions → update the unit/evidence → review.

Port surrounding continuations even when a dependency is absent. An explicit unported result belongs at the missing call boundary; an early return silently omitting the rest of a function is not a complete body. Document any minimal co-ported type/function under its owning unit so another person extends it instead of duplicating it. When wiring becomes possible, run both the newly reached branch and the affected callers.

Use a PR operation table: source symbols, what is implemented, successful comparison case IDs/observable values, remaining gaps. Keep legitimate negative tests; add real successful geometry/state comparisons. Do not count “both returned error,” zero cases or all-deferred cases as geometry parity. Turn every reproducible divergence into a small checked-in fixture with stable ID and provenance. Keep large licensed corpora in a separately versioned store if needed; manifests/hashes remain in Git.

## Refine the plan while working

Before marking ready, update the unit's open items and add narrowly scoped follow-ups for newly found work. Record evidence-bearing findings under `plan/findings/<unique-id>.md` and decisions under `plan/decisions/<unique-id>.md`; avoid everyone appending to one large file. Each decision explains the old/new approach, evidence, compatibility impact, affected units and review owner. Preserve resolved items/history instead of erasing discoveries.

Implementation that contradicts the architecture or documented steps must either be corrected or accompanied by a reviewed plan change. “The code now does this” is insufficient justification. Interface changes identify downstream unit owners and rerun their checks. Refine steps as evidence improves; keep the pure-Rust endpoint, pinned-reference identity and acceptance strength fixed unless the owner explicitly changes them.

Use issues for discovery/questions and PRs for scoped ownership/review. Keep unit IDs in both. Generated reports produce human-readable progress and next-ready tasks; manually curated milestone prose names priorities and material blockers, not duplicate counts.

## Review and integration

Run the same local gate CI runs, plus unit-specific comparisons. Attach source/Rust SHAs, fixture/case identities, seeds, comparator version, tolerances and native/Wasm reports. Show actual UI evidence for visible changes. A review approval applies to a commit; later changes need fresh checks/review. Never merge one's own contributor PR under the proposed default process. Maintainer sets actual branch protection and merge policy when the repo is activated.

Integration runs preserve every main check and exact-base comparison. Newly passing cases cannot disappear or become deferred/known-difference. The next main run validates the landed commit. Accepted status is evidence-backed, not a label set because a PR exists. Handoff lists current commit/branch, completed scope, exact unfinished items, case/run commands, failures/skips and the first next action.

## Practices adapted, not inherited as commands

SolidCore: per-unit files, scoped claims, faithful numeric porting, independent paired engines, completion evidence, exact-base coverage ratchet and mutation checks. Storytold: same local/CI gate, command-centered thin UI, layer/native/Wasm checks, regression tests for failures, and generated/shared status views. Their repository-specific licenses, private plans, runner limits, commercial-app policies, direct-main workflow and infrastructure are not imported wholesale. Our comparison target is ScanRuler and our contribution workflow is our own.
