<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# History recording comparison — 2026-11-02

Tested implementation `b5c2c40cddc6e139542d96230db87a7587f68c9e` against
original `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`. All 110 independently
authored source scenarios compare 1,095 complete operation observations byte for
byte. The initial 103 and expanded 107 source captures remain archived and are
verified as exact prefixes of the final source capture. Four fixed seeds
(1, 7, 99, 1039) contribute 48 scenarios.

Recording, grouping, clear, nested synchronous and overlapping asynchronous
wrappers, capture/body faults, typed CommandError codes and messages, guards,
late epoch drops, cap 64 and redo clearing are observed through the complete
original history module and real stores. Each observation includes ordered
before/after patches, graph identity and actual payload, workspace, revision,
pending bodies and capture log. The Rust serializer reads recorded cells; it
does not substitute expected values. The original bundle, source map, Node
provenance, source output and paired Rust output are archived losslessly.

The immutable typed-cell and pure-capture boundary, host quotas and shared
buffer ownership are described in [HISTORY-RECORD.md](../oracle/HISTORY-RECORD.md)
and the dated decision. The corpus tests verify all hashes, all full frames,
prior source versions, and rejection of changed recorded data or truncated
frames. Original private epoch is inferred from late-drop behavior; its internal
counter is not directly exposed. Undo/redo restoration, family capture/apply,
dynamic JS getters/prototypes, actual browser runtime, complete session/CLI
workflow and acceptance remain open. Parent `engine.session.contract` stays
specified.

Review gate: 28 mandatory native suites / 2,475 case identities, exact candidate
raw inventory, `cargo xtask ci`, original plugin/unit boundaries and original
source/package/lock identity. Record the final clean gate SHA and hosted checks
in the PR after they run.
