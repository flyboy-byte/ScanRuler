<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Import queue futures boundary — 2026-11-02

Queue state owns pending counts, listener identities and completion signals;
the runtime owns/polls scoped Run futures. This preserves job borrowing without
copying document state or requiring a foreign/external async runtime. A job's
success and failure both resolve its tail, matching the source's catch branch.
Live callbacks use shared Fn identities, so registration/removal and enqueue can
reenter without a mutable callback borrow panic. Weak handles prevent ownership
cycles where the host follows that lifetime pattern.

Source Promise jobs are runtime-driven even when the result is ignored. Rust's
owner must likewise schedule and retain each returned future. Abandoning one is
outside the source contract: quarantine, wake dependents to a distinct host error
and retain busy/resource metadata. Do not call this source cancellation or accept
a new engine restart policy; concrete runtime/worker termination remains open.

The original listener exception before enqueue leaves pending incremented. Preserve
that measured behavior; source-informed porting does not authorize silently
repairing it. Safe host resource limits stay separately classified.
