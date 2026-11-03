# Plugin state-section readout

2026-11-03. PR #87 ports registerStateSection, readStateSections,
readStateBriefs and readAll over shared owned data and lazy fallible conversion
providers. The source module is complete and unmodified in the reference graph.

Both registration maps preserve insertion order and live traversal. A duplicate
ID throws the exact ordinary error text before either map changes. Cleanup only
deletes the currently matching describe function; a stale cleanup cannot remove
a different replacement, but can delete a later identical function. Dropping a
cleanup does nothing. Deleting/reintroducing or appending during a read affects
that same traversal. Briefs are independently optional and never fall back to the
full describe function. No mutable map borrow spans a reader/conversion callback.

Successful reader values keep their composite reference identity. Error.message
is read lazily and kept as-is (including undefined/non-string values); other
thrown values undergo lazy String conversion. A failure in either conversion
escapes readAll unchanged and skips later readers. These are conversion providers,
not a Display normalization. Error wrappers are fresh ordinary objects.

Readouts model the fresh ordinary object's own properties and prototype separately.
Numeric own keys sort first. An assignment to __proto__ never becomes an own map
entry: null clears the prototype, composite values become the exact shared
prototype, and other primitives are ignored. Error wrappers under that key become
the prototype too. This scope does not execute inherited setters/getters or model
a general prototype/property-lookup runtime for arbitrary host objects.

192 original-first independent traces compare complete own-key/value/prototype,
returned object/array identity, duplicate messages, cleanup, describes/briefs,
conversion faults and every callback effect. Appended __proto__ categories retain
all source bytes of the first128 passing traces. Source outputs never become
native inputs. Full source/map/provenance and paired artifacts are retained.
Native controls cover borrowed non-Clone conversion faults, recursion and
metadata/visit admission. Slots bound live entries plus traversal tombstones;
callback visits include nested reads. Host failures do not become plugin errors.

The existing owned history Cell carries values without duplicating the business
state. Full session.state composition, actual plugin installation/native session
ownership, arbitrary host objects/custom prototypes and browser-Wasm readout
execution remain open. Wasm compilation and the existing native gates are required,
but do not establish runtime acceptance or M3.
