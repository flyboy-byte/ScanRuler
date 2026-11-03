# Session watch-store source identity

2026-11-03. PR #71 ports complete watchSessionStores and its sessionStores
ownership. It stores the exact readonly array and preserves member identities,
ordering and duplicates. Cleanup clears only when that exact array is current.
An old cleanup becomes effective again if the same array is reintroduced; a
same-content different array does not match. Successful cleanup creates a fresh
empty array, and repeated cleanup then does nothing. Dropping cleanup does not
release anything. No subscribe call belongs to this body.

80 source-first scenarios compare full container/member identities and zero
subscribe calls after every operation. They cover empty arrays, duplicates,
reordering, same-content distinct containers, old/repeated/dropped cleanup and
reconfiguration to a previously captured array. Every source/native state matches.
Full pinned activity graph/map/provenance and exact private-export proof,
independent inputs/generator and complete paired raw artifacts are retained.

The native slot owns only current-array metadata. Weak cleanup handles avoid
keeping a dead session alive; borrowed/unsized opaque store identities work.
Ordinary Rc allocation is not process-wide OOM isolation. Readonly typed arrays
exclude arbitrary JS getter/prototype/external array mutation.

Actual watchSession subscription, plugin store registration and full engine
session wiring remain separate. Native roster:38 suites /4,003 identities. Run
cargo xtask ci and cargo xtask oracle from rust/. Wasm compilation is required;
actual browser execution and whole-session acceptance remain open.
