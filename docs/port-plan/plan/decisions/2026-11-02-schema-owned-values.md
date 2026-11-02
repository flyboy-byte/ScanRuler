<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Owned validator data boundary — 2026-11-02

Unit: `engine.schema.validate`; owner/review: Codex PR #42, integrator review outstanding.

DESIGN: use dependency-free `port_engine::schema_value::Value` for this pure
function. Preserve Undefined, Null, Boolean, IEEE f64 (including nonfinite and
negative zero), UTF-16 strings/keys, dense Array, unique-key Object, and Bytes
(the Uint8Array distinction). Metadata remains in the schema so weight measures
all of it. Objects store insertion order; enumeration visits array-index keys
first as JavaScript does. Validation copies input and returns every error plus
the mutated copy. Earlier oneOf branches mutate the same copy later branches see.
The session owner can use or adapt this boundary; this PR implements no session
store, command registration, wire ingress, scheduler or history.

DOCUMENTED: pinned schema.ts uses ordinary Object.prototype lookups for required
fields and `properties?.[key]`. The twelve built-in Object.prototype names are
modelled as present inherited values/untyped schemas where appropriate. Own
undefined still shadows inherited values. Getters, proxies, functions as data,
custom prototypes, cyclic/shared-reference graphs and array holes are outside the
owned transport-data boundary. Duplicate keys and malformed schema types are
host InvalidData, not invented validation lines. The exported API documents this.

DOCUMENTED: SCHEMA_BUDGET is bytes=8000,depth=12, but bytes means JSON.stringify's
UTF-16 length. Defaults, ignored schema metadata, undefined omission, binary
numeric fields, number formatting and nesting all affect that measurement. The
budget is returned; registry enforcement belongs to the session owner. Empty
oneOf throws the original TypeError; the result keeps that source fault distinct
from schema lines and host limits, including any input mutation before it.

DESIGN: host preflight/depth/work/output limits are separate from the source
budget, and a host refusal publishes no partially coerced value. Output helpers
check generated-unit quotas, owned-copy vectors reserve fallibly. Rust's normal
small string formatting and sorting may still allocate; allocation failure is
not an exhaustive recoverability guarantee. No schema library or other crate is
added. P35 is not completed by this dependency-free slice.

EVIDENCE: immutable source-only capture precedes the initial Rust body; expansion
preserves all first 659 observations and adds 38 adversarial probes. All 697
observations compare full error lines, typed/coerced values, weight/budget/fault.
The 72 checked-in MCP schemas plus 68 plugin schemas from the validated original
all-plugin browser capture each have minimal/full successful and null refused
inputs. Plugin snapshot schema key order is the capture's stored order; both
engines receive those exact schemas. This is schema-function parity on that input
corpus, not proof the live Rust registry has 140 working commands. No actual
browser-Wasm runtime comparison or independent fuzz acceptance is claimed.
