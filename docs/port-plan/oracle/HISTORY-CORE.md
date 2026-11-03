<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Core history capture and apply

`engine.session.history.core` ports complete captureCore, pick and apply from
original2cd89c6 at a borrowed owned-data boundary. The owner lends its four core
field collections; no second mutable document store is added. Capture picks all
73 source fields (measure11, flat22, deviation26, thickness14), preserving order,
undefined fields and composite identity, then gets scope and scan.

Apply merges measure, flat, deviation and thickness in source order. Flat uses
CURRENT draft/dimension work to keep empty boxes or clear held work, replaces
the tool with none and increments current subjectVersion, overriding incoming
transient values. Deviation clears picking/pendingScan/marking. Participants see
the original patch cells in registration order (including duplicate keys), then
scope uses nullish fallback and scopeVersion increments only after its setter
returns. Scan stays with beforeRestore. Callback failure retains earlier effects.

98 independently authored/seeded inputs generate490 full observations before Rust.
All73 captured fields, complete owned field values, graph identities AND payloads,
transient fields, unknown retained/added data, scope/scan, participant values and
callback logs compare. Every family mask, flat draft/dimension variant, repeated
version increment, null/undefined/zero scope, absence, null roots, failure stage
and48 seeded cases (1,7,99,1039) remain fixed. Versions include signed zero,
large/nonfinite values; state/capture numbers compare full wire bits. The observer
only stringifies the fixed version domain in logs and fails on unsupported values.

The original full module/store/model graph executes from a fresh /tmp git archive.
An adapter appends ONLY `export { captureCore, apply }` to the temporary source
copy. A regression checks that source-map content equals the complete pinned
file plus exactly that export. No reached body, formula or guard is replaced.
The original and Rust frames, all inputs, bundle/map and Node provenance are
archived losslessly. The live comparator rejects a changed actual version and
a missing tail; corpus regressions verify all fields/hashes and frame completeness.

Native host-budget checks and a2.1MB shared-buffer test verify bounded metadata
and payload ownership. Stores and hooks are stable during calls; callbacks receive
an immutable view for inspection. Prototype keys, reactive subscribers, executable
getters and reentrant mutation of the borrowed family maps are outside this host.
Actual internal plugin keys (reverse, mesh, edited) do not use prototype names.
Rust panic/OOM isolation is not exhaustive. Concrete typed family models,
remeasurement, queue and end-to-end commands remain required integration; the
corpus's opaque owned field data is not measurement acceptance.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test history_core pinned_history_core_comparison -- --ignored --exact --nocapture`.
The suite is required by `cargo xtask oracle`:32 suites /3193 identities. Actual
browser core-history runtime and full parent session acceptance remain open.
