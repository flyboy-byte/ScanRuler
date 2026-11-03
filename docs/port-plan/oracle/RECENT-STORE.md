# Recent outcome recording and delivered reactions

2026-11-03. PR #82 ports the record/update/readout bodies of remember and
recentOutcomes at the caller-owned run/delivered-reaction boundary. The original
run is returned unchanged; exactly 100 records are kept, oldest evicted first.
Completion still updates its own evicted record without reinserting it. Readout
filters exact job strings (including empty), reverses without mutating the list,
and copies outer fields. Primitive results are copied values; object/array/error
contents remain shared by shallow snapshots. A captured running snapshot stays
running when its underlying record later completes.

Start samples raw Date.now, then clips only the ISO timestamp. Duration uses the
raw untruncated start. Reactions evaluate clock, then retained data or error
normalization/toJSON before assigning any fields. A failing observer leaves the
record running while the caller's own success/failure stays unchanged. Typed errors,
UTC conversion and owned retained data reuse existing Rust components. Native result
containers preserve fixed root kind, and error JSON is a fresh shared plain record.

362 independent source-first traces compare full retained records/filter results,
frozen/edited shallow snapshots, exact run identity, caller states, clock/normalizer
effects, observer failures, payload aliasing and the actual first record even after
eviction. They cover fractional/limit/invalid clocks, negative/NaN durations,
throwing observers, primitive/object/array/undefined/byte/omitted results and
105-start eviction with late out-of-order completion. All nine initial observables
remain byte-identical after adding actual evicted-record state as a tenth field.
Full source/map/provenance, generators and complete paired artifacts are retained.

The native executor owns and observes the actual future, attaching/retaining the
once-only Ticket reaction. This store does not poll it or invent a second task
runtime; automatic attachment and microtask scheduling remain integration work.
The input run is retained even on admission failure. Dropping an undelivered ticket
quarantines new remembering without changing recorded business outcomes; delivered
provider faults are returned separately and must not replace caller outcomes.
Pending/text/data budgets are native limits, not source retention/omission policy.
Tests prove non-Clone borrowed run preservation, observer ordering/no assignment,
repeat delivery, abandonment and distinct data-host refusal.

The supported result domain remains owned acyclic data from #81. General JS getters,
prototypes/toJSON/function/BigInt/cyclic conversion and mutation remain outside it.
Current tests do not establish actual browser/session runtime or M3. Run cargo xtask
ci/oracle from rust/; all previous native/formatter/date/error corpora remain required.
