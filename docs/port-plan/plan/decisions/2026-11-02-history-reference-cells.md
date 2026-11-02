<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# History recording reference cells — 2026-11-02

Unit `engine.session.history.record`, review PR #46. Parent session stays
specified. This is the recording stage, not undo/redo travel or a usable command.

DESIGN: the session supplies pure snapshots of its actual owned sections. History
holds immutable Arc-backed composite cells, so patches share array/object/byte/
word-buffer identity and payloads; it owns no second mutable business-state store.
Scalar values preserve undefined/null, UTF16 and IEEE nonfinite/signed-zero data.
Family adapters must preserve the identity of their actual state, not reconstruct
a fresh composite on each capture. Full family mapping/asset ownership is still
an integration task, not completed by this representation.

DOCUMENTED and MEASURED: sameValue is strict equality, or ordinary arrays of equal
length whose elements are strictly equal. Objects and typed arrays compare by
reference; scalar NaN differs from itself; signed zeros compare equal. Default
section comparison walks BEFORE object keys only. It has no root-reference
shortcut, so even the same object containing a NaN field records a change. Added
after-only keys are ignored. Normal inherited Object.prototype fallback is kept
for missing keys; custom prototypes/executable values are not represented.

DOCUMENTED and MEASURED: grouping uses `{...newBefore,...lastBefore}` and
`{...lastAfter,...newAfter}` (different key orders), retaining the first label.
A reversal inside a group is not collapsed.64 entries are kept; only a recorded
step clears redo. Clear increments history epoch/revision and resets group, not
depth/paused/nextGroup. Record changes neither epoch nor revision. Current section
policy, not its entry-time policy, controls comparison; scope/scan force identity.

DOCUMENTED and MEASURED: sync guards are depth/paused/restoring/store busy/the
configured hook. Async guards are **only paused/restoring**, ignoring depth,
busy and hooks. Participant/editor blocking is for historyBlocked, not the sync
wrapper's full guard. Before-capture precedes increment; after-capture follows
decrement. Body failures still record; after-capture failure supersedes a body
fault and records nothing. Skipped async wrappers stay skipped even when the
outer wrapper finishes first. A stale generation drops its record, not its body
changes. SESSION-CONTRACT.md's earlier “does the same” shorthand was too broad.

DESIGN: completion capabilities are domain-bound, one-shot tokens. Begin/finish
accept pure capture callbacks; async runtime/body execution remains in the session
adapter. Label/metadata/pending/counter budgets are explicit host refusals, not
invented source lines. Immutable payloads are not traversed/copied merely to
check history metadata; the session asset owner must bound their memory/lifetime.
Rust's regular Arc/clone/map allocations are not an exhaustive OOM recovery claim.

Boundary: stable registered section keys, unique own fields, acyclic immutable
ordinary data, object-shaped default-comparison sections; identity sections may
hold scalars or composites. Arbitrary mutable/reentrant/executable JavaScript
captures, changing key sets, non-object default captures and __proto__ section
setter semantics are outside this typed boundary. They are not silently accepted
or labelled ported. Captures can fail with lossless messages. Host invalid tokens/
quotas remain distinct from original capture/body faults. No new backend/session
epoch policy is adopted; this is only the original private history epoch.

Evidence:107 complete independently run original traces, with the first103 fixed
inputs/source observations preserved. Real full original history/store modules
run controlled fixture bodies and genuine participant callbacks. Each output
contains complete before/after patches, actual graph identity AND recursively
read data, workspaces/groups/revision/flags/current sections and capture order/
held state. Private original epoch is not exported; its late-drop effects and
revision movement are compared. Zero fabricated feature commands are claimed.
