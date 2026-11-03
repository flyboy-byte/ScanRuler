# Registry execution continuations verified from the pinned source

2026-11-03. Read src/commands/registry.ts start, interrupt, execute and guarded
at2cd89c6 before scheduling the next integration unit. These are not ported bodies.

- Read-only routing precedes interrupt routing. Its settling starts immediately
  before inScene enqueues the actual timed run. The deadline starts in the scene
  turn; scene waiting is not itself timed by limited. Deferred skips settling.
- Interrupt runs without the normal mutation queue/busy guard and does not carry
  deferred through this route. history=false bypasses historyAsync; labels are
  custom label(input) or the original title.toLowerCase default.
- execute preflight checks discards(input)===true and guard.busy()!==null. This
  happens before done is constructed and before its try/finally. Empty busy text
  therefore still requests plugin stopping. guarded uses truthiness to reject
  busy, so an empty string is not a busy refusal there. #70 preserves raw values.
- execute stops plugin work or settles, then guarded; its finally resolves done
  even when the guard/run fails. A preflight exception occurs before that finally.
- guarded's busy read/refusal, activity publication and current assignment are
  before its try. Do not normalize or roll back every failure as though it came
  from inside the body. Label/run faults inside the try are normalized and its
  finally clears current/activity. Source store-listener exceptions also need
  explicit branch tests before claiming a complete execution port.
- Aborted progress stops publishing to the strip, while onProgress still runs.
  Timeout ends the caller/history wait and advances the queue; body work can
  settle later and alter session state. #59 preserves this boundary; an executor
  must not silently drop the remaining body or stale-filter every late callback.

Preserve these distinctions rather than broadening a catch/finally, strengthening
an empty-string guard or introducing cancellation during integration. The general
Unicode title-lowercase default and actual label callback failures require source
comparison too. No source behavior, tolerance or existing fixture is changed.
