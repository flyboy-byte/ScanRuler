# Native cooperative task owner

2026-11-03. PR #88 introduces Rust host machinery needed to retain command bodies
independently of caller observation. This is not a port of a JavaScript source
function or evidence of integrated command/runtime parity. The unit deliberately
keeps verification none; its tests establish the host mechanism's own contract.

An admitted future is retained in the session's Owner and first-polled before
spawn returns. A Ticket only observes whether that future reached Ready. Dropping
all tickets does not remove the future. The session must keep pumping the owner
and retain it until finish succeeds; finish refuses pending/running work. Session
shutdown is distinct from dropping a caller result. There is no task-cancel API.

Futures can borrow the session and use non-Send Rc state. Each task has an Arc
wake signal; its thread-safe queue carries only ready identities. Native threads
can wake tasks, but application futures run exclusively in the owning thread.
Metadata is pre-reserved. Signals coalesce duplicate wakes and preserve FIFO wake
order. A self-wake goes to the back. Completion retires queued tokens before slot
reuse, so old wakers cannot wake a new task. The ready queue cannot allocate in
Wake. Rc/Arc/future allocations are ordinary Rust allocation, not OOM isolation.

No mutable owner borrow spans a poll or a completed future's destructor. Reentrant
submission is supported and depth-bounded. Reentrant pumping is refused to prevent
overlapping polls. Each pump has a finite configured poll budget and returns
active/ready counts. No ready work is different from no retained work: external
I/O/timer delivery may still be needed. Admission failure never first-polls the
refused future. Fully drained finish closes admission for every owner clone.

Nine behavior tests establish eager startup, ticket-drop/late-effects/opaque
fault consumption, FIFO/deduplicated/self/cross-thread wakes, finite turns and
fairness, stale wakers/slot reuse, nested submissions/depth bounds, non-idle finish,
closed admission, reentrant destructor submission into the freed slot and borrowed non-Send futures. A 128-generation self-wake/reuse
probe protects queue retirement. These are host tests, not oracle case identities.

Future polling alone does not reproduce JavaScript await/microtask ordering.
Adapters must deliver source reactions in the correct order and be compared against
the full pinned original. Next: retain body futures here, connect actual deadline
registration/abort/cleanup reactions, then history/guarded/interrupt and execute.
Timeout must finish the caller/queue/history while the separate body stays owned.
Late progress must skip the strip after abort but still call the caller callback.
Normalization and activity-listener/finally failures must retain source ordering.
Real platform timers and browser-Wasm execution also remain open.

Existing source comparison suites and native/Wasm compilation remain mandatory.
No accepted unit, command or usable scan/fit/undo/save workflow is claimed here.
