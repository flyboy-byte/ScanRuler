# Port review and next implementation handoff

Observed 2026-11-03 07:54:39 UTC. Main at `c3b46747f633398a54eaa225d6ca7354af1bd649`; original pin remains `2cd89c6`.
This is an observed review queue, not a live CI display or merge approval.
Refresh each PR's Checks and exact head before reviewing. The raw snapshot is
[evidence/2026-11-03-pr-review-snapshot.json](evidence/2026-11-03-pr-review-snapshot.json).
No PR was merged by this Codex pass. The owner audits/merges the code.

## Review order

- CI/source provenance and scheduling: **#62 → #69**. #69 retains two complete
  independent one-worker source runs at643bd5b and final green checks. It keeps
  every source test, the1500ms assertion and20-minute job budget. It supplies
  limited scheduling evidence; future timing failures must stay red.
- Independent metadata correction: **#64**, preserving every existing input and
  source/Rust result; old history seed labels are explained rather than rewritten.
- Session/history/scheduler stack: **#55 → #57 → #59 → #66 → #67 → #70 → #71 → #72**.
- Stale/error/clock stack: **#60 → #61 → #63 → #65 → #68**.
- **#58** is the other agent's3MF work. Audit it separately and retain its owned
  import scope; no ownership of that implementation is claimed here.

The stacks share baseline cases. Do not add their inventory totals together.
After integration, regenerate from canonical unit files. Generated status/backlog
conflicts should be regenerated after preserving both sides' affected units and
all historical evidence, not resolved by discarding a branch's inventory.

## Observed heads and checks

| PR | Base | Head | Observed required checks | Scope |
| --- | --- | --- | --- | --- |
| [#55](https://github.com/flyboy-byte/ScanRuler/pull/55) | main | `7080401a` | 11 green | engine.session.history.core: port core capture and apply rules |
| [#57](https://github.com/flyboy-byte/ScanRuler/pull/57) | codex/history-core | `79bdb116` | 11 green | engine.session.import.queue: port serialized jobs and listener failures |
| [#58](https://github.com/flyboy-byte/ScanRuler/pull/58) | main | `cc5e49ac` | red: source-tests,test-and-build | io.threemf: port 3MF import (zip container, units, textures) |
| [#59](https://github.com/flyboy-byte/ScanRuler/pull/59) | codex/import-queue | `61e790a3` | 11 green | engine.session.command.deadline: port timeout races and late outcomes |
| [#60](https://github.com/flyboy-byte/ScanRuler/pull/60) | main | `a68e1b7e` | 11 green | engine.session.stale.detect: port stale-import classification |
| [#61](https://github.com/flyboy-byte/ScanRuler/pull/61) | codex/stale-detect | `9ab46c69` | red: source-tests,test-and-build | engine.session.stale.clock: port first-stale timestamp and reset |
| [#62](https://github.com/flyboy-byte/ScanRuler/pull/62) | main | `24ec577d` | 11 green | ci: fetch original history for source provenance checks |
| [#63](https://github.com/flyboy-byte/ScanRuler/pull/63) | codex/stale-clock | `71898cc8` | 11 green | engine.session.stale.error: port stale-page envelope construction |
| [#64](https://github.com/flyboy-byte/ScanRuler/pull/64) | main | `c4ffc47d` | 11 green | test: derive history trace seed labels from actual fixtures |
| [#65](https://github.com/flyboy-byte/ScanRuler/pull/65) | codex/stale-error | `038cdff5` | 11 green | engine.session.error.normalize: port command fault classification |
| [#66](https://github.com/flyboy-byte/ScanRuler/pull/66) | codex/command-deadline | `06b624ad` | 11 green | engine.session.settle.run: port deferred-work settlement body |
| [#67](https://github.com/flyboy-byte/ScanRuler/pull/67) | codex/settle-run | `f0bb488b` | 11 green | engine.session.settle.registry: port live callback registration |
| [#68](https://github.com/flyboy-byte/ScanRuler/pull/68) | codex/error-normalization | `0b59d60d` | 11 green | engine.session.clock.utc: port pure-Rust UTC timestamp conversion |
| [#69](https://github.com/flyboy-byte/ScanRuler/pull/69) | codex/plan-reference-history | `b196017e` | 11 green | ci: isolate complete source suite to one worker |
| [#70](https://github.com/flyboy-byte/ScanRuler/pull/70) | codex/settle-registry | `b3f8c255` | 11 green | engine.session.activity.guard: port guard configuration and reset |
| [#71](https://github.com/flyboy-byte/ScanRuler/pull/71) | codex/activity-guard | `3597a328` | 11 green | engine.session.activity.stores: port watched-store source identity |
| [#72](https://github.com/flyboy-byte/ScanRuler/pull/72) | codex/watch-stores | `f6b6892c` | 11 green | engine.session.activity.watch: port subscription snapshot and teardown |

CLA is an independent skipped check in this owner fork, not one of these eleven.
Initial crease timing failures and the single failed-lane reruns remain in the
PR bodies/findings even when a rerun turns green. #61's two failed attempts were
retained; do not silently waive that red check. #69 is a separately reviewed
scheduling change, not permission to loosen timing or numerical assertions.

## What the code proves

These are scoped Rust library bodies over documented owned host boundaries,
with independently executed pinned source/value/state comparisons. Each PR body
names its source proof, immutable/raw evidence, tested SHA, exact native inventory
and remaining boundaries. All previous fixed cases, seeds and tolerances remain
required. No original source/dependency/lock edit accompanies these ports.

The complete session stays specified, with no accepted feature claim. Actual
application state ownership, command execution/routes, native plugin providers,
executor/timer/worker integration, project/CLI workflow and Rust UI remain open.
The Rust CLI still prints its banner. The port cannot yet perform M3's scan →
plane → undo → project-save workflow. Typed callback probes are not actual
feature-command bodies or restored measurement-family models.

A green browser-wasm lane covers its existing651-case library subset. It does
not establish actual browser execution for the new engine control components.
`cargo xtask diff`/`diff-ci` and the general exact-base coverage ratchet remain
unbuilt. Implemented is not accepted and no completion percentage is claimed.

## Audit commands and integration

Use system Rust (no rustup). Node22 is original-oracle tooling. From the root:

```sh
python3 docs/port-plan/tools/port_plan.py --root docs/port-plan render
python3 docs/port-plan/tools/ci_report.py status
python3 -m unittest discover -s docs/port-plan/tools -p 'test_*.py'
```

Then from rust/: `cargo xtask ci`, `cargo xtask oracle`. Validate raw artifacts
at the exact clean integration head using
`python3 docs/port-plan/tools/ci_report.py evidence --candidate <full-sha>` from
the root. A prior-head passing run is historical evidence; never relabel its
candidate to make a newer head appear tested. Keep source unit/MCP/browser,
catalog, license and actual Chrome-Wasm checks mandatory. No self-merge,
force-push, tag/release or dependency adoption is authorized by this document.

## First next implementation work

1. Finish native plugin busy/store/stop registration and activity-state providers.
   #72 receives the current ordered plugin-store view; it does not own that map.
   Preserve empty busy text: execute's discard preflight tests `!== null`, while
   guarded rejection tests truthiness. They are different source conditions.
2. Implement scene serialization and actual command execution over the reviewed
   history/deadline/settlement/error components. Retain started body work after
   caller timeout and keep late onProgress delivery; only strip updates are
   suppressed by an aborted signal. Executor ownership/abandonment needs real
   tests, not an implicit Rust future cancellation policy.
3. Wire the first real measurement/import command and actual session state, then
   undo and project-compatible save before declaring M3. Missing family models
   and project compatibility cannot be represented by generic probe callbacks.
4. The task picker also identifies `harness.verification.levels` as startable.
   That changes status/tooling across all unit files and needs one coordinated
   owner after these stacks integrate. Keep absent accepted/browser/coverage
   evidence explicit; never infer it from a green fixed-corpus lane.

Source execution details are recorded in the accompanying finding. Run
`next_task.py` against the integrated checkout and inspect live claims before
starting another shared contract; a branch's task inventory excludes unmerged
units present only on a companion stack.
