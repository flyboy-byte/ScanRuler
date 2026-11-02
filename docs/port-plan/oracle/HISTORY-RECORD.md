<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# History recording comparison

`engine.session.history.record`, original2cd89c6, PR #46.110 fixed scenarios,
including48 seeded cases (12 per seed1/7/99/1039). Initial source-only captures precede Rust; expansions preserve103 then107
scenarios before typed-error additions.

The immutable original historyStore and actual stores are Vite-bundled from a
fresh /tmp git archive, without reached stubs or original edits. Compiled code is
under ignored history-reference. Fixture participants supply typed immutable graph
cells, pure captures and controllable promises; the actual historyAction/
historyAsync/group/clear/record bodies execute. `seed_future` authors bookkeeping
through the original public store on each engine's OWN prior entry; it does not
copy an original answer into Rust or claim an undo implementation.

HRC1 is a full typed Value frame (same tags/UTF16 as SCHEMA.md). Each top-level
operation observes error, workspace, complete past/future entries with full
before/after graph references AND payload, labels/group, revision/restoring/
importing/blocked, current sections, outstanding fixture bodies and the complete
ordered capture/held log. Snapshots share actual graph references; serializers
traverse recorded cells, not an input/expected-value lookup. Unknown reconstructed
reference identities fail. No status-only or ID-only substitute for data checks.

The source's original private epoch is not exported. Late-drop behavior, unchanged
body mutations and revision movement are compared; Rust's exact increment is
also checked ordinarily. Timed job scheduling, undo/redo restore/local hooks,
full family capture/apply, arbitrary executable/reentrant JS/prototypes and the
new backend-epoch policy are explicit future scope. Default typed section values
are ordinary objects; keys stay registered. No accepted status or M3 claim.

```
fnm exec --using=22 node docs/port-plan/oracle/build-session-history.mjs
python3 docs/port-plan/tools/capture_history_record_source.py
fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test history_record pinned_history_record_comparison -- --ignored --exact --nocapture
cd rust && cargo xtask ci && cargo xtask oracle
```

Native raw outputs are retained under history-record-diff/run-* with all inputs,
original/Rust bytes, stderr and clean candidate/tool identity. A real label change
and missing patch tail are rejected. Evidence regression tests also alter actual
recorded graph data and key/order metadata. Domain/one-shot tokens, host budgets
and a2.1MB shared buffer verify safe ownership without a payload copy. wasm32
compiles; actual browser history runtime remains open. No new dependency.
