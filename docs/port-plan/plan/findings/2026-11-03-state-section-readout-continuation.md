# State-section readout is read, not ported

2026-11-03. Pinned src/commands/stateSections.ts uses two ordered maps. Duplicate
registration throws before insertion; cleanup compares describe function identity,
so an old cleanup can remove a later registration of the same function. Reads
iterate the live selected map, catching each callback fault. Error.message is read
without String conversion for actual Error objects; other values use String(e).
Failure of that access/conversion propagates out of readAll's catch. Preserve
partial callback/registry effects rather than extending the catch.

Output is a plain {} object assigned with out[id]. '__proto__' therefore has source
prototype-setter behavior, not ordinary own-map insertion. Define and compare the
full typed/serialized output boundary before implementation; arbitrary JS host
objects/proxies are not silently equivalent to an owned Rust map. No body, source
comparison or acceptance is claimed by this inspection.
