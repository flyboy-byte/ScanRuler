# Live plugin stopping and owned concurrent waits

2026-11-03. PR #78 ports the complete stopPluginWork body. Iteration is over
live ordered stop entries; checks are read for each current id. Null, empty and
thrown checks skip stopping. Truthy checks invoke the stop selected before the
check, even if that check replaces the registration. Synchronous stop errors
and asynchronous rejections are swallowed; all eligible stops start before any
wait. Every stop receives the identical shared done capability.

227 independent source-first cases cover every null/empty/busy/throw check pair,
ready/late success/rejection and synchronous stop failures. Two rounds compare
full maps, callback/effect order, exact capability identity, pending/completed
status and zero unhandled rejections at every scheduling boundary. Mutation during
checks and stops removes future entries, appends new entries, deletes/re-adds self
or future entries and replaces the current stop with a different stop. Empty registries and
registrations without stops still return a promise without invoking any check. The first
192 captured cases remain byte-exact after adding that last regression matrix.
Original source, full bundle/map/provenance and paired raw outputs are retained.

The implementation reuses SettleRun to poll every started wait concurrently and
suppress its error. The three-map storage implementation from #77 has only crate
visibility changes; its registration/lookup bodies stay unchanged. A separate host
admission failure starts no callbacks. A visit refusal after work starts is retained
in StopRun, reported only after all prior waits drain; a unit test proves this.
Empty work still returns a run, corresponding to the source's always-present promise.

The executor must retain and drain returned runs. Rust dropping a future would
cancel work unlike an already-started JS promise. Typed callbacks must be bounded;
callback Result errors model source throws, not process panics. The done capability
is opaque here and belongs to the command executor; resolving it and microtask/native
runtime scheduling remain integration work. No browser/whole-session acceptance claim.
Run cargo xtask ci and cargo xtask oracle from rust/; every older suite stays required.
