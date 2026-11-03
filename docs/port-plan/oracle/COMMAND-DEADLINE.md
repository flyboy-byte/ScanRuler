<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Command deadline race

`engine.session.command.deadline` ports limited() from registry.ts at2cd89c6
through an owned serial timer/body event boundary. The owner registers the timer
before the factory, delivers body settlements in reaction order, dispatches abort
listeners synchronously and performs clearTimeout cleanup before publishing the
caller result. The default remains600000ms.

Already-settled promises retain the original [ran, late] array-order tie. Once
handlers are attached, the first reaction wins. A timer constructs the exact
UTF16 CommandError, aborts with that same shared instance, then rejects late.
An abort listener can settle the body first:69 corpus cases succeed while the
signal is aborted. Timer cleanup runs even when the body throws. Late body
results/errors are consumed without becoming another answer; the owner must keep
that body work alive. No new session epoch, rollback or automatic cancellation
policy is introduced.

291 original cases compare2037 complete observations:104 successful callers,
187 failures. They include immediate/controlled/throwing factories, simultaneous
completion orders, cooperative abort, caller/error identity, late state changes,
timer registration/cleanup, no unhandled late rejections, UTF16 surrogate values
and48 fixed seeded wide-range float durations (1,7,99,1039). The original288-case
capture and passing Rust checkpoint remain byte-for-byte prefixes. Three added
source-first cases cover a timer settled before race registration while its body
is pending. Nothing was removed or relaxed.

The full original module graph executes with an injected serial clock. Only an
oracle export is appended in the temporary source extraction; a regression checks
the complete source-map content against the pinned file plus exactly that export.
Full source/Rust/input frames, bundle/map and Node provenance are archived.
Native capability tests cover foreign/repeated timer tokens, publication only
after cleanup, metadata bounds and late non-Clone error ownership. The live
comparator rejects an altered actual timeout message and truncated state.

Timeout text reuses the existing owned Number formatter, moved unchanged into
port-core. PLY keeps its original private wrapper and tests. All previous numeric
callers and cases remain mandatory. Float division stays ms/1000 at timer fire;
no reciprocal approximation or source-output input is used.

This is an event component, not a physical timer backend or executor. Raw requested
milliseconds are passed through; native/browser clamp policies remain required.
The clock registration/clear contract is trusted and synchronous. The signal is
fresh as at actual registry callers; arbitrary external event mutation, Promise
subclasses, timer resource failures and exhaustive panic/OOM isolation remain
outside this admitted boundary. A conservative name/message quota is checked
before acquiring a timer. Runtime cleanup/termination still needs integration.

Run `fnm exec --using=22 cargo test --manifest-path rust/Cargo.toml -p port-engine --test command_deadline pinned_command_deadline_comparison -- --ignored --exact --nocapture`.
The mandatory source roster becomes34 suites /3551 case identities. Native/Wasm
compilation and library traces do not claim actual browser deadline execution or
complete session/command acceptance.
