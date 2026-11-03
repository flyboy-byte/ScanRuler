# Retaining owned-data results

2026-11-03. PR #81 ports keptResult over the existing schema_value::Value domain:
undefined/null/booleans, binary64 including nonfinite values, UTF-16 strings,
dense arrays, unique enumerable own fields and Uint8Array data. It reuses the
existing value and number/key-order implementations, adding no second data schema.
Root undefined has no result. Bytes become their count string, object undefined
is removed, array undefined/nonfinite numbers become null, and negative zero
becomes positive zero. A fresh tree is retained only when its JSON text has at
most 16,000 UTF-16 units. Omission preserves the exact full count and source text.

413 independent source-first cases compare the original keptResult against Rust:
full value tags, exact number bits, key/member order, UTF-16 content and omission
messages. Both outputs are recorded before/after recursively mutating the original
input, proving retained values are independent copies. Inputs include four stable
binary64 seed streams, decimal/exponent boundaries and adjacent floats, lone/paired
surrogates, every control character, prototype-like and array-index keys, large byte
arrays, nesting and exact threshold neighbors. Every numeric case also receives a
large-array probe exposing its JSON length through the omission message. All 389
prior records remain byte-exact after appending the final numerical boundary probes.
Full pinned source graph/map/provenance and paired artifacts are retained.

The implementation counts source JSON characters directly while producing the
normalized tree. Quoted strings count escapes and UTF-16 pairs, not UTF-8 bytes.
Native node/input-unit/encoded-unit/depth budgets and duplicate keys return separate
host errors, never the original non-JSON omission. A hard depth ceiling protects
native stack use. Tests check resource refusals remain separate and exactly 16,000
characters stay retained while 16,001 are omitted. Ordinary bounded standard-library
metadata/formatter allocations are not process-wide OOM isolation.

This is the owned acyclic data branch. Arbitrary host getters, custom toJSON,
prototypes, functions, BigInt and cyclic/shared JS object conversion are not this
input domain; the original exception/non-JSON classification branch is not ported
here. Do not treat a host refusal as that branch or accept the full session on this
basis. Recent-outcome lifecycle, complete command runtime and browser integration
remain separate. Run cargo xtask ci/oracle from rust/; all older formatter/schema
callers and numerical cases stay required. No dependencies or original files change.
