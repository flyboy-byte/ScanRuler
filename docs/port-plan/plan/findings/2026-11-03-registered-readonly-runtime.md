<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Registered read-only and job dispatch — 2026-11-03

Implementation: `207e00b8943eb80ed4afb67fd9a8ed2e312f6f93`. Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (app 0.9.2).
PR #100 follows merged main `7a6ae9a`; it is the sole open owner of shared
oracle/module/plan files. No main push or merge was performed.

The actual runtime now routes read-only before interrupt, starts settlers before
scene admission, keeps the original null/ready distinction, starts the deadline
inside the scene turn and retains settling/body work beyond caller timeout.
Read-only progress reaches its caller without publishing activity/history/current.
Deferred context is present for read-only and mutation, absent for interrupt.
The read-only catch and rejected-promise adoption retain their owned continuation
boundaries before replying, including scene release and batched timer cleanup.

`submit_job` wraps the already-started actual run. Empty/absent jobs bypass
remembering and its clock. Other jobs attach the reviewed recent record/result
components, including shallow readout, 100-record eviction, full errors/omissions
and once-only receivers. A synchronous remember failure returns its distinct
submission error without cancelling work; an otherwise unobserved failure and
any failed recent reaction reach the mandatory Host observer-fault sink. Observer
failure never replaces the caller's actual command result. No global request-ID
routing was added. Date.now and new Date constructor timestamps are separate
providers, preserving their separate source fault sites.

## Independent original evidence

2,591 deterministic original-first profiles compare five full frames each:
owned values (UTF-16, bytes, undefined and JSON results), caller errors and typed/
signal identity, document/captures, activity/current/history, timers and complete
ordered effects, full/filtered recent records/results/omissions, stale state and
detached failures. Original outputs never enter Rust inputs. Every prior case is
retained; append-only profiles cover shared scene contention, 101 simultaneous
calls with per-receiver index/value observations, eviction, negative durations,
lone-surrogate jobs, clock/Date/reaction faults and failed asynchronous settlers.
Native settlement uses the actual reviewed live callback registry.

The full passing raw input/original/Rust/stderr profile, source/generator hashes,
oracle provenance, failed logs and minimized raw counterexamples are archived
in `evidence/2026-11-03-registered-readonly.json.gz`.

## Failed attempts retained

- The initial 1,968 profiles matched. Broader shared-scene/burst profiles then
  found 99 ordering mismatches: receivers ran before the next scene turn or before
  other already-queued timer cleanup. The source read-only catch continuation
  now delivers on its own reaction boundary.
- The first continuation fix left 12 mismatches when the handler returned
  Promise.reject(asCommandError(e)). Its rejected-promise adoption is preserved
  separately from successful catch passthrough. All original comparisons remain.
- Four minimized appended clock-site profiles exposed two mismatches caused by
  using Date.now for the source's independent new Date timestamp. Host now has a
  separate stale timestamp provider; all 2,591 profiles pass after the fix.
- The complete wrong-state canary changes actual native document increments from
  1 to 9 and fails with exit 101 and positive divergences. Changed and missing
  output controls remain in every profile. Raw canary reproduction is archived.
- One canary invocation had a wrong manifest path and returned 101 without running
  the comparison. It is retained separately and is **not** canary evidence. The
  corrected invocation checks both exit 101 and positive full-corpus divergences.
- A sandboxed Node builder could not spawn Git (EPERM); an authorized rerun passed.
  Compilation feedback and intermediate logs remain locally, never relabeled green.

## Clean-candidate validation

On the exact immutable implementation above, `cargo xtask ci` and `cargo xtask
oracle` passed, retaining all original gates, cases, seeds, tolerances and timing
thresholds. The native inventory is 60 mandatory suites / 27,513 case identities
including refusals. `ci_report.py evidence` validated every suite's exact candidate,
reference and complete raw profile. `cargo xtask wasm-probe`, complete browser
contract tests and real Chrome passed all 4,431 prior cases and 36 negative protocol
checks (1,952 successful values and 2,479 source refusals). Browser evidence remains
library probes, not a Rust browser/session application.

The candidate stayed clean throughout these runs. Final metadata/docs point to
it; hosted checks apply separately to the final PR head before readiness. This
is fixed-corpus verification, never fuzzed/coverage-reviewed/accepted status.

## M3 continuation

Next canonical unit is `engine.session.scan`: actual mesh/source assets and the
sole family/session document, replacement/reset/revision semantics and provider
ownership. Then connect supported import and marked-plane commands, global
undo/apply, original-compatible projects and native CLI. The CLI still has no
scan/plane/undo/save workflow. Native platform timers/event ingress and MCP
transport remain explicit continuations.

This is a typed source-provider boundary, not a general JS Promise/DOM interpreter.
Arbitrary thenables/proxy getters, non-JSON host objects and global abort-listener
exception reporting remain outside it. The Host must provide real settlers and
a trusted detached-fault sink; providers preserve source completion/await boundaries.
No original TypeScript, external/workspace dependency, source reference, numerical
formula, seed, tolerance or 1500 ms crease threshold changed. No M3 acceptance.

| Retained artifact | SHA-256 |
| --- | --- |
| `readonly-extended-compare.log` | `6750b0a75da29131569c5c70a99d780ecf276ce55fb25713e2c95ef80dc236ed` |
| `readonly-continuation-red.log` | `44a5aba2e08307ad39dea93c06aa57a72b2a4257e60bd393ff07a9be82c2b15d` |
| `readonly-clock-sites-red.log` | `e6517ad28b1ca766d739caafe9b7bb0c4698c598860b0641c193c2a25d3982eb` |
| `readonly-full-canary-red.log` | `a7839baba796892b95d1e36e9e8e6f7f6d87f47bc91238ff8d4db9ae828c9b3a` |
| `readonly-full-canary-exit.txt` | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |
| `readonly-canary-invocation-red.log` | `2c01aea4b651fefab44497ce2115468554a792fc7401d7894200c3d0e0b284b1` |
| `readonly-committed-ci.log` | `3cd3bc25ae1aa82a3f0c93a87ab3c5a8409a34dc6cc02d988a6b35c0f6bedfdb` |
| `readonly-committed-oracle.log` | `29b518f29d20ca3c59446b5b74dd7d6c730e625fdc4afbf58e501f34e7eab5e0` |
| `readonly-committed-evidence.log` | `55178e0e492b79eb9a26ed25344c7d60ce424a1c5347fbf18d684f42a83b9719` |
| `readonly-committed-wasm.log` | `bb23345a327ac940f016b777bd7450ec839f888a6323a9249af4b99c8e0e6e17` |
| `readonly-committed-browser-contract.log` | `361897c55f3264acac63830a83421cf653f189c0d1fa3005afaffc3d5b30a196` |
| `readonly-committed-browser.log` | `04f3ed167098bd605f50bfa105f1578e8ecbf819ca760e1a64874cc4bc13bd09` |
| `2026-11-03-registered-readonly.json.gz` | `923aa031545b3895a857c9bd6c70b14b6bbce2770589585a4e951ee5588f9b5c` |
