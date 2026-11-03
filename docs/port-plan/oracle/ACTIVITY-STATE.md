# Command activity publication

2026-11-03. PR #86 ports the three typed patches issued by guarded execution:
start sets running and clears progress; progress preserves the running payload;
clear sets both null. Every patch publishes a fresh state identity, even when its
field values match. Initial/previous snapshots remain valid. Running name/title
preserve UTF16, clock values preserve raw bits, and an empty command name still
satisfies commandRunning. Empty progress is distinct from null.

The full pinned activity module is executed with the actual installed Zustand
vanilla implementation. Its MIT algorithm is attributed in
rust/licenses/ZUSTAND-MIT.txt; provenance records both CJS/ESM dependency hashes
and version, plus the original module/source-map/dependency-lock identities.
Captured licensed dependency bytes let plan/fast checks run without node_modules; the live oracle builder also requires exact equality with the installed dependency. No new runtime package is added. The Rust owner is the sole activity store.

Notification uses live Set-style callback identity/order. Duplicate subscription
has one entry; any old cleanup deletes a reintroduced identical callback. Dropping
cleanup does nothing. Removal skips pending callbacks; new/reintroduced callbacks
append and may run in the same pass. Each callback receives the latest current
state and the fixed previous snapshot for its publication. Reentrant publication
therefore changes later callback arguments. A listener fault preserves its exact
identity, leaves the new state published, and skips remaining callbacks.

256 fixed original-first traces compare all state values, raw clock bits,
snapshot/payload identities, commandRunning, full callback arguments/effects,
held snapshots, and exact faults. Appended empty-progress/nested-fault/live-readd
cases retain every byte of the initial192-case source profile. No original values
enter the native computation. Native controls separately exercise bounded slots,
text, callback visits and recursion plus borrowed non-Clone faults.

Finite slot/text/depth/visit admission is a host boundary, not source error
normalization. Pre-publication text/depth refusals leave state unchanged; a visit
refusal leaves the already published state and earlier callbacks visible. Depth
unwinds on returns/faults. Rc allocation is not process-wide OOM isolation.

Scope excludes arbitrary Zustand partial functions/replace, externally mutating
snapshot fields, middleware and React hooks. Full guarded/interrupt execution,
progress-after-abort routing, current assignment/cleanup ordering, actual native
session ownership and browser-Wasm activity execution remain required follow-ups.
Wasm compilation does not grant runtime acceptance; this is not M3.
