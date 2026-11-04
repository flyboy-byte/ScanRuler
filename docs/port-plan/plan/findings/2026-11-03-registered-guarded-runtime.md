<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Registered guarded runtime — 2026-11-03

Implementation candidate: `2b6710a9368c25d087a062bb0f6091287642fbd7`. Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (app 0.9.2).
PR #99 follows merged main `a3681cb` and owns the shared oracle/module/plan files.

Actual registry definitions and command callbacks now execute over the retained
owner, mutation/scene queues, execute/settle/plugin stopping, guarded/interrupt,
history capture, labels, activity/current/progress, deadline and error normalization.
The Host owns the sole business document; the runtime owns metadata and immutable
history snapshots. Callbacks use weak runtime handles, and dropping the session
releases pending owned producers without a self-owning task cycle.

## Independent evidence

The full pinned original graph and Rust independently construct fresh registered
commands from 1,972 control profiles. No source outputs enter Rust inputs. All
profiles compare full UTF-16 values/errors and reason identity, document/captures,
activity/current/deferred, actual history patches/labels/workspace/group/flags,
timers and ordered effects, stale/paused state, nested replies and plugin done
observations. Previous deadline, execute, parser and numerical cases remain.

The product covers guarded/interrupt, history, busy, immediate/pending/late
outcomes, labels and deferred execution. Append-only profiles exercise listener
and capture failures, string/stale coercion, nested shared history, reentrant
readout, epoch invalidation, strict discards, scene dispatch, synchronous factory
throw precedence and duplicate abort-listener identity. The independent fixed
clock input is 1234; the source shell workspace is `elements`.

## Failed runs retained

- Initial source comparison found 357 differences from a native fixture workspace
  initialized to `measure`; source inspection established `elements`. Only the
  independently constructed native fixture initialization changed.
- Other/string conversion with reentrant page-state readout reproduced a native
  RefCell panic. Conversion now occurs outside the stale-state borrow; capture
  callbacks likewise execute outside the history recorder borrow.
- One profile exposed synchronous label-conversion failure before the first await.
  Catch/finally now runs at that boundary, including interrupt ordering.
- Four profiles exposed duplicate invocation of the same abort callback. Listener
  registration now deduplicates function identity before enforcing the host cap.
- A full wrong-value run changed real resulting state from 1 to 9 and exited 101.
  The normal 1,972-case rerun had zero divergences. Changed and missing output
  checks also remain in every case. No input/seed/tolerance/gate was removed.

Full passing raw inputs/original/Rust/stderr, source/generator hashes and the
failed logs above are retained in `evidence/2026-11-03-registered-guarded.json.gz`.
The ordinary debug runs were dirty and are not labeled clean candidate evidence.

## Validation on the clean implementation

- `cargo xtask ci`: formatting, Clippy, complete plan/Python/workspace tests and
  native/Wasm compilation passed.
- `cargo xtask oracle`: all 59 mandatory suites / 24,922 case identities passed;
  this counts declared cases including refusals, not feature completion.
- `ci_report.py evidence`: candidate/reference/raw completeness validated for all
  59 suites at the exact candidate above.
- `cargo xtask wasm-probe`, complete browser-contract tests and actual Chrome
  runner passed, retaining all 4,431 cases and 36 negative protocol checks.
  These remain library probes, not a browser session runtime.
- Native shutdown regression proves the retained producer is dropped once and
  weak owner/runtime/host handles expire when the session is dropped.

Final metadata/docs point to the immutable tested implementation above. Hosted
checks apply to the final PR head and must be green before ready status.

## Explicit remaining scope

This is a typed native source-provider boundary, not a general JS Promise/DOM
interpreter. Providers preserve actual body completion/await boundaries;
arbitrary thenable/proxy/getter behavior and browser abort-listener exception
reporting remain outside it. Plugin done is observed at quiescent trace frames;
no universal cross-provider microtask scheduler is claimed. Native real platform
timers/ingress remain a Host continuation. This is fixed-corpus verification,
not fuzzed/coverage-reviewed/accepted status.

Read-only registration is explicitly refused until `engine.runtime.readonly`
implements the original readOnly-before-interrupt branch and remembered job
outcomes. Actual scan and measurement families are not the fixture participant.
The CLI still prints its banner: M3 needs real session state, an import command,
plane-fit command, global undo/apply and original-compatible project save.

Queued calls capture the requesting receiver and deliver failure too (D4/D5);
MCP transport remains unimplemented. No original TypeScript, dependency, source
pin, seed, tolerance or 1500 ms threshold changed. No merge/main push occurred.

| Retained artifact | SHA-256 |
| --- | --- |
| `guarded-source-compare-first.log` | `95e25cb653fb7005e4026c3297c627d4b25dc52d193c09bc619fc4c124b762df` |
| `guarded-conversion-reentry-red.log` | `b87d3b9a94f960f096647362bdb86734b1ecd90039b5eb456757bb8b174c9c11` |
| `guarded-label-order-red.log` | `01fb5151fe1128ac6adb01aac5667c67cd85959f272a865398be64be3d7cad30` |
| `guarded-abort-identity-red.log` | `87ce22c5472afbebaba0d9f2210b8ef16fd4c61797d4ec07f93e3f602d0f3d43` |
| `guarded-full-canary-red.log` | `4e05f26da68debf39d53e59065d22da3429af679c08ec3b56d932aee617ed608` |
| `guarded-full-canary-exit.txt` | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |
| `guarded-committed-ci.log` | `e26b9ef2bf09f9df1430b26b018d0fd7f2d57d75abf4eb1d849d381a2919add2` |
| `guarded-committed-oracle.log` | `f387e743493407f94525417833f5111f691ac3e7a60f496b962e7f5f9fb335a7` |
| `guarded-committed-evidence.log` | `aa997a79416a859481efe083593a7fc622fd57719ccf6504b60b15ef214233cf` |
| `guarded-committed-wasm.log` | `9176e670bc414a8c18a8be5fb187e01e8de9500e295df234d0a20c351df8b583` |
| `guarded-committed-browser-contract.log` | `59e09e733345f4f7164e7e6b27ad8849794580ad92393c672cc7cc06b44fe1f1` |
| `guarded-committed-browser.log` | `04f3ed167098bd605f50bfa105f1578e8ecbf819ca760e1a64874cc4bc13bd09` |
| `2026-11-03-registered-guarded.json.gz` | `b214f32a101c81d7caf05ffb393b95787a5059c388837837fd79063b860b2736` |
