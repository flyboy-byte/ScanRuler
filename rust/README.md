<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# rust/: the pure-Rust port

Work in progress, 2026-11-01. The binary STL triangle decoder is the first bounded port; see [the guide](../docs/guide/README.md)
and [the plan](../docs/port-plan/HANDOFF.md).

| Crate | Role | Contents today |
|-------|------|----------------|
| `port-core` | Pure core: no DOM, files or global state | Units, JSON-safe ids, rigid transforms, the reference pin |
| `port-engine` | Session, commands, jobs, history | Empty |
| `port-io` | Formats and the `.scanruler` project reader/writer | Binary STL triangle decoder (`io.stl.binary`, under P06); no format dispatcher yet |
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
triangle budget. It does not yet detect file format, read ASCII, weld a mesh or
load a session. Those continuations remain open under P06.

Run the independent live reference comparison (Node is oracle-only):

```bash
cargo test -p port-io --test binary_stl pinned_original_comparison -- --ignored --nocapture
```

This requires Git history containing the frozen reference commit. CI uses Node 22.
The ordinary fast gate explicitly ignores this test when the oracle is unavailable;
CI runs it separately and fails on missing tools or source. Native comparison is
not browser-Wasm runtime parity; the Wasm library is compile-checked only.
