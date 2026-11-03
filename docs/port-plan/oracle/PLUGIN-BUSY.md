# Plugin busy registration and lookup

2026-11-03. PR #77 ports complete registerBusy/pluginBusy bodies and ordered
busyChecks/busyStores/busyStops metadata from the frozen activity module. It
preserves UTF-16 ids and messages, exact function/array/stop identities, duplicate
refusal before mutation, insertion order, empty-message skipping and swallowed
check errors. A cleanup compares function identity: an old handle removes a new
registration under the same id if the same function is reintroduced. Dropping a
handle does nothing. Delete/re-add appends, and live lookup visits newly appended
entries while skipping deleted future entries. No snapshot replaces that iteration.

96 independently generated authored matrix cases run the full original first.
Full maps, callback counts, errors and first-busy values are compared after every
operation. Cases cover empty/prototype-like/quoted/non-BMP/unpaired-surrogate ids,
null/empty/busy/thrown check outcomes, old/repeated/dropped cleanup, optional stops,
array duplicates and mutation of current/future/new entries during checks. Complete
raw paired inputs/outputs, immutable source map/bundle and generator are retained.
No source outputs are inputs to Rust. The source-map test proves the only change
to the original module is exporting its three private maps for observation.

Native slots retain tombstones to implement source live iteration. Explicit slot,
id-unit, visit and recursive-depth limits return separate host refusals; duplicate
errors retain the exact original UTF-16 message. Returned strings are shared; no
lossy Display conversion occurs. Rc metadata allocation is not process-wide OOM
isolation; supplied callback bodies must be host-owned and bounded. Native callback
Result errors model thrown checks, not process panics or arbitrary JS getters.

Stop handles are opaque and never invoked here: asynchronous stopPluginWork is the
next continuation. watchSession may use current ordered store arrays, but actual
integration is separate. Whole-session executor, browser runtime and acceptance
remain open. Run cargo xtask ci/oracle from rust/; unchanged older suites are required.
