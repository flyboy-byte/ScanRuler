<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Import queue comparison

`engine.session.import.queue` ports the full original ImportQueue class from
src/app/importQueue.ts at2cd89c6. Rust uses standard owned futures and local-thread
Rc metadata, with no runtime dependency or second business store. The session's
executor retains and polls every returned Run, including jobs whose result is
not otherwise used. A factory may borrow its host; no artificial static bound
is imposed on the job future.

run increments pending and notifies synchronously BEFORE appending its tail.
A listener exception at that point leaves the increment and never enqueues the
job, matching the source. Reentrant listener jobs therefore enqueue before the
outer job. Completion decrements pending and notifies even when a job fails;
a listener error in finally replaces the job outcome. Every success/rejection
resolves the tail signal so the next job proceeds.

Listeners preserve function identity and live Set iteration: duplicate subscribe
does not reorder; unvisited deleted callbacks are skipped; added callbacks are
visited in the same notification. Old unsubscribe capabilities remove later
registrations of the same function, and token drop does not unsubscribe. Weak
queue handles allow callbacks to avoid strong Rc reference cycles.

67 source cases precede Rust:755 full observations,59 successful job outcomes
and57 failed outcomes. Cases cover FIFO, immediate/controlled jobs, job/listener
errors, function identity, add/remove while notifying, reentrant enqueue and48
seeded schedules (1,7,99,1039). Every step compares busy, actual pending count,
complete notification/job/result logs, each accepted job's status/value/error and
operation exceptions. Job results and errors contain unpaired UTF16 surrogates.
No original result enters Rust. The oracle transpiles the full pinned class with
its installed pinned TypeScript compiler, without reached stubs. Bundle/map,
compiler/Node provenance, inputs and source/Rust bytes are archived losslessly.
Regressions verify the entire original source-map content and all frames/hashes.

Host pending/registration/work/depth bounds are separate failures. Removed listener
slots count against registration storage. Arbitrary Rust panic/OOM isolation is
not exhaustive. A dropped unfinished Run quarantines the queue, keeps resource
busy metadata and wakes dependents to Abandoned without starting their bodies.
This is a host ownership failure, NOT a source cancellation feature or accepted
engine restart policy. The owner must settle workers/resources and terminate that
queue. Runtime scheduling, native/Wasm executor and actual restore/import/command
integration remain required. Ordinary tests cover these boundaries, scoped
borrows, non-Clone errors and explicit unsubscribe lifetime.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test import_queue pinned_import_queue_comparison -- --ignored --exact --nocapture`.
The source suite is mandatory under `cargo xtask oracle`:33 suites /3260 case
identities. Actual browser queue execution and parent session acceptance are open.
