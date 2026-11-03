# Runtime implementation handoff

Updated 2026-11-03. Read the [new review queue](REVIEW-QUEUE-2026-11-03-SECOND-PASS.md)
for observed heads/base dependencies/checks. It is a finite observation, not a live
merge approval. The owner is merging earlier PRs while this pass runs; refresh
main and PR heads before integration. Codex has not merged any PR.

## Current implementation and evidence

The combined continuation has 51 required native suites / 9,578 case identities,
including refusals. These are declared library comparisons, not successful feature
counts or a completion percentage. No unit is accepted. Verification labels remain
independent of lifecycle; see VERIFICATION.md and generated STATUS.md. Raw coverage
reports with unknown/unreviewed reach cannot grant coverage-reviewed.

Implemented scopes added after the earlier handoff:

| PR | Complete scoped body | Explicit continuation |
| --- | --- | --- |
| #75 | Combined pending branches, case/suite/module union | Owner integration; no new feature body |
| #76 | Evidence labels, negative validation, fresh parser fuzz proof | General paired diff/exact-base ratchet, coverage review |
| #77 | Plugin busy registration and live lookup | Native discovery/installation and full session wiring |
| #78 | Live plugin stopping and concurrent owned waits | Executor resolves done and drains runs |
| #79 | Direct routing and serialized scene turns | Runtime scheduling, deadline placement, viewport |
| #80 | Execute preflight/wait/finally orchestration | Full guarded/history/deadline provider |
| #81 | Retained results over owned acyclic data | Arbitrary host-object/non-JSON conversion |
| #82 | Recent record/update/readout at delivered reactions | Executor attaches and retains reactions |
| #83 | Workspace mappings/current metadata names | Plugin metadata ownership and full state/commands |

Native compilation to Wasm passes, but the new engine scopes have not run in a
Rust engine browser runtime. The existing browser gate covers eight older library
corpora; it does not establish these helpers' browser/session/UI parity. The CLI
still only prints its scaffold banner. That banner's 'nothing is ported' wording
is stale for the libraries, but there is no scan/fit/undo/save CLI workflow yet.

## Next bounded implementation

1. **Complete guarded/interrupt execution over the native owner.** Read the pinned
   registry bodies and the failure-order finding before claiming symbols. Reuse
   existing error normalization, clocks, deadline reactions, scene/import queues,
   settlers, plugin stopping and execute orchestration. Define who retains every
   body/provider after timeout; do not cancel it by dropping a Rust future.
2. **Close the remaining label/history boundary.** Preserve custom-label failure,
   nullish fallback, exact UTF-16 default-title lowercase and the history=false
   branch. Do not quietly substitute ASCII lowercase, lossy conversion or a label
   stub. Pre-register any bounded split and name every missing continuation.
3. **Wire actual family/session state and commands for the measurement slice.**
   Use one owned business-state model, concrete restore/capture participants,
   revisions/epochs and worker/job lifecycle. Route STL scan import and the full
   marked-plane branch through real engine commands. Keep seeded element.fit
   unavailable until its entire continuation is ported.
4. **Implement project compatibility and CLI M3.** Reuse own inflate/ZIP work;
   preserve schema-1 recipes, units/pose/IDs and unknown disabled-plugin payloads.
   Test open scan → fit plane → undo/redo → save/reopen against independent
   original sessions. An executable banner or direct library call does not pass M3.

Required branch tests for execution: preflight before done construction; empty busy
text selecting stop but not guarded busy refusal; publication/current assignment
before guarded's try; source store-listener exceptions; label faults; read-only
settlement beginning before scene waiting; deadline starting in the scene turn;
interrupt routing/history; queue release after timeout; late body effects and
progress (strip suppressed after abort, caller progress callback still invoked).
Compare complete state/effects and exact faults on original-first stable inputs.

The recent store needs an actual once-only reaction owner, not a second task
runtime. Reactions that throw leave source recent records running even though the
caller finished; retain that behavior. Date ISO clips the start, but duration uses
raw sampled start. Readouts copy outer fields and share object/array/error payloads.

## Source still needing implementation/review

stateSections.ts has been read; its register/readAll bodies remain unported. It
uses live maps, function-identity cleanup, catch/error conversion and a plain
object output. Preserve '__proto__' assignment semantics and failure of String or
message access inside its catch; do not assume an ordinary Rust map is identical.
agentLink.ts and the remaining command host/actual state ownership still need review.
The full measurement families, project reader, MCP server, UI and pure-Rust solid/
STEP kernels remain substantial work. No foreign kernel wrapper is a Rust kernel.

## Settled constraints and merge procedure

Use system Rust; no rustup. Own Rust ZIP/inflate/image-codec direction is decided;
do not reopen dependency/rename/licensing questions. Original pin, formulas,
operation order, defaults, cases/seeds/tolerances and every passing comparison
remain fixed. Source outputs never become Rust inputs or expected values generated
from Rust. General diff/diff-ci and the exact-base numerical ratchet remain unbuilt.

Audit parent units before children. #75 additionally incorporates #74, #69 and #64;
merged dependencies must be verified in main, not dropped from the graph. After
parents land, retarget/resync the child and inspect its actual diff; regenerate
backlog/status after preserving canonical units/evidence. Rerun at the resulting
head. Historical green checks do not prove a changed merge base. Keep source CI's
unchanged timing assertion/job budget and every Rust/original/browser gate.

Recent retained incidents: #78's first browser attempt failed at Chrome startup,
then one unchanged retry passed. #83's per-string original-module import exhausted
Node heap; bounded configuration contexts preserved every captured byte and all
306 cases. Older source timing failures remain visible in dated snapshots. None
of these findings authorizes loosening a limit, trimming a corpus or merging.
