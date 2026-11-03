<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Local editor history dispatch

`engine.session.history.local` ports complete `activeLocalHistory` selection and
the local-editor branch of `travel` from original 2cd89c6. The owner supplies the
complete historyBlocked result. `Dispatch::Global` explicitly requires the caller
to continue global restoration; it is not a completed undo.

The first active editor in registration order wins. Neither canUndo nor canRedo
is consulted. Workspace switches before the synchronous editor action, including
when the action fails. Selection, workspace and action errors propagate unchanged;
source global restore's error handler does not catch local errors. Selection
itself does not consult the travel guard. Native opaque UTF16 error ownership and
4096-participant/workspace host bounds have ordinary tests.

192 authored scenarios cover selection/undo/redo, all three-editor active masks,
blocked/restoring state and faults at every active/action position. The source-only
capture predates Rust. The complete original history graph and real stores run
from a fresh pinned /tmp archive with no reached stubs. Both engines independently
emit selected identity, error, workspace, complete past/future sizes (empty in
this slice), revision/restoring and ordered callback/workspace logs. Capability
callbacks throw if wrongly consulted. Source and Rust raw frames, inputs, hashes,
compiled bundle/map and provenance are archived losslessly.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test history_local pinned_history_local_comparison -- --ignored --exact --nocapture`.
The comparison is required by `cargo xtask oracle`. Native inventory is now
29 suites /2667 identities. No original source result enters Rust. Existing
recording, registry, parser and browser cases remain unchanged.

Full family guards, global capture/apply/restore, editor integration, dynamic JS
participants and ignored Promise-returning callbacks, browser runtime and session
acceptance remain open. The parent contract stays specified.
