# Subscription snapshot, identity and teardown

2026-11-03. PR #72 ports complete watchSession over typed session/also/ordered
plugin-store arrays. It snapshots all members, then identity-deduplicates before
calling subscribe, in session → also → plugin order. Exact listener identity is
passed to each distinct store. A subscribe/listener fault preserves already
applied subscriptions and yields no composite cleanup, matching source partial
failure. Successful empty input still yields an explicit callable cleanup.

Cleanup starts from its first callback every time, stops at the first fault and
retains later callbacks for a future invocation. It does not roll back earlier
calls. Shared unsubscribe functions are called once per captured entry. Dropping
cleanup does not unsubscribe. Native host reference/metadata admission precedes
all subscribe calls; those refusals are separate from original callback errors.

144 source-first cases compare1152 complete states: subscription order and
listener identity, active registration IDs, callback/emit logs, error object
identity/messages, cleanup availability and source-array state. They include
empty/duplicate/overlapping store groups, multiple orders, immediate listener
faults, partial subscribe faults, one-shot/permanent cleanup faults and
reconfiguration while subscribing. All source/native values match. Full original
activity graph/map/provenance, exact source proof, independent input generation
and complete paired raw artifacts are retained. Ordinary native tests cover
borrowed non-Clone faults, pre-admission and repeated shared cleanup functions.

Original plugin metadata is installed through real registerBusy during capture.
The native body accepts its current ordered store-array view directly; native
plugin registry wiring remains a required continuation. Owned readonly arrays
and typed callbacks exclude arbitrary JS property/prototype mutation. The host
must preserve explicit-only unsubscribe semantics and bound callback-owned
resources; partial-failure subscriptions are not silently cancelled by Rust Drop.
Actual browser watch execution and complete engine/session wiring remain open.

Native roster on this stack:39 suites /4,147 identities. Run cargo xtask ci and
cargo xtask oracle from rust/. Wasm compile is required, not whole-session
acceptance. No source body, old case, seed or numerical budget changed.
