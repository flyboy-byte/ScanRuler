<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# rust/: the pure-Rust port

Work in progress, 2026-11-04. Native scan opening and plane fitting are available through the engine
and `info` / `fit-plane` CLI (M3a/M3b); see [the guide](../docs/guide/README.md) and
[the current M3 order](../docs/port-plan/ROADMAP-TO-USABLE.md).

| Crate | Role | Contents today |
|-------|------|----------------|
| `port-core` | Pure core: no DOM, files or global state | Full mesh-graph construction, adjacency/components/selection, bounds/centroids, normals/orientation, dense solves, TLS/clipped planes and curvature discrimination; initial units/IDs/transforms and pin |
| `port-engine` | Session, commands, jobs, history | Runtime/task owner, real scan document, STL/PLY scan.open, queries/marks and plane fitting |
| `port-io` | Formats and the `.scanruler` project reader/writer | STL, PLY, OBJ/MTL, inflate/zip and 3MF libraries; compatible project IO remains open |
| `port-cli` | `scanruler-rs`, native front end for differential runs | `info` and `fit-plane` use registered engine commands |
| `port-wasm-probe` | Private bounded browser-runtime test ABI | Bounded original/native/browser comparison ABI; not a product UI or engine integration |
| `xtask` | Project tasks | `ci`, `oracle`, `pin`, `plan`, `wasm`, `wasm-probe`, `env` |

```bash
cargo xtask ci     # the fast gate; same thing CI runs
cargo xtask oracle # all native source suites; fails if a required suite disappears
cargo xtask env    # toolchain versions to record with any result
```

## Open a scan natively

From `rust/`:

```bash
cargo run -p port-cli -- info /path/to/scan.ply
cargo run -p port-cli -- info /path/to/scan.stl --units cm
```

STL, ASCII/little-endian/big-endian PLY and standalone OBJ are supported.
OBJ faces (including polygon fans and negative indices) use the real engine
import path; `fit-plane` also accepts OBJ. Companion images are not loaded by
`scan.open`; an OBJ that names a missing texture retains the original notice.
3MF and STEP command wiring remain unavailable. JSON on stdout includes
filename, source units, vertex/triangle/component counts, bounds, center and size
in millimetres. Errors go to stderr with exit code 2. Node is needed only by the
comparison oracle, not the native CLI. Input bytes and producer work are bounded.
Fit a scan's full surface, or query/mark a box before fitting:

```bash
cargo run -p port-cli -- fit-plane /path/to/scan.ply --sigma 0
cargo run -p port-cli -- fit-plane /path/to/scan.ply --box 0 0 -1 3 4 1 --marked --orient XY --extend 1 2 3 4
```

Box coordinates and extensions use mm. `--normal x y z` and `--within-deg` filter
box/query selections. `--in-plane` changes the relation used with `--orient`.
The engine owns all selection, fitting, finishing and history recording. Global
undo/redo and compatible project save/read are subsequent M3 steps; no `--undo`
or `--redo` switches are exposed yet. Required suite registration and broader
acceptance remain open under roadmap section 0.

## Rules of the workspace

- **No third-party dependencies yet.** Each one must pass the licence policy in
  `../CONTRIBUTING.md` and the purity audit (P35): a Rust crate that wraps C/C++ does not count.
- `unsafe` is forbidden; `unwrap`, `expect` and `panic` are lint warnings (errors in the gate)
  outside tests.
- Everything in `port-core`, `port-engine` and `port-io` must build for `wasm32-unknown-unknown`.
- Keep crates few. Split only when build time or dependencies justify it.
- The toolchain is the system one (no rustup). Record `cargo xtask env` output with results.

## What `ci` does not do

It does not compare anything with the original app. That harness (`cargo xtask diff`,
`diff-ci`) is planned under P36 to P39, and the disabled example workflow is designed to fail
until it exists. A green `ci` run is not parity. `cargo xtask oracle` separately discovers all registered
required native oracle entrypoints, rejects missing/duplicate discovery, then runs
all ignored core/IO integration tests without filtering their cases. It requires
Node and the frozen Git object. It does not implement the general P36 ratchet or
browser-Wasm execution. CI retains every existing raw-output artifact.

## First import operation

`port_io::stl::read_binary_triangles` ports the original `parseBinary` helper:
explicit triangle count, owned f32 coordinates, progress callbacks and a caller
triangle budget. The separate dispatcher below adds format detection and ASCII parsing; welding
and session loading remain open under P06.

Run the independent live reference comparison (Node is oracle-only):

```bash
cargo test -p port-io --test binary_stl pinned_original_comparison -- --ignored --nocapture
```

This requires Git history containing the frozen reference commit. CI uses Node 22.
The ordinary fast gate explicitly ignores this test when the oracle is unavailable;
CI runs it separately and fails on missing tools or source. Native comparison is
not browser-Wasm runtime parity; the Wasm library is compile-checked only.

`port_io::stl_dispatch::read_stl` now provides the complete STL dispatcher and
ASCII extraction, with explicit byte/triangle budgets. It retains the original's
permissive `vertex` matching, numeric-prefix parsing and binary-before-text rules.
This returns an unwelded triangle soup; mesh indexing/session loading remain open.

```bash
cargo test -p port-io --test stl_dispatch pinned_stl_dispatch_comparison -- --ignored --nocapture
```

This second native suite compares 82 cases (69 successful outputs, 13 original
refusals), including Unicode whitespace, malformed UTF-8, IEEE numeric boundaries
and the 512-byte prefix cutoff. Its raw evidence is under
`rust/target/stl-dispatch-diff/` and uploaded separately in CI.

## Geometry and fitting audit

Current operations and exact test/evidence identities live in the canonical units
under docs/port-plan/plan/units. Start with the [ordered audit queue](../docs/port-plan/AUDIT-QUEUE.md)
for parent-first PR review, source symbols, reports, known gaps and complete run commands.
These are partial library ports: port-engine is still empty, the CLI is a banner,
and no Rust UI or whole-product runtime parity is claimed. Bounded Chrome execution for three library operations is documented below. No child unit is accepted.

## Curved fit versus plane

`curvature_gate::rms_over` retains finite residual filtering, selection/callback
order, duplicates and overflow behavior. `curved_beyond_plane` reuses TLS and the
source factor 1.1; a refused plane continues with true. NaN/negative/infinite
curved RMS values retain source comparison semantics. Invalid shape/selection
fails before callbacks. This helper does not fit a sphere/cylinder or provide a
user-facing command. Eighty independent source cases compare exact bytes,
including adjacent floating-point values at the decision boundary.

## Vertex adjacency

`adjacency::build_adjacency` ports the complete vertex CSR builder: every input
triangle appends two neighbors per corner, with repeats and source order retained.
Explicit vertex/neighbor budgets, complete-triangle/index guards and fallible
reservations bound storage; source offset arithmetic is exact in the admitted
u32 range. Fifty-six source cases compare both complete arrays, including four
fixed seeds. A local pipeline check feeds the existing weld output into adjacency.
Region growth, source/render mapping and command/session integration remain open.

## Directed connected components

`components` ports `findRoot`, `makeUnionFind` and `vertexComponents`: path halving
and root directions remain observable in read-only parent state. Factory unions
put a's root under b's; mesh triangles put second/third roots under the first.
Standalone invalid/cyclic chains and invalid union indices fail before mutation.
Storage is fallible and capped by caller budget and signed Int32 parent capacity.
Fifty-one independent source fixtures compare complete parent state and every
query result after each operation. Browser-Wasm execution and downstream shell/
region/command integration remain open.

## Mesh selection preservation

`mesh_selection::triangles_within` keeps complete triangles whose three corners
are marked, retaining order/duplicates and skipping incomplete trailing triples.
Out-of-range marks are ignored and triangles with out-of-range corners excluded.
`remap_vertices` retains input order/repeats while dropping negative/missing map
entries. Fallible storage and explicit vertex/output budgets bound results.
Sixty-two source fixtures compare complete output arrays exactly. Undo/project/
edit commands and browser-Wasm execution remain open.

## Full mesh-graph construction

`mesh_graph::build_mesh_graph` composes welding, index-degenerate filtering,
adjacency, normals, optional outward orientation and the source bounding diagonal.
It preserves validation/message and progress order, including validating supplied
soup indices before ignoring them and retaining adjacency built before winding
flips. Typed parsed input transfers owned arrays; caller limits bound raw vertices,
indices and neighbors. Fifty cases compare complete graph/error/progress output
exactly, including 36 successful graphs. STL command/session integration, metadata
handling beyond this source body remain open. The fixed graph corpus executes in Chrome under the runtime harness below.

## Closed-shell parity

`closed_shells::closed_shells` retains the original two-word edge-parity criterion,
including private exact-bit welding that canonicalizes signed zero. This differs
from the general mesh welder. Even-edge parity is probabilistic and does not
guarantee manifoldness; repeated/degenerate triangles keep original outcomes.
Incomplete coordinate/index tails are ignored. Explicit budgets, fallible storage
and consumed-index guards bound the host. Sixty-four source fixtures compare
flags/count, private welded IDs and exact hash words, including raw NaN payloads.
Browser-Wasm execution and viewer-cap/other downstream integration remain open.

## Marked-selection plane elements

`marked_plane::fit_plane_on_selection` completes the original marked-plane
continuation: minimum twenty marks, clipped fit, surface-facing normal, sampled
minimum-area rectangle, projected center, oriented residual extremes, full
statistics and the original ordered region. Extents and normal votes use every
original mark even when clipping reduces usedPoints. Fifty-one independent source
cases cover the complete result and fixed-angle probes under a preregistered
comparison budget. This corpus also executes in Chrome; seeded region growth,
public commands/history and broader runtime coverage remain open. See fit.plane.selection and its evidence report.

## Actual browser-Wasm execution

`cargo xtask wasm-probe` compiles a safe private buffer/word test ABI with a hard
64 MiB linear-memory maximum. The required Chrome CI lane runs the existing TLS
plane, full mesh graph and marked-plane inputs against original/native/Wasm data.
The verified 114-case scope and immutable report are in
[the runtime guide](../docs/port-plan/oracle/BROWSER-WASM.md). This narrows the
compile-only gap for those operations; it does not implement UI/session commands,
all-unit runtime parity or a stable production ABI. Node smoke is diagnostic only.
