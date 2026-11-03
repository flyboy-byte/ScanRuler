# Direct routing and the viewport queue

2026-11-03. PR #79 ports the entire inScene helper and its private sceneQueue.
Non-scene factories run immediately and return their exact future value. Their
synchronous exceptions remain synchronous. Scene factories run after preceding
scene work; their thrown errors become turn rejections, and either success or
failure releases the next turn. Non-scene work never appends to or reads that queue.
The helper adds no settlement, timer, history or viewport business state.

120 independent source-first schedules cover all three-command scene masks,
ready/late success/rejection, thrown factories and three completion orders. Every
case adds a final scene command to prove recovery. Compare complete factory order,
direct promise/future identity, synchronous vs asynchronous faults, caller result/
error identity, states at stable scheduling boundaries and zero unhandled rejection.
Inputs, full raw pairs and the full pinned registry graph/map/provenance are retained.
No original output is a Rust input. The source-map test permits only the private
inScene observation export and verifies every original module byte.

The private native queue reuses ImportQueue with no exposed listeners. Notification
bodies therefore have no effects; the scene adapter adds no duplicate queue scheduler.
The direct Route variant contains the exact original owned future. Future polling
supports Unpin direct futures (pin-box an async future when necessary); callers may
also extract the direct future. Opaque result/error values need not be Clone and
borrowed future payloads remain valid. Native bounds/refusals stay separate from
source errors. Dropping unfinished queued work quarantines the queue; the executor
must retain/drain it as the original already-started promises continue running.

Comparisons use stable drained scheduling boundaries, not a claim that native polls
are JS microtasks. Actual runtime scheduling, command deadline placement, late body
ownership and real viewport wiring remain integration work. No browser or whole-
session acceptance claim. Run cargo xtask ci/oracle from rust/; all older cases remain.
