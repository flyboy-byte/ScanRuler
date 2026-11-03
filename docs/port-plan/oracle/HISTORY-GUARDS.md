<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# History guard and editor-work comparison

`engine.session.history.guards` ports complete historyBlocked, draftHolds,
flatDraftHolds and dimHolds from pinned original 2cd89c6. The owner lends a
snapshot of the fields these functions actually read, including reference/edit
presence and pick/selection lengths; geometry and editor state are not copied.
An edit is present when editId is not undefined, even for zero, null or NaN.
A reference is filled when it is not null.

Evaluation preserves short circuit order: restoring, paused, configured hook,
worker/import flags, held editor work and then participant hooks. Callback errors
stay opaque and lossless. Absent optional participant hooks are omitted by the
owner. A host bound of 4096 blocked callbacks is checked only when that stage is
reached. Source store snapshots are taken before callbacks; the Rust snapshot
has the same timing contract.

212 complete original observations precede Rust, including all one-guard and
pair combinations, all-but-one guards, draft variants (empty, edit, picks,
filled/empty references, marked selection, explicit undefined and NaN edit IDs),
participant masks/fault positions and 48 seeded cases (1,7,99,1039).
`generate_history_guard_cases.py` reproduces the immutable input schedule.

The full original history/store/model graph runs from a fresh pinned /tmp archive.
No reached stubs or original result enters Rust. Output compares the guard result,
error, all four leaf-work results and exact hook invocation order. Paused cases
open and finish an actual original historyAsync wrapper. Full source and Rust
frames, input hashes, compiled bundle/map and Node provenance are archived.
Corpus regressions replay every frame and verify bundle/input identities.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test history_guards pinned_history_guards_comparison -- --ignored --exact --nocapture`.
This suite is required by `cargo xtask oracle`; the roster is 30 suites /2879
identities. Native opaque UTF16 fault/host bounds have ordinary tests. Actual
browser execution, family state/editor integration, dynamic JS getters/reentrant
participant mutation and global restoration remain required. The parent session
contract stays specified; this is not a usable undo/redo command yet.
