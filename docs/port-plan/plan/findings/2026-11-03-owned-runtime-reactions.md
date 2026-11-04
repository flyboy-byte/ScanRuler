<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Retained deadline/execute/scene reactions — 2026-11-03

Implementation candidate: `415b4ed4828ec96346a45068550004b2e0341b2c`.
Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, app 0.9.2.

`reactions::Limited` connects the reviewed deadline state machine to the real
native task owner. Completion is reported at actual promise settlement, not
inferred from a later Future poll. Factory-return registration preserves ran-first
ties; abort listeners run before late rejection. Owned finally cleanup runs after
the synchronous event stack, and the retained task survives until the body settles.
The single owned caller future wakes queues/execute independently of late-body
ownership. Dropping an observer/ticket cannot cancel it. Outputs and faults need
not implement Clone. `retain` eagerly invokes admitted provider factories and
resumes their returned await work asynchronously, delivering one exact result to
the captured receiver. It never selects and drops a losing body.

This is a source-boundary adapter, not a general JavaScript Promise interpreter.
Providers must report completion and preserve nested await boundaries explicitly.
Real wall-clock timers and the complete guarded/interrupt/session runtime remain
open. The real Chrome run below covers existing numerical/parser library probes,
not an engine/session browser runtime.

## Independent original comparisons

All 291 previous limited scenarios and all 1,200 previous execute scenarios now
run through the retained owner with unchanged inputs, outputs, seeds and budgets.
Execute waits now arrange actual wakes instead of relying on repeated manual
polling. Original value/error identity, state/effects and pending/done behavior
remain compared. Existing changed-timeout/missing-result checks remain; execute
also rejects changed or missing output.

A new mandatory suite covers twelve independent combined full limited/inScene
traces: 3 abort policies × 2 late outcomes × 2 native ticket lifetimes. Every
fresh session queues a limited pending body, a synchronous failing command and a
successful command. It proves timeout/failure releases later replies before an
ignored-abort body finishes. It compares all five complete frames: caller and
body values/full UTF-16 errors and reason identity, document effects, timer/abort
state, factory/cleanup/reply order and unhandled rejection count. Original outputs
never enter the Rust run. Full raw proof is archived in
`evidence/2026-11-03-runtime-reactions.json.gz`.

Eight native integration tests also cover real late future effects/fault disposal,
abort dispatch reentry, factory array ties, once-only captured receivers, admission
before effects, completed non-Clone ownership, observer abandonment and reentrant
late error destruction. These host tests do not substitute for paired traces.

## Failed attempts retained

- The real wrong-value run (`SCANRULER_REACTIONS_SABOTAGE=1`) changed document 42
  to 43 and failed with exit 101; its raw mismatch and log remain. A normal rerun
  passed all twelve cases. Missing/changed output controls remain in the suite.
- The new suite initially tripped the exact inventory assertion (58 versus 57).
  Its expected totals were increased to 58 / 22,950, retaining all 168 Python
  tests and every earlier case. No assertion or gate was removed.
- A native regression reproduced a RefCell panic when a late fault's destructor
  inspected the runtime. Unused fault payloads now leave the interior deadline
  borrow before disposal; the regression passes. This changes Rust resource
  ownership, not the source deadline math/winner rules.
- A superseded pre-fix run was rejected by raw candidate identity checking after
  the fix advanced HEAD. It is not counted as a clean candidate result.

## Final validation

On the clean immutable implementation candidate above:

- `fnm exec --using=22 cargo xtask ci` passed all 168 Python tests, formatting,
  Clippy, workspace tests (including the eight host regressions and twelve-case
  native matrix), and native/Wasm compilation.
- `fnm exec --using=22 cargo xtask oracle` passed every earlier suite plus the new
  required suite: 58 registered suites, 22,950 case identities including refusals.
  All earlier numerical/parser/engine cases and budgets remain unchanged.
- `ci_report.py evidence` validated clean candidate/reference/raw identity and
  completeness for every registered case. One invocation mistyped the SHA and
  was correctly rejected; the corrected invocation used the verified full SHA.
- `cargo xtask wasm-probe`, the complete browser contract tests, and the real
  Chrome runner passed: 4,431 cases (1,952 successful values, 2,479 source refusals)
  plus 36 negative protocol checks. This retains the whole earlier browser roster.

The exact-count guard, wrong-value canary and late-fault red regressions were
retained. The final candidate stayed clean throughout the successful native and
Chrome runs. Final metadata/docs reference this tested implementation; they do
not retag historical reports as fresh application acceptance.

| Retained log/proof | SHA-256 |
| --- | --- |
| `runtime-queue-canary-red.log` | `09eff68d4444c7c0fb70f6227e24abdde9c2a7230f8c59b7e8e2c6f7e3a29c3a` |
| `runtime-late-drop-red.log` | `2adb9bfa88e534b5fcd5d4bc78115d8e0f211421a15b9bd44fe61c65fcffb5ce` |
| `runtime-final-ci.log` | `c679b6c79a7e298068b0f13215f5f21148158816af749ddd145ede66e299723e` |
| `runtime-final2-evidence.log` | `72e3e9610ddededcd92512c0dc049d27bba6a5650bc218771bd06eabb71eb708` |
| `runtime-final3-ci.log` | `c763e8abbbb9b648ffc2a7858bd0555705b3f8b9528a0c781995fde209eb0d06` |
| `runtime-final3-oracle.log` | `06e804357338619ccb96d4d2d1f2ae4dba267485380c7cd27f76955b83459fac` |
| `runtime-final3-evidence-corrected.log` | `86d2f09f9e00d0846b7e18bd0e7e6b77eaf819f8e00e128e347ba535ee73a56a` |
| `runtime-final3-browser-contract.log` | `57884c7b791727a15cd8d6d67c5ac6d6d946025151d97f308753fadcba2166f2` |
| `runtime-final3-browser.log` | `04f3ed167098bd605f50bfa105f1578e8ecbf819ca760e1a64874cc4bc13bd09` |
| `2026-11-03-runtime-reactions.json.gz` | `84436b7e38e1fdd3ef529a2509677b21f02927fe36e063af60f1d729be758ac6` |
| `browser-wasm/report.json` (local raw artifact) | `d73afa26494dec4e2795c4fd098223cb923d775c6582c71c830801da21e485e9` |


Full local logs remain under `.port-run-artifacts/` in the original checkout.
The red canary, count-guard failure, destructor regression and superseded identity
failure have distinct filenames and were not overwritten with green results.

## M3 still lacks

The CLI still prints its banner. `engine.runtime.guarded` is the next canonical
unit: run actual registered command bodies with history/activity/current/error
normalization and late-progress semantics. Then wire one real scan/session and
import/plane family, undo and original-compatible project save. No session or M3
acceptance is claimed. Upstream 0.9.4 is not merged/rebaselined; D4/D5 remain
requirements for executor/MCP: every queued failure is answered, only to its
requesting receiver. Captured native receivers are tested here; MCP transport is
not implemented. No TypeScript, dependency, tolerance, seed or timing threshold
changed. No PR was merged or main pushed.
