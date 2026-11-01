# Native oracle regression gate — 2026-11-01

Implementation: eeb21367b4bb0c7a27dd02910cba10201dd5a9e1. Reference: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7.

`cd rust && cargo xtask oracle` first checks the pin, discovers ignored core/IO integration tests, requires all eleven known entrypoint names without duplicates, records the toolchain, then runs all ignored integration tests with no-fail-fast. Unexpected additional tests also run. Four regression tests cover complete discovery, empty/unrelated output, each missing entrypoint and duplication. No adapters, fixtures, seeds, numerical budgets or source bodies changed.

Validation at this exact implementation revision: `cargo xtask ci` and `cargo xtask oracle`. The fast gate includes 21 ordinary tests, formatting/lints and Wasm compilation. Eleven oracle entrypoints exercise 272 existing case identities including refusals, not 272 successful numerical results. Each suite retains its independent source/Rust inputs and raw output hashes. Local Node 20.20.2; hosted workflow provisions Node 22 and retains every existing per-suite artifact upload.

This is a bounded contributor tool, implemented rather than accepted. It is not the generic P36 paired protocol, per-case coverage ratchet, browser-Wasm runtime test or engine command acceptance. Discovery guards entrypoint names; deleting cases inside a suite still requires the independent case/coverage audits. See AUDIT-QUEUE.md for stack order and known separate original-app timing failure.
