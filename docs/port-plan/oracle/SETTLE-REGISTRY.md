# Settlement callback registration and live iteration

2026-11-02. PR #67 supplies registerSettle and live Set iteration over #66's
unchanged wait-polling component. Functions deduplicate by shared identity;
re-registering an active function does not reorder it. Unsubscribe is explicit,
not Drop-based. Old capabilities remove a later registration of their function.
Deleted unvisited callbacks are skipped; new and deleted/re-added callbacks run
later in this same iteration. List borrows end before callbacks, permitting safe
registration mutation. Fresh metadata preserves UTF-16 name and readOnly values.

116 source-first two-run scenarios compare full callback argument/order logs,
all null/pending/done states, external completion order and unhandled-error counts.
They cover self/other deletion, add, duplicate add, delete/re-add, old unsubscribe
and re-registration, mixed ready/pending successes/faults, and seeds1/7/99/1039.
All source/native bytes match. Full original graph/maps/private-export source
proof and complete paired raw artifacts are retained. Both authored/seeded corpus
generators reproduce their immutable inputs without reading any engine answers.
The previous64 settlement body cases remain required. The only shared change is
a crate-private constructor for its existing wait vector; no poll formula changes.

Registration slots (including tombstones), command text and reentrant start depth
are bounded. Wait capacity is reserved before calls; at most one wait per finite
registration slot can be collected. Registration host failures are exposed by
register, not converted to source command codes. Ordinary tests cover borrowed
work, non-Clone errors, explicit subscription lifetime and reentrant depth refusal.
Weak handles avoid callback/session ownership cycles. Removed slots retain bounded
callback metadata until the registry owner drops; no unbounded history is built.

The executor must keep and drain all started work after caller abandonment.
Actual browser promise turns, complete registry wiring and runtime termination
remain open. Ordinary Rc allocation is not process-wide OOM isolation. Owned
metadata excludes mutable JS getters/prototype replacement. Native roster on
this stack:36 suites /3,731 identities. Run cargo xtask ci and cargo xtask oracle
from rust/. Wasm compile is required, not browser or whole-session acceptance.
