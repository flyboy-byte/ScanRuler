<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Original-source corpus reach

`harness.coverage.original`, PR #43,2026-11-02. Oracle-only Node22/V8/TypeScript,
no new dependency. V8 precise coverage is collected with `node:inspector`, then
mapped using the exact pinned TypeScript compiler's source-map segments.

```
# First produce a complete same-Rust-tree native checkpoint, with clean Git state:
cd rust && cargo xtask oracle
cd ..
python3 docs/port-plan/tools/ci_report.py evidence --candidate "$(git rev-parse HEAD)"
# Run from the repo root; final argument is that full native checkpoint SHA:
fnm exec --using=22 node docs/port-plan/oracle/original-coverage.cjs rust/target/original-coverage CHECKPOINT_SHA
fnm exec --using=22 node docs/port-plan/oracle/original-coverage-report.cjs rust/target/original-coverage/raw.json rust/target/original-coverage
fnm exec --using=22 node --test docs/port-plan/oracle/original-coverage.test.cjs
```

The report run records its own tooling SHA separately from its native input/result
checkpoint, and requires their entire Rust tree to agree. It rejects stale raw
runs, missing/extra identities and any changed output/sidecar. The same 1422
inputs from ten native suites replay through the existing complete adapters in
fresh VM contexts. Only input bytes enter each adapter. Its source Git reads are
cached by immutable pin/path, stdin/stdout/stderr use bounded corpus buffers, and
actual Node Uint8Array/TextDecoder/fflate and the complete TypeScript module are
used. Full original outputs, error/progress sidecars and source exit statuses
must match the independent original artifacts from the native gate. Binary STL
also reuses its independent count.txt, never inferred from outputs. Existing
STL manual annotation erasure is replaced by full compiler erasure at the same
helper/dispatcher entrypoint only for this coverage replay; all values/sidecars
are checked. No original source file or mathematics changes.

Eight modules map: STL, PLY, OBJ, OBJ texture helpers, formats, scanFiles, command
error types, schema validation. All compiled module source/maps and precise V8
ranges,1422 input/output/sidecar identities and tool versions are in raw.json.gz.
The report is regenerated from raw data, never hand-edited. It lists every mapped
unreached line and every zero-count observed block/function. Whole file scope
means unported image ingress/schema builders can appear as unreached; line reach
does not establish that every field/state/branch is compared. Full original pixel
comparisons from the separate Chrome audit are deliberately not folded into this
native diagnostic. Unmapped stripped geometry/helper sources are explicitly
unknown; each affected implemented unit carries the finding link.

Metrics: an original line is reached if any compiler-mapped generated segment has
positive count in its smallest enclosing V8 function/block range. A partial line
has both reached and unreached segments across the runs. Branch records are the
union of V8 detailed block ranges observed in those runs. **This is not a static
branch/condition/MC-DC denominator**: an entirely uncalled function may only have
one zero-count range and no internal branches. Block offsets, mapped positions,
counts and function roots remain visible. No percentage is an acceptance gate.
Independent fuzz campaigns, browser-only work and unmapped helper source branches
remain open; this report does not claim full original coverage.

The prototype initially missed the real TextDecoder global at the adapter VM
boundary and failed immediately, before claiming a report. Adding the normal
Node global fixed the harness; no fixture/comparator expectation changed. Five
real Node tests cover source identity/hashes, full regeneration, nested zero-block
precedence, changed counts and missing/duplicate/empty/malformed artifacts.
