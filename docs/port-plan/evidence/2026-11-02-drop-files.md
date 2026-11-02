# Complete drop traversal — 2026-11-02

PR #40, io.obj.companions.drop. Source-captured 40 independent callback traces
precede Rust implementation. Complete pinned droppedFiles/fileOf/childrenOf
bodies execute against authored snapshots; Rust receives independent copies.
Every DRP1 byte matches: complete file identities, platform name/message errors
and callback event order. Full outputs/hashes, actual tested SHA, source identity
and first rejected harness attempt are retained in the report/raw directories.

The Rust body uses thin generic async callbacks and bounded temporary vectors.
All file-item getters run before traversal; no-directory input uses the browser
fallback. Top entries stay ordered; each directory gets a fresh reader and all
batches are collected until empty before child files are read. Nested directories
stay ignored, duplicates remain, both-file/directory flags keep source precedence,
and original opaque callback errors propagate without Display normalization.

Tests cover pending-once file futures, fallback/entry/reader/batch/output/call
limits and a never-ending directory provider; no host failure is encoded as a
source error. Changed/missing-result canaries reject divergence. Source-only
capture, fixture generator, all seeds and complete raw state remain immutable.
No package or engine state added; file handles are platform-owned opaque values.

Actual DOM adapter/browser callback runtime, concurrent live-list behavior,
production cancellation/timeouts, public import/history/session and image decoding
remain open. Wasm compilation is not runtime acceptance. Parent stays planned;
this complete callback algorithm is implemented, not accepted.
