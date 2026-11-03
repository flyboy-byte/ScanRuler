# Marked-sphere execution in Chrome Wasm

2026-11-03. PR #94 depends on #93 and #90; review both parents before retargeting
to integrated main. The private input-only mode9 reuses the bounded sphere packet
decoder; its optional size is knownDiameter rather than mode8's held radius. It
executes the actual native marked-sphere function and emits all element fields,
held presence/summary, clipped/original counts, original ordered region and real
Arc owner-identity comparison. Source refusals emit exact FitError name, brand,
constructor and full UTF8 message; host/codec/allocation failures poison output.

Live v6 requires4431 cases: every earlier2211 plus all2220 marked-sphere cases.
Historical v2-v5 proofs retain their original rosters. No prior inputs, numerical
budgets,30 protocol checks, three native-only workload exclusions or1MiB input/
output,30000scalar,10000selected-point,64MiB memory caps change. Mode9 retains the
source-pre-registered1e-10 absolute/relative scalar contract; all discrete IDs,
counts, optional presence, messages, zero signs and nonfinite classes are exact.
Six new protocol checks distinguish completed marking-size errors from malformed
packets, unsupported held presence, oversized counts, trailing data and invalid
geometry indices. The unknown-native-mode canary moves from newly valid9 to10;
unknown99 and every prior failure/recovery category remain tested.

The browser receives only mode/input. Original/native/expected outputs stay in
the coordinator. Complete native sidecars/descriptors/post-input hashes are
mandatory. Reference executes the full pinned18-module dependency graph; no
Rust output becomes a source expectation. Actual clean Chrome execution and full
raw proof/module/browser/native/candidate identities are required before claiming
coverage. Node smoke/Wasm compilation cannot substitute for it.

This covers a library continuation and its exact decimal-error formatter through
real marked-sphere calls; it is not the complete8492-case standalone formatter
browser corpus or engine/session/command/history/project/UI/CLI functionality.
Seeded sphere/growth/guides, broader profiles, generic exact-base ratchet and
accepted-feature/M3 gates remain open.
