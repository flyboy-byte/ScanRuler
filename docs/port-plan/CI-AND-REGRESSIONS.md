# Our CI and regression system

## Contract

The default reference is Stefan's pinned original code. CI proves that Rust preserves it and that previously established behavior stays covered. Rust compilation, test counts, status-only agreement or a beautiful screenshot cannot replace differential evidence. Source algorithm changes and source rebaselines are separate reviewed changes.

Current fast-gate interface: `cd rust && cargo xtask ci` runs the pin/plan checks, formatting, clippy, workspace tests and wasm32 library compilation. It exists in `rust/xtask/src/main.rs` and is run by `.github/workflows/rust-port.yml`. It does not run differential comparisons or browser-Wasm execution.

Remaining proposed comparison interface: `cargo xtask diff --unit <id>` for targeted comparison, and `cargo xtask diff --baseline <report-dir>` for the exact-base ratchet. The `diff` and `diff-ci` commands **do not exist yet**. P36–P39 establish the harness, contracts, CI and maintained plan process. Bootstrap templates fail if invoked before those commands exist; they must not report fake passing builds.

## Paired-runner protocol

Each case has immutable ID, unit/operation, versioned input payload or asset hash, seed, operation sequence and declared observable output paths. A coordinator feeds independent copies to the original adapter and Rust adapter. Source measurement adapters can use a pure-core Node harness or the real browser; the original headless app is limited. Rust runs native and browser-Wasm where required. The adapters emit versioned data, not guessed summaries.

Capture reference SHA, candidate SHA, lock/toolchain/backend versions, fixture manifest digest, generator/seed schedule, comparator version and environment. Result states distinguish pass, mismatch, deferred/unported, unsupported, skipped, reference-failure, harness-error, timeout and crash. Failures/skips never become passes by omission. Preserve raw original/Rust outputs and comparable key sets/lengths along with computed differences.

Compare the results that matter for the operation: fits plus residuals/selection, geometry plus topology/volume/bounds, mutation plus state/history, formats plus reopened meaning, UI plus picked identities and displayed numbers. Exact comparisons apply to discrete contracts; per-field numerical tolerances are set from source tests/representation before seeing a Rust mismatch. No copied universal SolidCore tolerance. Normalizations for equivalent geometry are versioned and limited to representation differences; they must not erase real divergence.

## Required report ratchet

For a PR, compare candidate reports with validated reports for the exact target base commit, same pinned reference, input manifest, seed schedule and comparator contract. Never use an arbitrary old successful artifact. Missing/expired/incompatible artifacts require regenerating that exact base or a blocked comparison, not waived coverage.

Every previously passing case must remain, remain passing and retain previously compared values. Dropping a unit, input, output key or successful case, shrinking seeds or converting a pass to skip/deferred/known-difference fails. New incomplete units can be explicitly partial; they cannot hide lost coverage from existing accepted units. Error-only checks do not substitute for successful value/state coverage. Track operation-specific successful comparisons rather than choosing an arbitrary aggregate percentage as proof of completeness.

Golden regeneration is never “bless whatever Rust produced.” Reference outputs come from the pinned source run. A deliberate new source pin requires side-by-side old/new source evidence and migration review. Review changes to comparators, tolerances, generators and outputs as test-infrastructure changes; these can otherwise make broken code look correct.

## Gate tiers

| Tier | When | Contents / required result |
| --- | --- | --- |
| Planning bootstrap | Every PR before code exists | Live unit/dependency/schema/claim checks, plan-checker self-tests, artifact validity; named explicitly as plan-only |
| Fast product gate | Every implementation PR | fmt, clippy, unit/regression tests, layers/assets/dependency checks, native compile/tests, wasm check, targeted diff and changed-caller regressions |
| Merge gate | Before merge / merged-result validation | Complete accepted-unit regression corpus and exact-base value-coverage ratchet; browser project/history/MCP checks; no filtered baseline comparison |
| Main validation | Each main commit | Landed-commit reports and baseline publication keyed by commit and contracts; never cancel a main run because another merge arrived |
| Scheduled/manual depth | Budgeted schedule | Larger seeded corpus, fuzzing, mutation survivors, large meshes/performance and wider platforms/device checks; retained failures become ordinary regressions |
| Release | Explicit release work | Full native/web/function/purity/deployment matrices and installer/legacy file acceptance |

Superseded PR runs may be cancelled. Main runs are keyed to commit. Cache dependencies, not unvalidated golden results; key by toolchain/locks/feature/profile/reference identity. Parallel test units need process isolation and bounded machine budgets. Large immutable assets can use a separate corpus store with content hashes, not unchecked mutable URLs.

## Test the testing system

The paired harness must reject a deliberately altered Rust value, a missing key, an empty suite, a skipped reference, a stale input hash and changed case identity. Perturb one output or operation at a time and demonstrate the comparator fails. Mutation testing has two layers: ordinary Rust tests first, differential tests second. Mutants missed by both indicate a concrete gap; minimize the input and add assertions/observable output. Equivalent or uncompilable mutations are classified, not counted as meaningful catches.

Mutation checks are initially scheduled/targeted because cost is unknown. Do not inherit SolidCore's timing numbers or infrastructure. Deterministic seed prefixes preserve reproducers as campaigns grow. Capture timeout/crash cases with enough state to rerun them alone. A failed mutation or fuzzer finding does not authorize changing the source target.

## Activation kit and honest status

`repo-bootstrap/` contains original contributor/agent instructions, claim/PR/issue templates, a plan-only GitHub workflow and a disabled product-gate example. Install them deliberately in the fork and review/update action/tool pins and paths. Do not blindly invoke Storytold's shared workflow: it has suite-specific setup/fonts and policies. We adopt the pattern, not an unreviewed dependency on their infrastructure.

The planning workflow can run `tools/port_plan.py` now. The product workflow stays disabled until the Rust workspace and xtask commands genuinely exist, then fails closed if required files/commands or baseline reports are missing. A green plan-only check must never be labelled port CI or numerical parity. No remote workflow, repository setting, CI secret, branch rule or public issue was created during handoff preparation.
