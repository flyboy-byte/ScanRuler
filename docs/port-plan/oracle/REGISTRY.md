<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Registry and admission comparison

Unit `engine.session.registry`; pinned original2cd89c6.210 deterministic operation
scenarios include all140 real command definitions, one whole140-command catalog,
name syntax/order, duplicate/partial registration, repeat cleanup/reregistration,
headless/metadata presence, schema8000/8001 and depth12/13 boundaries, defaulted
and refused/coerced input, original empty-oneOf fault and32 seeded scenarios
(eight per seed1/7/99/1039). This does not implement140 command bodies.

`build-session-registry.mjs` verifies original package/lock identities, archives
the pin to ignored target, and Vite-bundles the actual registry and its real
history/activity/schema/store dependencies. No reached behavior stub or original
source edit. Callback fixtures echo independently supplied arguments and identify
the body key. Each case imports a fresh registry module; calls await its original
runCommand result. The Rust side compares its explicit admission stage; queue,
scene, history, timeouts, recent jobs and application command bodies remain open.

Input is one owned Value (same binary tags/UTF16 as SCHEMA.md) with defs and ops.
REG3 output is a Value list of full operation states: value, error, afterInput,
target, commands, headless. Errors include full name/message/code presence. Each
catalog row retains all schema metadata and exact field order/presence; headless
flags cover every current name. Empty catalog states are valid, empty scenarios
are not. Batch transport uses u32 byte lengths, no trailing/missing frames.

Independent source captures REG1/REG2 are kept losslessly; dropping the newly
added fields from REG3 reproduces them byte-for-byte. No previously compared
field/case/seed disappears. Compressed captures retain every byte and digest.
A changed actual metadata title, coerced boolean, error-line unit and missing
tail fail comparisons; native raw input/original/Rust/stderr remain under target.

```
fnm exec --using=22 node docs/port-plan/oracle/build-session-registry.mjs
python3 docs/port-plan/tools/capture_registry_source.py
fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test registry pinned_registry_comparison -- --ignored --exact --nocapture
cd rust && cargo xtask ci && cargo xtask oracle
```

The public Rust API owns definitions and coerces mutable caller input even on a
source refusal. Callback keys and repeatable cleanup tokens are opaque, not a
new wire ABI. Explicit host quotas and owned typed-data scope are documented in
the session boundary decision. wasm32 compilation is not browser execution.
No session workflow, M3, registry acceptance or locale-wide parity is claimed.
