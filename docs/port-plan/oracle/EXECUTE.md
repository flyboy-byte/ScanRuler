# Execute preflight, provider order and done lifetime

2026-11-03. PR #80 ports the complete execute orchestration body from the pinned
registry, at owned stop/settle/guarded provider boundaries. It preserves strict
`discards(input) === true`, conditional guard evaluation and raw busy presence.
Empty busy text still selects stopping. Preflight runs before done is created
and before try/finally: a preflight fault creates no completion capability. Stop
wins even for a deferred command; otherwise non-deferred commands settle, while
deferred commands start guarded immediately. After waiting, guarded starts once.
Every failure after done construction resolves it; dropping native work does not.

The full original registry/activity graph produces 1,200 source-first traces over
absent/false/true/truthy-other/throwing discards, null/empty/busy/throwing guards,
both deferred flags, ready/late/success/rejected/thrown/void settlement and body
outcomes. The original uses its real stopPluginWork, settlers and guarded branch
with history=false. The native driver wires the actual Plugins::stop_work and
Settlers::start providers from #78/#67. A separately owned guarded adapter supplies
its typed outcomes; this unit does not port guarded activity/history/deadline code.
Only orchestration-owned observables are claimed: complete call order, exact caller
outcome fields/identity, pending state, done notifications and unhandled-rejection
count at stable drained boundaries. Original source/map/provenance, independent
inputs/generator and complete raw paired artifacts are retained. No source output
is a Rust input; the source-map proof permits only the private execute export.

Native done is a shared completion capability with independently admitted waiters.
Each stores its own waker, and finally wakes every surviving observer once outside
internal borrows. Waiter slots, including removed waiters, have an explicit host
bound. Limits/allocation errors are separate from source outcomes. Tests exercise
multiple wakers, no resolution from Drop, preflight without done, repeat-poll misuse
and borrowed non-Clone values. Done clones compare by capability identity. Standard
Rc/Box metadata allocation is not process-wide OOM isolation.

Stop and guarded are async providers: their returned work is retained rather than
converted into a synchronous throw. Settlement may throw synchronously. The native
executor must drain every started run/provider; Rust dropping a future would cancel
work unlike original promises. Exact microtask/timer integration, full guarded body,
history/default-label conversion, worker/command ownership, browser execution and
whole-session acceptance remain open. Run cargo xtask ci/oracle from rust/; all
older corpora remain required. No M3 or accepted-unit claim.
