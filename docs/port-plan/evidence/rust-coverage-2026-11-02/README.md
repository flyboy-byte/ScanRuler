<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust comparison coverage bundle

Instrumented source checkpoint e2c3d0010700899a47c6ec6c454d5cb3299b1141;
reporting correction be43ab8 (full SHA in manifest.json). All26 required native
suites /2015 identities passed; full exact-SHA native artifact inventory is
retained. No ordinary tests were executed in this profile campaign.

Rust1.99.1 (LLVM23.1.3), system llvm-profdata/llvm-cov23.1.4, Node22.23.4. No rustup
or added packages. Flags, complete compiler versions,59 profile identities and29 test-object
identities are in run.json. The separate instrumented target and raw binary
profiles remain in rust/target; [one-command reproduction](../../oracle/RUST-COVERAGE.md).

Full LLVM JSON/LCOV, source texts/owning-unit identities, native raw-file hashes,
oracle log and build messages are gzip archives with zero timestamps. report.json
and report.md regenerate from the raw data. manifest.json hashes every measured
artifact/log; four fast-gate tests replay the full report, check raw LLVM JSON
summaries, validate native inventory/hash scope and reject malformed/changed data.

35 owned files;398 explicit zero-count DA records; three module files have no
emitted executable records. Inline cfg(test) sections remain explicit.29 files
have LLVM DA-versus-summary discrepancies; both metrics and the initial parser
failure remain retained, not normalized away. Root cause is unresolved. Stable
instrumentation emits no branch records; branch reach is unknown. No acceptance,
coverage percentage, browser/fuzz scope or exact-base ratchet claim.
