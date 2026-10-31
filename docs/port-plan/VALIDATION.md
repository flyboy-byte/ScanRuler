# Evidence and acceptance contract

All checks below are planned unless an evidence record says otherwise. No upstream or port test results exist in this packet. Planning consistency checks do not establish application correctness.

## 1. Establish the actual reference

In a full checkout at the pinned commit, use Node 22 and the lockfiles. Run the commands from upstream CI:

```sh
npm ci
node scripts/check-plugin-boundary.mjs
node scripts/check-units.mjs
npm test
npm --prefix mcp ci
npm --prefix mcp test
npm run build
CHROME=/usr/bin/chromium npm run e2e:ci
npm run build:engine
```

The browser command above assumes the observed Linux Chromium path; use a recorded supported path elsewhere. Baseline records include commit, lock hashes, OS, CPU/RAM/GPU, Node/browser versions, command, exit status, duration, peak memory where measurable, and actual skipped tests. Run the separate dependency-license workflow checks, including the exact pinned meshstep exemption, when verifying dependency changes; read that workflow rather than reconstructing its allowlist from memory.

CI browser coverage is selective: six core development-server suites, plugin-selected checks from `ci.json`, and a built-production smoke test. It does not run every browser script. Preserve the baseline set and add relevant feature-specific scripts from the inventory. The MCP helper is a separate package with its own tests and lockfile; `npm test` at root is not enough.

Capture live registries for core-only, all plugins and original headless mode, plus plugin feature schemas and runtime flags. The saved 72-command snapshot lacks plugin descriptors and headless flags. The 140-row static candidate list must be reconciled, not blindly adopted. Preserve the captured JSON and hash; specify unsupported-in-headless separately from missing commands.

## 2. Oracle strategy and differential harness

Stefan's pinned original code, mathematics and observable behavior are the authoritative target, per the user. Run original-versus-Rust comparisons on independent copies of identical inputs and preserve source formulas/defaults/operation order. Analytic and metamorphic checks supplement port/harness diagnostics; they do not override the original or justify redesign. A discrepancy remains an open port divergence until explained as a port/harness/fixture or reproducibility issue. Any proposed source correction or behavior change is separate reviewed work, not an expectation update in a port PR. See PORTING-PROCESS.md and CI-AND-REGRESSIONS.md.

Original measurement operations need a browser or direct pure-core harness, since the original headless engine excludes scans and viewport operations. Prefer pure-core tests for numeric kernels and browser command replay for stateful behavior. Source-translated tests and fixtures may be used in this standalone AGPL port with provenance. Keep each program/session isolated; never shadow-run two mutations on one document. Store raw results and an explicit comparator version.

Fixture manifest fields: ID, license/source/author, generator version and seed, source/input hashes, units and coordinate frame, raw precision, scan topology/selection IDs, exact recipe, expected values and derivation, numerical budget, applicable platforms/backends, known limitations and skip reason. Public generators must create redistributable fixtures. Private ball-bar/bracket files remain optional until the user supplies permitted copies; their absence is not a blocker for synthetic coverage or a passed acceptance test.

Initial independent corpus:

- Plane and two planes at a known spacing; origin and large translation; exact/noisy/collinear/coincident points; normal-sign ambiguity handled explicitly.
- Sphere/cylinder/cone/torus with known dimensions; partial surface, sparse sample, near-degenerate shape and two neighboring surfaces; fixed IDs before region growth is tested.
- Box/bracket scans with raised/sunken faces, known rigid pose and deliberate scale error. ICP must not absorb scale. Include symmetric shapes with ambiguous valid poses; compare geometric placement/coverage, not only matrix entries.
- Open/inverted/duplicated/degenerate meshes; thin parallel walls, wedge and curved wall; one-sided/open nominal with an explicit unreliable-sign result.
- Flat image with known DPI and anisotropic calibration; translated/rotated/mirrored sheets; no calibration must remain visibly uncalibrated.
- Closed sketch/extrusion with hole, parameter edit, circular pattern legacy versions, mirror identity migration, STEP import and spline/trimmed surface/network cases.

## 3. Comparators: precise without false promises

Discretely specified results (IDs where promised, counts, units, field presence/defaults, error code, history length, calibration flags, format versions and original member bytes) require exact agreement. Do not compare raw session kernel hashes across native/Wasm backends; compare persistent reference behavior and geometric descriptors instead.

Numerical comparisons use a per-field rule such as `abs(actual−expected) <= abs_tol + rel_tol * abs(expected)` with units stated. Derive budgets before port output is observed, using source fixture tolerances, input f32 quantization, algorithm discretization and independent oracle error. Record budgets in manifests. Do not pick one global epsilon or use scanner noise as a blanket excuse for algorithm errors. Fit sigma is not total instrument uncertainty.

Compare full-precision internal results and rounded public reports separately. Include selected-point membership, residual distributions, region boundaries, outlier cutoffs and estimator settings, not only fitted radius. Equivalent axis sign/periodic curve or symmetric pose representations need documented normalization. Nondeterministic sampling uses pinned generators where feasible; otherwise use predeclared outcome bounds and repeat counts. Native parallelism must not move results beyond those bounds.

Backend equivalence measures shape validity, closedness, components, volume/area/bounds, sampled surface distance, intended topology and edit-reference behavior. STEP bytes and triangle ordering generally differ; byte comparison is appropriate for preserved source members, not regenerated CAD exports. Read exports with an independent permitted parser/tool and test named part/color/units where supported.

Keep a compatibility-exceptions record only for separately reviewed changes: baseline behavior, proposed new behavior, rationale, affected clients/files, migration/version requirement and acceptance evidence. Stronger atomicity or different timeout behavior is not part of the default faithful port. Preserve the original contract until such a change is explicitly accepted; do not use a redesign to explain away a failing comparison.

## 4. Persistence, history and hostile inputs

Round trips must cover source scan/nominal/image bytes, OBJ companions/textures, calibration, element recipes/selections/settings, dimensions/tolerances, transforms, sections/sheets, notes/plans, editable-copy maps, parameters, sketches/constraints, all feature kinds, rollback and style/counter IDs. Load legacy reverse versions 1/2/3; save/reopen and edit them. Load/save with one plugin absent, reinstall it and demonstrate its state survived.

Malformed/truncated/missing/duplicate archive members, invalid indices, deep JSON, overflow/nonfinite values, corrupt images and decompression expansion must fail with bounded work and leave the active session intact. Restore should be prepare-then-commit; simulate failure after preparation and verify no half-loaded document. Recovery tests simulate interrupted write/worker termination and restart from the last committed checkpoint.

Every mutating workflow gets execute → inspect → undo → inspect → redo → save → reopen, including a failure/cancel path. Mesh adoption must restore the original and its measurement references on undo. Test stale worker replies after opening another scan, operation timeout followed by a new operation, backend crash and old shape-handle reuse. Long work must not mutate after a reported cancellation/timeout outcome without a documented legacy exception.

## 5. UI, device and MCP acceptance

Each of six workspaces needs visible stories: load/select/preview/commit/edit/undo/export, correct prompts/error/units, busy/cancel behavior, keyboard shortcuts, resize/high-DPI, clipping/split views and accessible text/controls. 2D needs calibrated true-scale export. Reconstructed feature picking must keep references across rebuilds. Capture actual screenshots/recordings; architectural diagrams are not UI evidence.

Development matrix: Linux desktop + Chromium web first. Release matrix: Windows, macOS (Apple silicon at minimum), Linux; Chromium/Edge and Firefox web, plus Safari where upstream features are expected supported. Feature-test WebGPU, provide a demonstrated fallback or declare an explicit unsupported platform in a preview. A missing baseline-supported feature blocks full parity. Touch and SpaceMouse require actual hardware/manual coverage in addition to synthetic events; do not mark their rows passed because CI has a Gamepad mock.

MCP matrix: original helper → Rust browser; Rust helper → original browser; Rust helper → Rust browser; Rust-native headless and declared limitations. Exercise list/call/error mapping, path/binary/image outputs, framing order, malformed packets, pairing/token/origin handling, second client/relay, disconnect/reconnect, version mismatch, long-job progress/recent outcomes, duplicate-name batch rejection, deferred rebuild, cancellation and trace redaction. Capture capabilities from the runtime rather than serving the incomplete core snapshot as authoritative.

## 6. Performance and resource gates

Create fixed small/medium/large corpora, initially targeting approximately 100k / 1m / 5m triangles where valid. Record vertex counts, textures and topology; do not treat triangle count alone as a workload. Measure cold/warm import, index build, fit/selection, ICP, fields, reconstruction edit/rebuild, save/reopen, startup and interactive frame/pick latency; include peak process RSS, Wasm linear memory, JS heap and GPU resources where observable. Record hardware and background load.

Proposed review triggers, to be confirmed after baseline capture: correctness budgets always hard; common operations slower than 1.25× warm reference p95 or peak memory above 1.5× comparable reference require investigation; cancel feedback within 250 ms for cooperative jobs and p95 pointer-to-feedback below 100 ms on the medium corpus are initial goals. These numbers are proposed SLOs, not observed results or universal pass limits. Native and web have separate baselines. Use repeated runs, report median/p95 and distinguish load/compile/cache costs.

A noninterruptible kernel call cannot honestly meet a cooperative cancellation guarantee; stop the host and recover, or disclose the limitation. Memory budgets must include original files plus all workers/kernel heaps and undo peaks. Gate large uploads before allocating, and visibly refuse oversized work. Exact limits are set from measured platform capacity, never guessed to match only a tiny CI fixture.

## 7. Completion evidence

R1 is only an internal application-parity checkpoint. The required R2 endpoint needs every runtime catalog/feature/format/UI row accepted; mandatory native/Wasm/browser/device evidence; all exact/mesh/STEP Rust capability suites passed; source/notices and reproducible artifacts present; legacy project round trips passed; active tooling ported; and an audited transitive/runtime/artifact inventory proving no retained non-Rust geometry, original TS feature logic or Node sidecar. Rust wrappers around foreign geometry do not pass. Retained kernels, missing features or active development-tooling debt mean the user's goal is unfinished.

R1 does not require replacing Markdown, JSON data, YAML CI or browser bootstrap code with Rust. It does require retiring or explicitly accounting for executable repo tooling; development-only Node tools remain listed debt until the whole-tooling scope is closed. No test-suite count alone establishes feature completeness.

Do not label the whole requested repository port finished while that development-tooling debt remains. It may still qualify as a complete R1 application release with a separately incomplete tooling milestone. Reference-only upstream JS tests can be retained in the frozen oracle; active product build/test/release tools need migrated replacements for the whole-repository milestone.
