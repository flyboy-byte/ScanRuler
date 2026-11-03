# First stale-page timestamp and reset

2026-11-02. PR #61 ports pageStale, markPageStale and resetPageStale.
The first successful mark captures the UTC timestamp; subsequent marks do not
sample again. Reset clears it and permits sampling again. A thrown clock call
leaves the state unmarked. Getter/reset perform no clock calls.

A synchronous non-reentrant native host supplies owned UTF-16 ISO UTC text.
The port retains it exactly. Clock errors preserve opaque non-Clone payloads;
text-budget refusal is a separate host outcome and also leaves state untouched.
Actual system-clock acquisition and UTC conversion remain an explicit adapter,
not an implemented Date backend. The browser preload hook and registry error
normalization are separate follow-ups. Mutable JS Date constructor reentrancy
is outside this owned boundary.

The existing builder compiles both complete pinned source modules; #60 retains
those exact source files/maps/provenance. The new source adapter injects the
platform Date clock only: its actual Date.toISOString still performs conversion.
64 source-first input sequences compare every timestamp, clock call count,
error and state after get/mark/reset. Inputs include pre-epoch/year-zero/extended
year UTC stamps and failures on the first three sampled calls. Full raw paired
artifacts and independent source-only capture are committed. Existing #60 cases
and every previous native suite stay mandatory.

Native roster on this stack:33 suites /3,467 identities. Native tests and Wasm
compile are required; actual browser runtime and full session acceptance remain
open. Run cargo xtask ci and cargo xtask oracle from rust/.
