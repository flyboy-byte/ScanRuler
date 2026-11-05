<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust comparison coverage bundle (M3 milestone batch)

Instrumented checkpoint c2966f3bd4d8bf9e676de3bb5b2ae3267aee3822, run with
`python3 docs/port-plan/tools/rust_coverage.py run` on a clean tree. All 69
required native suites / 54,040 case identities passed inside the instrumented
oracle; the exact-SHA native artifact inventory is native-evidence.json.gz.
Only the mandatory differential tests ran; ordinary unit tests are excluded.

Rust 1.99.1 (LLVM 23.1.3), system llvm-profdata/llvm-cov 23.1.4, Node 22.23.4. No rustup
or added packages. Flags and full versions are in run.json. Raw profiles and the
instrumented target stay in rust/target. Large files are gzip archives with zero
timestamps; manifest.json hashes every retained file. report.json and report.md
regenerate from the raw data (`rust_coverage.py report`).

95 owned files: 16,205 of 17,950 instrumented lines reached by the comparisons.
Three modules are never reached by any comparison (port-core ids.rs, transform.rs,
units.rs). 78 files have LLVM DA-versus-summary discrepancies; both metrics are
retained, not normalized. Stable instrumentation emits no branch records, so branch
reach is unknown. The per-file unreached lines are in report.md. No acceptance,
coverage percentage target, verification-level promotion or exact-base ratchet claim.
