<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Command schema validation comparison

Unit `engine.schema.validate`; reference `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.
The entire pinned schema.ts is annotation-erased with its pinned TypeScript
compiler, evaluated unmodified under Node22 with the real Uint8Array constructor.
No original answers enter Rust. No imported product dependencies/stubs are needed.

697 fixed inputs: 420 for all 140 real command schemas, 128 seeded mixed-value
schemas (32 per seed1/7/99/1039), four 64-double seeded number/metadata schemas,
and 145 authored type/guard/coercion/oneOf/names/id/prototype/UTF16/budget probes.
The original 659-case capture is immutable; the expanded capture retains it
byte-for-byte. No input or passing comparison is replaced by the expansion.

Input is schema Value, input Value, path UTF16. Value tags:0undefined,1null,
2false,3true,4f64 little endian,5UTF16 string,6array,7object,8Uint8Array. Lengths
are u32 little endian; strings count u16 units; arrays count values; objects count
key/value pairs. Data is dense, acyclic, with unique own enumerable keys.
Objects have ordinary prototype behavior for the validator's lookups; arbitrary
JavaScript executable objects/prototypes and shared-reference identity remain
outside this owned ingress contract. See the value decision and public API.

Output VLD1: fault field count (0 or2, then full UTF16 name/message), refusal-line
count and full ordered UTF16 lines, complete coerced Value, then four u32 fields:
weight.bytes, weight.depth, budget.bytes, budget.depth. The source preserves the
empty-oneOf TypeError and mutation state; it never becomes a successful schema
validation. Top-level strings are not coerced. Errors never replace value checks.

The batch oracle receives u32 length + input frames and emits u32 length + output
frames. All frames are supplied before interpreting results; missing/trailing
results fail. Native raw per-case input/original/Rust/stderr plus candidate and
identity are retained in rust/target/schema-diff/run-*. A wrong actual error-line
unit and a missing output tail are rejected. Source-only capture defaults to
rust/target/schema-source.json; use --out only to add a new immutable observation.

```
python3 docs/port-plan/tools/capture_schema_source.py
fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test schema_validation pinned_schema_comparison -- --ignored --exact --nocapture
cd rust && cargo xtask ci && cargo xtask oracle
```

This suite adds697 observations to the mandatory native roster (26/2015). Browser
runtime stays651 cases; this module is Wasm compile-checked only. No session,
registry, command scheduling/history, UI or acceptance status is supplied here.
