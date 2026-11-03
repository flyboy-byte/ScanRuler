# Deferred-work settlement body

2026-11-02. PR #66 ports complete settle over a stable ordered callback slice.
Each callback receives fresh immutable name/readOnly metadata. All callbacks run
before any returned waits are polled; synchronous failures are ignored. No waits
means None (source null). Otherwise every returned wait is polled and both
successful and failed completions are suppressed; the run finishes after all.
No fail-fast or sequential-await substitution is used.

64 source-first scenarios include empty/no-work/throwing callbacks, ready success
and rejection, pending success/rejection, mixed and reversed/all-at-once completion,
UTF-16 command names and both readOnly values. Seeded cases use1/7/99/1039. Full
frames retain call arguments/order, caller null/pending/done and unhandled-error
count. All source/native bytes match. Full frozen registry bundles/maps, exact
private-export source proof and complete paired artifacts are retained. No source
answer is an input to Rust. Ordinary tests cover opaque faults and admission
before callbacks; metadata is bounded/fallibly reserved before starting work.

Stable callbacks cannot mutate registration during a run. registerSettle identity,
removal and live Set iteration remain a separate owning continuation. Callbacks
must start work synchronously before returning its completion future; futures
may borrow host resources. The executor must retain and drain started runs even
if a caller stops awaiting. Dropping a Rust future does not implement source
Promise lifetime/cancellation. Actual executor/microtask scheduling, browser
runtime and full registry execution remain explicit required work.

Native roster on this stack:35 suites /3,615 identities. Run cargo xtask ci and
cargo xtask oracle in rust/. Wasm compilation is required, not browser runtime
parity or complete session acceptance.
