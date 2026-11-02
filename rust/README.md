<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# rust/: the pure-Rust port

Work in progress, 2026-11-01. The binary STL triangle decoder is the first bounded port; see [the guide](../docs/guide/README.md)
and [the plan](../docs/port-plan/HANDOFF.md).

| Crate | Role | Contents today |
|-------|------|----------------|
| `port-core` | Pure core: no DOM, files or global state | Welding, bounds/centroids, normals/orientation, dense solves, TLS/clipped planes; initial units/IDs/transforms and pin |
| `port-engine` | Session, commands, jobs, history | Empty |
| `port-io` | Formats and the `.scanruler` project reader/writer | STL binary/ASCII dispatch and triangle extraction (P06); no welding/session loader |
| `port-cli` | `scanruler-rs`, native front end for differential runs | Prints a banner |
| `xtask` | Project tasks | `ci`, `pin`, `plan`, `wasm`, `env` |

```bash
cargo xtask ci     # the fast gate; same thing CI runs
cargo xtask env    # toolchain versions to record with any result
```

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
until it exists. A green `ci` run is not parity.

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
and no Rust UI or browser-Wasm runtime parity is claimed. No child unit is accepted.
