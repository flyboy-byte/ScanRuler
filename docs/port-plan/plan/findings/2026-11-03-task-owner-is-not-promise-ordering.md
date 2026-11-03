# Retained futures still need source reaction ordering

2026-11-03. DOCUMENTED: pinned registry limited registers the timer before the
body factory, attaches ran.catch, races [ran,late], then clears the timer in
finally. Its abort dispatch happens before rejecting late. Existing original
oracle command-deadline.mjs line22 resolves/rejects the controlled body inside
an abort listener; factory profiles fire-resolve/resolve-fire and both reaction
orders are retained. The existing Deadline state machine exposes those phases.

DOCUMENTED: #88's native Owner eagerly polls and retains futures independently
of caller tickets, with bounded FIFO wake tokens and explicit drain. Its tests
prove that host contract, not Promise/microtask parity. It has no automatic
losing-body cancellation. A generic Rust future awaiting an already-ready future
need not yield like JS await; waking only says poll again, not which source
promise reaction resolved first.

INFERRED implementation risk to test: a bridge that records timeout at fire_end
then first polls a body resolved by the abort listener afterward can choose the
wrong winner. Polling that body early merely to discover readiness can instead
publish post-await effects before the original microtask turn. Preserve delivered
completion/reaction ordering explicitly and compare the whole source trace;
do not fix it by dropping late bodies, changing the original race array, loosening
comparisons or adding an arbitrary number of wake/yield steps.

Next unit engine.runtime.reactions names that bridge. Full guarded still needs
pre-try activity/current publication, exact normalization/finally override order,
history capture, queue release and late progress (strip suppressed after abort,
caller callback still invoked). Existing helpers remain scoped, not a native
session. No design here changes the source contract or closes M3/P36/P37.
