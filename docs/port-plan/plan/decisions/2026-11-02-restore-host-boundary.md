<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Restore host boundary — 2026-11-02

The global control sequence owns history movement, epoch/revision, await order,
status/error handling and restoration cleanup. Concrete state families own their
capture/apply and remeasurement. Pass borrowed patches and previous snapshots
through typed callbacks; do not create a second mutable document store or insert
a fake global command to claim M3.

The original app configures runRestore through ImportQueue.run, which invokes its
job once. Model that owned scope with a single-use Rust closure and boxed standard
Future. The host may skip the job or handle its error, which is tested against the
original. Arbitrary concurrent/repeated callback invocation is outside this typed
boundary and is not covered by a general JS compatibility claim. Real queue and
worker lifecycle integration remain separate required work.

A drop guard clears restoring if the future is abandoned. This is an explicit
Rust ownership boundary, not a claim of a source cancellation feature or rollback.
Partial document effects remain; an engine termination policy must settle busy
and pending resources. Ordinary tests preserve epoch invalidation on abandonment.
