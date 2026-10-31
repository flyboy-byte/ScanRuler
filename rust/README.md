<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# rust/: the pure-Rust port

Work in progress, 2026-10-31. Nothing here is ported yet; see [the guide](../docs/guide/README.md)
and [the plan](../docs/port-plan/HANDOFF.md).

| Crate | Role | Contents today |
|-------|------|----------------|
| `port-core` | Pure core: no DOM, files or global state | Units, JSON-safe ids, rigid transforms, the reference pin |
| `port-engine` | Session, commands, jobs, history | Empty |
| `port-io` | Formats and the `.scanruler` project reader/writer | Empty (binary STL first, unit P05) |
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
