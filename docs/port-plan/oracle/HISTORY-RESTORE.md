<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Global restore orchestration

`engine.session.history.restore` ports the global control sequence from travel
at original2cd89c6, after complete guards and local dispatch. The host owns the
current document, actual capture/apply, family hooks and asynchronous runtime.
The recorder retains shared immutable patch references and history bookkeeping.
This does not implement concrete core-family capture, restoration or remeasurement.

Capture happens before restoring/epoch mutation and outside the restore catch.
The owned restore scope awaits app-before, participant-before in order, applies,
opens the entry's workspace, moves the entry and increments revision, awaits
app-after and participant-after, then sets success status. A scope failure clears
busy and records the exact UTF16 error prefix/message; it does not roll back.
Finally always clears restoring. A runRestore wrapper can skip its single-use
job or catch its error, preserving those source behaviors too.

216 authored original scenarios precede Rust: both directions, empty/one/two
entry histories, two entry workspaces and 18 fault/scope variants. They compare
complete owned participant graph identity AND payload, ordered patches, labels,
workspace/group, both entry stacks, revision/restore/import flags, busy/status/
error text and every capture/hook/application event. Labels and errors contain
an unpaired surrogate to detect lossy text conversion. The original full module
and actual stores execute; fixture callbacks supply typed participant data and
failures. The owned snapshot boundary includes scope, scan and both participants.
Original core family snapshot fields remain outside this child, with their real
concrete port required as a continuation. No source result enters Rust.

The complete original and Rust observations, immutable inputs, bundle/map and
Node/lock provenance are archived losslessly. Ordinary tests cover a pending
future's drop cleanup and late wrapper invalidation by the restore epoch. A
cancelled/dropped operation clears restoring but does not undo partial effects;
the session still needs an explicit termination policy for worker busy/resources.

`RestoreHost` owns a linear single-use job. Arbitrary repeated/concurrent JS job
invocations, dynamic participant mutation/getters, primitive throw normalization,
full import queue semantics and actual native/browser command integration are
explicit open scope. Stable metadata/participant/counter bounds are host refusals,
not source-matching errors. Rust panic/OOM isolation is not exhaustive here.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test history_restore pinned_history_restore_comparison -- --ignored --exact --nocapture`.
The suite is required by `cargo xtask oracle`, now31 suites /3095 identities.
Source and earlier Rust/browser cases are unchanged. Parent session stays specified.
