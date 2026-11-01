<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# STL dispatcher and ASCII port — 2026-11-01

Unit `io.stl.dispatch`, parent P06. Tested implementation: `77683bf515590ac3f5cb8a9c3552ec64700ce44a`. Base: `a8c0bb121e0ff5d75e933e102bc5469e869cfeac` (binary decoder PR #2). This contribution is a separate PR stacked on #2; it does not merge either PR.

Frozen source: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/parsers/stl.ts`, source SHA-256 `5d010557ac00c17aa8c95c54137e6c2a0fd24b8a6426e283891f2323e7392013`.

## Bodies and contract

`stl_dispatch::read_stl` ports the complete `parseSTL` dispatcher and `parseAscii` continuation, reusing `read_binary_triangles` for both binary paths. The caller provides byte/triangle budgets. Output is owned f32 XYZ triangle soup, not an indexed mesh or mutated session.

Preserved behavior:

- Exact header count/byte size wins before ASCII detection, including `solid` binary headers and empty binary files.
- Otherwise, only the lossy-decoded first 512 bytes are inspected for the whitespace/`solid` prefix; non-ASCII inputs of at least 84 bytes fall back to the complete binary record count.
- ASCII progress is emitted before decoding/extraction, including before original ASCII refusals. Case-sensitive `vertex` is unanchored, just as in the source regex; incomplete matches do not consume subsequent potential matches.
- TextDecoder-style UTF-8 replacement and initial BOM handling, ECMAScript whitespace, and parseFloat decimal/Infinity prefixes are preserved. Conversion is decimal to f64 to f32, including signed zero, overflow, subnormals and invalid-token NaNs.
- Original refusal messages are exact. Rust adds explicit byte/triangle/allocation limits; it does not reject non-finite coordinates or enforce additional STL structure.

All text/output growth uses fallible reservations. Dispatcher size arithmetic remains valid for wasm32 instead of overflowing at a large u32 header count. Resource-limit refusals are separate host behavior, tested without treating them as source parity.

## Independent paired execution

The oracle adapter reads the frozen Git object, removes only the known TypeScript signatures/type declarations and executes the original bodies. Fixtures are generated from fixed text/bytes and LCG seeds 1, 7, 99, 1039. Rust runs independently before the source. Original output never enters the Rust call.

Comparator `io.stl.dispatch.bits.v1` compares complete little-endian float32 output bytes, lengths, UTF-8 progress and refusal messages, with no numeric tolerance/normalization. Result-kind `soup` is also checked by the oracle adapter. The fixed schedule has **82 cases: 69 successful outputs (66 nonempty), 13 source refusals**. Refusals are separate from successful value coverage.

Coverage includes binary precedence/fallback/trailers, short/empty inputs, uppercase and embedded keywords, regex token consumption, malformed ASCII, all 25 ECMAScript whitespace characters and four excluded controls, BOM/malformed UTF-8, 512-byte prefix cutoffs, numeric-prefix/exponent edge cases, float32/float64 rounding, non-finite values and four seeded 19-triangle ASCII inputs. The existing binary helper suite remains unchanged and passes all ten successful comparisons plus its separate truncation case.

`2026-11-01-stl-dispatch-cases.csv` stores input/output/progress identities and results. Manifest SHA-256: `844c5e7d186f1b8b4426a9a40a418081ee88b881e5feead48c73c854d2b4733d`.

Raw exact-commit inputs and both outputs: `rust/target/stl-dispatch-diff/run-1793564030375226705/` (identity records clean source, candidate SHA, Node and Rust versions). Binary regression raw outputs: `rust/target/stl-diff/run-1793564028694719019/`. Runs have distinct directories; prior failures are retained. CI uploads the new `stl-dispatch-comparison` artifact for seven days, separately from binary helper artifacts. Regenerate expired artifacts from the identified commit rather than treating hashes as a substitute for execution.

## Gate and comparator check

Commands run successfully at the tested implementation commit, system Rust/cargo 1.99.1, Node 20.20.2, Linux 7.2.11-zen1-1-zen:

```bash
cd rust
cargo xtask ci
cargo test -p port-io --test binary_stl pinned_original_comparison -- --ignored --nocapture
cargo test -p port-io --test stl_dispatch pinned_stl_dispatch_comparison -- --ignored --nocapture
```

Fast gate: fmt/clippy, eight ordinary workspace tests, live-plan checks, nine invalid-state checker mutations and wasm32 library compilation. Both oracle tests are explicitly ignored by the ordinary gate and then executed separately. Hosted CI also passes: [run 41346035132](https://github.com/flyboy-byte/ScanRuler/actions/runs/41346035132), PR head `458d9d8e10c30c55feb66532649976e5629ef7b3` merged with base `a8c0bb1` for testing. Node 22.23.4 ran both native suites, including all 82 dispatcher cases. The uploaded dispatcher artifact zip SHA-256 is `6ab2896e854928b9c8db07ba6116b60fa3e70da9f49e0f98510aba90989e481e`.

A temporary implementation mutation added 1.0 to decoded ASCII coordinates. The live comparator rejected `ascii_triangle` for changed coordinate bytes (exit 101). The original implementation was restored before the implementation commit and the clean successful run. Failure raw outputs remain under `rust/target/stl-dispatch-diff/run-1793563963998136844/`; its identity records the dirty pre-commit tree. This is one focused comparator sensitivity check, not a claim that the general P36 mutation/coverage harness is complete.

## Open work

Status stays **implemented**, not accepted. Browser-Wasm runtime comparison, G0/P05 prerequisites and the general P36 harness/ratchet remain open. No welding, topology, normals, engine command, project or UI integration is included. Next bounded source operation: `io.mesh.weld` (`weldTriangleSoup`/`hash3`/`filterDegenerateTriangles`), preserving pinned exact-bit vertex identity and order.

## Separate original-app CI failure

PR #2's unchanged original-app suite failed `plugins/reverse/tests/reNetworkCreases.test.ts:209` on the 1,500 ms wall-time threshold: 1,516.53 ms initially, 1,894.01 ms on one rerun ([run 41340398062](https://github.com/flyboy-byte/ScanRuler/actions/runs/41340398062)). The original source/tests are unchanged in either parser PR. These are observed timing failures, not proven harmless flakes; no test budget, comparison or source expectation was loosened. The new PR's general original-app CI is still separate from the passing Rust gate.
