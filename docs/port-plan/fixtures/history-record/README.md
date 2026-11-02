<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Authored history recording inputs

110 immutable graph/operation scenarios. Manifest IDs/hashes/seeds are fixed;
12 cases per seed1/7/99/1039. The generator reads no source/Rust outputs. Composite
node references point backward into an authored acyclic graph; distinct equal
nodes retain distinct identities. Snapshot sections use object fields or explicit
identity policy. Scalar undefined/nonfinite/UTF16 data remain distinct.

Source103, expanded107 and final110 captures are retained independently; no old input,
field, passing comparison or seed is removed. See HISTORY-RECORD.md for full
state/graph protocol and actual original-store provenance. No feature commands,
undo/redo restore or browser runtime acceptance is claimed.
