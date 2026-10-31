<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 3. Porting strategy

## The one idea

**Keep the original running and use it as the judge.** A port is not finished when the Rust
compiles, passes its own tests or looks right. It is finished, piece by piece, when the same input
given to the original and to the Rust produces the same answer, and a record of that comparison
exists. Everything else in this page is machinery for that one idea.

Consequences that surprise people:

- The original TypeScript stays in the repository, runnable, until the very end. Only then is it
  allowed to become a frozen reference.
- Rust results are never fed into the original, and original results are never fed into the Rust
  run. They get independent copies of the same input.
- Never "fix" the original while porting. If the port finds what looks like a bug in the
  original, it is written down as a finding with a reproduction, and the port still matches the
  original. Changing behaviour is a separate, reviewed decision.
- A passing comparison needs a real value on both sides. Both sides returning an error, or zero
  cases being run, is not a pass.

## Pin the target

The port targets one commit, `2cd89c6` (app 0.9.2). Upstream keeps moving; changes go in a drift
ledger and are backported by decision, never automatically. Rebaselining means rerunning the
oracles and updating the catalogs, and is its own piece of work.

## Gates, not dates

The plan is a dependency graph with eight gates. A gate is passed by doing the thing it names, not
by writing about it.

| Gate | Passed when |
|------|-------------|
| G0 | The original builds and its tests run here; skips and failures are recorded; the real command and feature catalogs are captured |
| G1 | Pure-Rust candidates for the exact kernel, STEP and mesh work have been tried natively and in WebAssembly on a small extrude, boolean and STEP round trip, and missing operations are written down |
| G2 | One measurement works end to end (plane fit) in native and browser, matching the original |
| G3 | The measurement engine is at parity: all fit families, alignment, deviation, thickness, dimensions, sections, calibrated 2D |
| G4 | Projects (including old versions), undo, recovery and the agent interface are at parity |
| G5 | Reconstruction and mesh tools are at parity |
| G6 | All six workspaces run in a Rust UI, native and web, with no TypeScript app or Node needed (checkpoint R1) |
| G7 | Kernel, STEP and mesh capability corpora pass and a dependency audit proves it is all Rust (endpoint R2) |

G1 and G2 can run in parallel. The riskiest unknowns (kernel, viewer) are probed early so they
cannot hide until the end.

## The first vertical slice

A "vertical slice" is one thin feature through every layer. The first one is deliberately small:

1. Define units, frames, identifiers and an error envelope (started: `rust/crates/port-core`).
2. Read a binary STL into Rust-owned storage with stable vertex ids.
3. Fit a plane to an **explicitly named** set of vertices. Naming the selection keeps the maths
   separate from region growing.
4. Compare with the original on exact planes, noisy seeded planes and degenerate input, natively
   and in WebAssembly behind the old UI.
5. Save, reopen, undo and redo the measurement; run it through the command line and the agent
   interface.
6. Only then add a cylinder, and region growing after that. One new dependency or feature per
   commit, so a mismatch can be located.

## Rules for faithful numerics

- Keep the original's precision: meshes are 32-bit floats, solvers and poses are 64-bit. Do not
  silently widen a stored mesh to 64-bit and call the difference a fix.
- Preserve formulas, operation order, clipping, defaults and rounding. Do not use a nicer rewrite,
  a different solver, or fused multiply-add because it looks cleaner.
- Tolerances are set from the original's own tests and its number representation **before**
  seeing a Rust mismatch. They are never copied from another project.

## Architecture in one paragraph

Dependencies point inward: UI and agent interface → engine → core. The core has no browser, no
files and no global state. A single owner holds the session; commands commit one at a time; jobs
compute against a captured revision and commit only if it is still current. The engine speaks in
commands (like the original), and the UI is a thin skin over them, so everything the app can do
is also scriptable. Start with four crates (`port-core`, `port-engine`, `port-io`, `port-cli`) and
split only when build time or dependencies demand it.

## Shadow mode and switches

While migrating, a completed Rust operation can run beside the original ("shadow mode") with the
original authoritative. Exactly one result is ever applied to the document; two implementations
must never both mutate it. A per-operation switch selects TypeScript or Rust and is removed only
when parity, undo, project behaviour and failure cases pass. Every cutover is a commit with its
evidence and a one-step rollback.

## The work unit life cycle

Every piece of work is a JSON file, `docs/port-plan/plan/units/<id>.json`, and is the only place
its status lives. States, in order:

`planned → specified → baseline-captured → implemented → compared → integrated → accepted`

- A blocked unit keeps the last state it truly reached and lists the blocker in `open_items`.
- "Accepted" needs a tested commit, real evidence, accepted dependencies and no open items in its
  scope.
- Big units (P00 to P39) can have dotted children like `fit.plane.residuals` so one person can
  take a few functions without reserving a whole subsystem.
- The aggregate `backlog.json` is generated. Edit the unit files, not the aggregate.

## Scoping a piece of work (for contributors)

Pick a bounded operation, reproduce its behaviour in the original, claim it in a draft pull
request (unit id, symbols, base commit, files you will touch, how success will be measured), port
the whole body even where a dependency is missing (an explicit "unported" result at the missing
call, not a silent early return), compare natively and in WebAssembly, then update the unit file.
`CONTRIBUTING-RUST.md` has the checklist.
