<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Session, command and history contract (unit `engine.session.contract`)

Written 2026-11-02 from the pinned original (`2cd89c6`). Every statement below is tagged: **DOCUMENTED** (read in
the named source file at the pin), **INFERRED** (our reading of that code) or **DESIGN** (our choice for the Rust
port, not the original's behaviour). This is a specification, not an implementation. It does not change any rule in
`HANDOFF.md` and it does not claim acceptance: the traces in section 8 are what would prove it, and none exists yet.
`ARCHITECTURE.md` ("Commands, history and background work") states the intent; this file states the observed rules
the Rust session has to reproduce.

Sources read in full: `src/commands/types.ts`, `registry.ts`, `state.ts`; `src/state/historyStore.ts`;
`src/commands/activity.ts` (busy guard, plugin stop); `src/core/workerRpc.ts`; `src/core/workerProtocol.ts`
(request/response union); `src/core/workerClient.ts` (request ids, staged imports). Not read here and still open:
`schema.ts` (validation rules), `stalePage.ts`, `host.ts` (what the session guard reports), `agentLink.ts`,
`core/*.ts` (each command's own body).

## 1. What a command is

DOCUMENTED (`types.ts`). A command has: `name` (verb; full name is `namespace.verb`, lower case, words joined by
underscores, regex `^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$`), `title`, `description`, an object `input` schema, and
flags that change how it is scheduled:

| Flag | Effect |
| --- | --- |
| `readOnly` | Changes nothing; runs at once beside other work; not a history step. |
| `scene` | Touches the viewport; runs one at a time with every other `scene` command, never beside one. |
| `history: false` | A non-read-only command that is not a history step (it walks history itself or starts afresh). |
| `interrupts` | Stops what the session is busy with; runs at once, not refused when busy. |
| `discards(input)` | A run, as asked, throws away the session's work; plugin work is stopped first, not waited for. |
| `deferred` (call option) | More changes follow; work only the last needs may be left undone and is finished before the next non-deferred run. |
| `timeoutMs` | Time before the caller is answered `timeout`; default 10 minutes (`COMMAND_TIMEOUT_MS = 600000`). |
| `exact`, `long`, `returnsFile`, `returnsImage`, `names`, `askForLess`, `headless` | Metadata for callers (MCP, listing); no scheduling effect except as noted in section 6. |

A command returns `Promise<unknown>` (JSON-like, or a result carrying `file: FileOut {name, mimeType, bytes}`) and
may call `ctx.progress(text)`, read `ctx.signal`, and read `ctx.deferred`.

**Registration** (`registerCommands`, DOCUMENTED): a duplicate full name throws; an input schema that is not an
object throws; a schema past `SCHEMA_BUDGET` (bytes or nesting depth, values in `schema.ts`, not yet read) throws.
Unregistering removes exactly the names that call added. `listCommands()` is sorted by `localeCompare` on name and
returns each command's public fields; `exact`, `long`, `names`, `askForLess` appear only when set (key presence
matters for the wire form).

## 2. Errors

DOCUMENTED (`types.ts`, `registry.ts`). Twelve codes: `unknown_command`, `invalid_input`, `busy`, `no_scan`,
`not_found`, `invalid_state`, `failed`, `unavailable`, `not_implemented`, `timeout`, `internal`, `stale_page`. A
`CommandError` serialises as `{code, message}`. `runCommand` rejects with a `CommandError` and **never anything
else**: a thrown `CommandError` passes through; a stale-import fault becomes `stale_page`; any other thrown value
becomes `failed` with its message (`Error.message` or `String(value)`).

The Rust port reuses the merged `port-engine` error envelope (unit `engine.error.envelope`, PR #30): the twelve
codes, lossless message fields and the JSON form already exist and are compared. This spec adds no code and does
not replace UTF-16 messages with `Display` text. **Open (needs `host.ts` and `schema.ts`):** the `busy` message is `The session is busy — <what>. Try again once it has
finished.` (DOCUMENTED); `<what>` is the sentence the session guard returns, which `host.ts` supplies for the
session's own causes. The plugin causes are in section 3.10.

## 3. Scheduling lanes

DOCUMENTED (`registry.ts`). There are four lanes, and the order of checks inside `start` matters.

**3.1 Admission (at call time, before any queueing).** Unknown name -> reject `unknown_command`
(`There is no command "<name>".`). Input is validated against the schema -> reject `invalid_input` with the errors
joined by `"; "`. A missing input is `{}`. Both happen synchronously at call time, so a bad call never waits in a queue.

**3.2 Reads (`readOnly`).** Start at once. A fresh abort controller per run. Unless the call is deferred, the run
first waits for all settlers (3.5) to finish what deferred runs left. A `scene` read is serialised on the scene
queue (3.4). The run is raced against its timeout (3.6). Errors map as in section 2. Reads are not history steps
and are not refused when the session is busy.

**3.3 Mutations (not `readOnly`, not `interrupts`).** One FIFO chain `queue`: each mutation starts after the
previous one settled, whether it succeeded or failed (a rejected mutation does not stall the chain). Inside the
chain a `scene` mutation also waits on the scene queue. A mutation runs `execute`:

1. `stops = command.discards?.(input) === true && guard.busy() !== null`. If `stops`, plugin work is stopped
   (`stopPluginWork`), not waited for, and the run continues once that is arranged; the stop is released when the
   command finishes. Otherwise, if not deferred, all settlers run first.
2. `guarded`: if `guard.busy()` is non-null **at this moment (after waiting in the queue)** the call is rejected
   `busy`. The guard is evaluated when the call's turn comes, not when it was submitted.
3. A fresh abort controller, an activity record `{name, title, since}` and a progress line are set; the module-level
   `current = {name, deferred}` is set for code the command calls into.
4. The run is wrapped in a history step labelled `Agent: <label(input) ?? title.toLowerCase()>` unless
   `history === false`; the command body is raced against its timeout inside the step.
5. On any outcome `current` and the activity record are cleared.

**3.4 Scene lane.** `inScene(command, run)`: a command with `scene` runs after every earlier `scene` run has
settled; a failure does not block the next. Non-scene commands never wait on it.

**3.5 Settle (deferred work).** `registerSettle(fn)` adds a function called with the next `{name, readOnly}`; it may
return a promise, may throw (ignored), and may leave the work to the command about to run. A non-deferred run waits
for all returned promises (their failures are ignored: "the next command's to meet"). A deferred run skips settling.
The returned function removes the settler.

**3.6 Interrupts (`interrupts`).** Not queued, not refused when busy, no settling. It is a step of the history only
if no step is open: if a change is running, the history wrapper sees an open step and the interrupt's effects fall
into that change's step; otherwise it has its own `Agent: ...` step (INFERRED from `historyAsync`'s `paused`
check, section 5). `history === false` skips the step.

**3.7 Timeout.** `limited(name, ms, abort, run)` races the body against a timer. On expiry it aborts the controller
with the `timeout` error and rejects the caller with `timeout`: `<name> did not finish in <ms/1000> s and was given
up on. It may have changed the session in part — read session.state (history.undo says the last step) before
running it again.` The body is **not cancelled**: a body that ignores `ctx.signal` keeps running and may still
change the session afterwards (DOCUMENTED in the message and in `types.ts`; `ARCHITECTURE.md` says to preserve it).
Its late rejection is swallowed. The history step ends when the timeout fires, because the race is inside the step,
so changes made after expiry fall outside the step (INFERRED). **The Rust port must not add rollback here.**

**3.8 Progress.** `ctx.progress(text)` forwards to the caller's `onProgress`. For a mutation it also sets the
activity strip, but not once that run's signal is aborted ("a run let go past its time says nothing more").

**3.9 Recent outcomes (jobs).** If the caller supplies a `job` name, the run is recorded: `{job, name, state:
running|done|failed, started (ISO), ms (null while running), result? | resultOmitted?, error? {code, message}}`.
At most `RECENT_KEPT = 100` are kept (oldest dropped when exceeded). A result is stored as a JSON copy, with
`Uint8Array` replaced by `"<n> bytes"`; if its JSON text exceeds `RECENT_RESULT_CHARS = 16000` characters only
`resultOmitted: "<n> characters, past the 16000 kept"` is stored; a non-JSON result stores `resultOmitted: "not kept:
it is not JSON"`. `recentOutcomes(job?)` returns copies, newest first, optionally filtered.

**3.10 The busy guard and plugin work** (`activity.ts`, DOCUMENTED). `commandGuard().busy()` returns a sentence or
null; the host installs it (`configureCommands`) and the default is "never busy". Plugins add checks with
`registerBusy(id, check, stores, stop?)`: two under one id throw; `check()` returns a few words or null; a check
that throws counts as not busy. `pluginBusy()` returns the first non-null check in registration order. `stop(done)`
exists only for work that can be stopped; `stopPluginWork(done)` calls `stop` for every plugin whose check is
currently busy, awaits all, and **ignores a stop that fails** (the plugin then stays busy and the guard reports it).
`done` settles when the discarding change has run, so work the plugin would restart meanwhile can wait for it.

## 4. The session readout (`session.state`)

DOCUMENTED (`state.ts`). One JSON object read from the stores at full precision in millimetres and degrees, with
nothing per vertex (a marked surface is a count, a map is its statistics). Top-level key order as written: `app`,
`serverTooOld?`, `stale?`, `project`, `workspace`, `busy`, `running`, `runningSince?`, `runningMs?`, `scan`,
`elements`, `dimensions`, `sections`, `open`, `deviation`, `thickness`, `flat`, `hint`, `guide?`, `history`, `view`,
`ui`, `plugins`. `sessionAfterStep()` is the brief form attached to a history step (counts, `history.undo/redo`,
`held`, `plugins` briefs).

The readout is assembled from every family's state (elements, dimensions, sections, deviation, thickness, flat,
view, plugins). It cannot be implemented until those families exist. **Contract for now:** the session owns an
ordered list of *state sections*, each contributed by its owning family (the same extension point as
`registerStateSection`), and `session.state` is the ordered concatenation. The shell keys (`app`, `project`,
`workspace`, `busy`, `running*`, `history`, `ui`) belong to the session; family keys belong to families. Key order
and presence-only-when-set rules are part of the compared output.

## 5. History

DOCUMENTED (`historyStore.ts`), the rules the Rust history reproduces:

- **Steps.** An entry holds `before` and `after` patches containing only the keys that changed, a label, the
  workspace it was taken in, and a group number. Capture covers the core stores (measure, flat, deviation, thickness,
  scope, scan) plus each participant's sections. `scope` and `scan` compare by identity; other keys compare field by
  field (arrays element by element). A step with no changed keys is not recorded.
- **Limit.** `LIMIT = 64` entries; recording a 65th drops the oldest.
- **Recording clears redo.** A new step empties `future`.
- **Groups.** Within one `beginHistoryGroup`/`endHistoryGroup` span, consecutive steps in the same workspace merge
  into the previous entry (earliest `before` wins per key, latest `after` wins), only when there is no redo.
- **Nesting.** `historyAction` inside another action or while restoring or busy or blocked runs the job without a
  step; the outermost action records one step. `historyAsync` does the same for async jobs and holds `paused` for
  its duration, so nested or concurrent async wrappers while paused do not record (the interrupt case, 3.6).
- **Epoch.** A module counter `epoch` increments on `clearHistory` and on every undo/redo travel. `record` captures
  the epoch when the job starts and discards its result if the epoch changed ("a source replacement starts another
  document"). This is how a step that finishes after a new scan was opened is dropped.
- **Revision.** `revision` increments on `clearHistory` and on each travel. It does **not** change when a step is
  recorded (INFERRED from the code: `record` does not touch it).
- **Blocked.** `historyBlocked()` is true while restoring, while any async step is open, when a hook says so, when
  the session is busy, when a map or alignment or thickness run is in progress, or when any editor holds work.
  Undo/redo while blocked is a silent no-op.
- **Travel.** Undo/redo take the last past/future entry, run `beforeRestore` hooks (core, then each participant),
  apply the patch (core stores, then participants' sections, scope), set the workspace, move the entry between
  `past` and `future`, run `afterRestore` hooks (re-measuring what the step changed), and set the status line
  `Undone: <label>.` / `Redone: <label>.`. If any hook throws, `busy` is cleared and the error line is `History could
  not be restored — <message>`, and `restoring` is cleared. The scan file is held by identity and not copied.
- **Local history.** A participant may supply its own undo that replaces the app's while active.

DESIGN. In Rust the history is a participant list over **owned state sections**, not store snapshots; a step stores
diffs of section values. The observable rules above (limit, redo clearing, grouping, nesting, epoch drop, blocked
no-op, label strings, status text) are the contract; the representation is free.

## 6. Background compute (worker protocol)

DOCUMENTED (`workerRpc.ts`, `workerProtocol.ts`, `workerClient.ts`):

- **Ids.** Each client numbers its requests from 1 with a monotonic counter. A response carries the same
  `requestId`; a response whose id is not pending is dropped silently.
- **Errors.** A response of type `error` rejects that request with `Error(message)`.
- **Serial worker.** One request is processed at a time. `limitMs` is per request and its clock starts only after
  every request sent before it has settled; on expiry the request is rejected `<name> did not answer the "<type>"
  request in <s> s; it was let go.`, removed from pending, and its late answer is dropped.
- **Dead worker.** `fail(message)` marks the worker dead, terminates it, rejects **every** pending request with the
  message, and calls `onFailure`. While dead every new request is rejected immediately with `<name> is unavailable.
  Retry the operation.` until `restart()`. `restart()` first fails everything pending with `<name> was restarted.`
  A worker `onerror` fails with `<name> failed: <message>`; an unreadable response fails with `<name> returned an
  unreadable response.`
- **Events.** Messages the `event` hook consumes (progress, `align-progress`) settle nothing and arrive while the
  request is open.
- **Abort.** `align-abort` has no id, is never answered on its own, and jumps the queue; the interrupted request
  settles as `align-stopped` instead of `align-ok`.
- **Staged imports.** `load` with `staged` returns a prepared model by request id; `commit-import {scan?, nominal?}`
  puts prepared models in place (a number selects a prepared load, `null` empties the slot, omitted keeps it);
  `discard-import {ids}` drops prepared models. The current scan stays available until commit. Nothing is committed on
  a failed or abandoned staging.
- **Transfer.** Buffers in `transfer` are handed over, not copied.

DESIGN (**the session epoch**). The original's worker protocol has no session epoch; stale answers are dropped by
request id, and history uses its own epoch. `ARCHITECTURE.md` requires the Rust backend protocol to carry version,
capabilities, request id and session epoch. Reconciling them: a Rust request carries `(epoch, request_id)`; the
session epoch increments exactly where the original *would drop a late answer by other means* (a scan or reference
replacement, a worker restart, a project open). A late result with a stale epoch is dropped. Observable behaviour
matches the original's, which is the test in section 8. The epoch must not become a second way to refuse work the
original accepts.

## 7. Rust shape (DESIGN, to be agreed before implementation)

1. **One owner.** A `Session` owns the document state and is the only writer. Families (elements, sections,
   deviation, thickness, flat, plugins) own their state *inside* the session through registered sections; there is no
   second copy of business state (`acceptance` of this unit).
2. **Deterministic, event-driven core.** The original's promise queue is expressed as a state machine: `submit(call)
   -> CallId`, events in (`Tick(now)`, `BodyFinished(id, result)`, `Progress(id, text)`, `WorkerReply`), effects
   out (start body, answer caller, set activity). It needs no async runtime and no dependency (the default in
   `ROADMAP-TO-USABLE.md` section 6), runs identically native and in Wasm, and lets tests inject a **clock** so the
   10-minute timeout and the 100-entry job log are tested without waiting.
3. **Adapters are thin.** Native threads and the browser worker implement "run this body, report when finished";
   scheduling decisions stay in the core.
4. **Errors** reuse `port-engine::error`.
5. **No stronger guarantees.** No rollback on timeout, no all-or-nothing batches, no retry, unless a separate
   reviewed decision says so with comparison evidence.

## 8. How this gets proved (original-versus-Rust traces)

The registry and history store run under Node without a browser (the original's own `tests/` import them), so
scripted scenarios can be recorded from the pinned original on independent copies and replayed against the Rust
session with an injected clock. Required scenarios (each compares result, error code and message, the ordered event
log, the final `history` readout and `recent` outcomes):

1. unknown command; invalid input (one and several schema errors)
2. two mutations queued; a failing first one does not stall the second
3. read while a mutation runs; scene commands serialised, non-scene not
4. mutation while busy -> `busy` decided at run time, not submission time
5. timeout: body ignores the signal and mutates afterwards; step boundary; message text
6. interrupt during a running change: undo step merges into the change's step; without one, its own step
7. `discards` stops plugin work instead of waiting
8. deferred run then non-deferred: settle ordering, throwing settler
9. undo/redo: patch contents, label, status text, workspace switch, blocked no-op, hook failure path
10. history limit (65 steps), redo cleared by a new step, group merge, nested actions
11. epoch: a step finishing after `clearHistory` is dropped; `revision` movement
12. recent outcomes: cap 100, 16000-character cut, `Uint8Array` as byte count, filter by job, newest first
13. worker: id matching, dead worker rejecting all pending then refusing, restart, per-request limit clock, late
    answer dropped, abort messages
14. staged import: commit with number/null/omitted slots; discard; failed staging commits nothing

Acceptance for this unit stays unmet until those traces exist and pass; the spec alone moves it to `specified`.

## 9. Open items

- Read `host.ts` (the session guard's busy sentences), `schema.ts` (validation rules and `SCHEMA_BUDGET`) and
  `stalePage.ts` and add them here.
- `session.state` per-family content waits on those families.
- Decide the clock/event representation (section 7.2) with the Codex side before code; record it as a decision file.
- Extra revision/epoch semantics for the Rust Wasm worker boundary (section 6, DESIGN) need the owner's review.
