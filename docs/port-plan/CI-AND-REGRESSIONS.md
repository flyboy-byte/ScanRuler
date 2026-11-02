# CI and regression contracts

Updated 2026-11-02. The frozen original is authoritative. No workflow change in this integration alters a numerical formula, tolerance, seed, source test or timing threshold.

## Executable gates

| Gate / check | Executes | What success means |
| --- | --- | --- |
| `Plan and status integrity / plan` | canonical validation, generated backlog/status freshness, negative tooling tests | Metadata/inventory is consistent, not numerical parity |
| `Rust port validation / fast` | `cargo xtask ci` | Native ordinary tests, fmt/lints, plan/status and Wasm **compile** pass |
| `Rust port validation / native-oracle` | `cargo xtask oracle`, then raw evidence inventory | Every required native source suite passed and its full current-case artifacts exist at the exact checkout SHA |
| `Rust port validation / catalog-contracts` | original headless/browser snapshot and negative validators | Immutable captured metadata remains valid, not command execution or Rust parity |
| `Rust port validation / browser-wasm` | actual Chrome running three Rust library corpora plus hostile codec/memory probes | All 114 source/native/browser cases passed; not UI or all-unit browser parity |
| `Rust port validation / gate` | all four Rust lanes, even if another fails/skips | All required lanes succeeded; existing check ID `gate` retained |
| `CI / source-tests` | plugin/unit boundaries, complete `npm test` | Original unit checks pass, including unchanged timing limits |
| `CI / source-mcp` | complete separate MCP suite | Original agent tests pass independently of unit-test timing |
| `CI / source-browser` | original production build and complete `e2e:ci` | Original browser regression/smoke checks pass independently of unit-test timing |
| `CI / test-and-build` | all three original lanes | Every original lane succeeded; existing check ID retained |
| Frozen original catalog workflows | real clean-pinned headless/browser captures, full metadata comparison | Original catalog evidence reproducible in core/all contexts |
| License check | unchanged root/MCP allowlists | Existing npm licensing policy passes; not the Rust P35 purity audit |

`cargo xtask diff`, `diff-ci`, the general paired report protocol and exact-base numerical coverage ratchet **remain unbuilt**. Wider browser-Wasm coverage also remains incomplete. Bounded actual browser execution is implemented in oracle/BROWSER-WASM.md; the remaining planned contracts below are not implemented CI claims. No check here authorizes merging or sets branch protection/repository settings.

## Native evidence and status

`ci/native-suites.json` maps all required Rust entrypoints to canonical units and raw directories. The reporter verifies the manifest agrees with Rust discovery and all numerical unit inventories. It rejects missing/duplicate suites, stale candidate identity, missing/extra case IDs and absent input/original/Rust files, including compared progress/count sidecars on successful cases. The two STL adapters retain their existing explicitly enumerated refusal/error artifacts; legitimate zero-length values are permitted but missing files are not.

ASCII PLY adds 60 exact parser cases (46 outputs / 14 source refusals), and binary
PLY adds 168 (152 outputs / 16 refusals), bringing the native inventory to 20
suites / 914 identities. The adapter uses the pinned
original TypeScript compiler after lock-preserving `npm ci`, solely as reference
tooling. The binary body/full dispatcher are implemented; actual browser-Wasm PLY
execution remains open. ASCII artifacts require full-dispatcher value/progress
sidecars in addition to the original ASCII-entrypoint comparisons.

Numerical comparisons remain in the existing Rust tests with their original fixed budgets and cases. The inventory reporter hashes raw files; it does **not** reinterpret values, declare a refusal successful geometry, compare against the exact PR base, or replace P36/P37. Canonical case edits and changed input/output hashes still need review; there is no automatic accepted-case ratchet yet.

Artifacts are SHA-named and retained 30 days: Rust fast logs/environment, every `*-diff` raw directory plus native manifest/log, original unit/MCP logs and successful/failing browser diagnostics. Catalog snapshots remain committed and hashed. Main runs use unique run IDs and are never cancelled by a newer commit. Only superseded PR runs are cancelled. Explicit Bash enables pipefail so log capture cannot hide a failed command. No build-output/golden cache is introduced.

Generated [STATUS.md](STATUS.md) and `status.json` describe canonical declared state, not the latest CI result. `AUDIT-QUEUE.md` records observed run outcomes. The generated backlog and status must be refreshed after canonical changes. Python remains temporary bootstrap tooling until P39's Rust replacement; it is not shipped application logic.

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

## Local reproduction and review

```bash
python3 docs/port-plan/tools/port_plan.py --root docs/port-plan render
python3 docs/port-plan/tools/ci_report.py status
cd rust
cargo xtask ci
cargo xtask oracle
cd ..
python3 docs/port-plan/tools/ci_report.py evidence --candidate "$(git rev-parse HEAD)"
node --test docs/port-plan/oracle/catalog-contract.test.mjs docs/port-plan/oracle/browser-catalog-contract.test.mjs
```

Use Node 22 for CI reproduction and real original captures. Local native math runs previously used Node 20.20.2; those records stay explicit. System Rust is the host policy; hosted stable provisioning is not a request to install rustup locally. Original `npm test`, MCP, build and browser commands are still required and retain all existing acceptance thresholds. Known source timing failures remain red until a real passing run or separately reviewed source change; never suppress them in a port PR.

The disabled `repo-bootstrap` product-gate example describes the future generic ratchet; it is not the active workflow. Release deployment remains unchanged and no Rust release workflow is activated. Branch protection settings need the owner's separate review after these check names are proven on hosted CI.

The browser-wasm lane depends on the successful native-oracle artifact, checks exact candidate/reference/input hashes, builds its own dependency-free Rust module and uses real Chrome. Source/native outputs never cross into the browser payload; only fixture inputs do. Safe scalar-buffer functions retain unsafe_code=forbid. The private linker exports are not the product ABI. Exact corpus/budgets/limits and actual evidence are recorded in oracle/BROWSER-WASM.md; a Node-Wasm smoke cannot satisfy this lane. The current native inventory is generated in STATUS.md; browser coverage remains a separate 114-case subset, not additional native numerical cases.
