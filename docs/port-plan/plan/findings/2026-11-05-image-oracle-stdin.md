<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Live image oracle transport — 2026-11-05

The scheduled PNG campaign reached 10,109 seed-2 sweep cases with no mismatch,
then failed before decoding the structural batch: Node 22 `readFileSync(0)`
reported EAGAIN on the Rust child's stdin pipe. This is a transport failure,
not a decoder comparison result. `oracle/image-decode.mjs` now awaits stdin
chunks before parsing the same binary batch. Frozen source, image inputs,
comparators, seeds and limits are unchanged. Retain the failed campaign output
alongside the clean rerun in the PR's comparison record.
