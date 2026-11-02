<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Starting prompts for Claude Code and Codex

Copy the common block, then one lane block. Lanes and their dependencies are in `../AGENT-OPERATIONS.md`; run at
most one agent per lane and check `gh pr list -R flyboy-byte/scanruler` for claims first. Updated 2026-11-02.

## Common block (always first)

```
You are working on the pure-Rust port in ~/projects/scanruler (trunk: main). Read, in order: AGENTS.md or
CLAUDE.md, docs/port-plan/HANDOFF.md, ROADMAP-TO-USABLE.md, AGENT-OPERATIONS.md, then your unit JSON(s) under
docs/port-plan/plan/units/ and their source_paths in the original TypeScript.
Rules: branch from current origin/main as <agent>/<unit>-<slug>; open a DRAFT PR first naming the unit, the
original symbols and the files you will touch; one unit per PR; no stacked PRs. The original is the judge:
write the independent comparison cases and tolerances BEFORE the port, port the whole body, compare values and
state, and show the comparison fails on a deliberately wrong Rust value. Never feed original output into Rust,
never loosen a tolerance or trim seeds, never regenerate expected values from Rust. Skips are not passes; a
green `cargo xtask ci` is not parity. Run `cd rust && cargo xtask ci`, and `cargo xtask oracle` for numerical
work (it selects Node 22 itself). Update only your unit JSON (tested SHA, evidence, open scope), then regenerate:
python3 docs/port-plan/tools/port_plan.py --root docs/port-plan render && python3 docs/port-plan/tools/ci_report.py status.
Do not edit CHANGELOG.md or the README status table. Do NOT merge, push to main, add a dependency without a
decision file, edit the original TypeScript, or contact anyone outside the repo. Do not work on renaming,
trademarks or licensing; that is parked. Finish by listing what is proven, what is not, and the open items.
```

## Lane blocks

**A. Baseline (P00 to P03)**
```
Lane A, baseline reproducibility. Make the original runnable and recorded on this machine and in CI: pin
toolchains and lockfiles, a per-skip ledger with the reason for each skipped test (private scans stay skipped),
complete the command/feature catalogs with executed coverage, and build the generated fixture corpus with a
manifest (generator, seed, hash, units, expected values derived independently of both programs). The
reNetworkCreases wall-time test is a recorded flake: never raise its threshold. Start with P00/P01 evidence.
```

**B. Session contract (P05, `engine.session.contract`)**
```
Lane B. Write the spec first, as a doc PR, before any implementation: owned buffers and limits, units, frames,
stable IDs, session epoch/revision, structured errors, read vs mutation lanes, history participants, late-job
rejection. Derive it from src/commands/{registry,state,types}.ts, src/core/{workerProtocol,workerRpc}.ts and
src/state/historyStore.ts, with source-trace fixtures of successful and refused mutations. Implementation of the
contract in port-engine is a second PR only after the spec PR is merged.
```

**C. Paired harness (P36, then P37)**
```
Lane C. Build the generic original-versus-Rust paired runner (stable case IDs, independent input copies,
structured outputs, distinct outcome states) then the exact-base ratchet that compares a PR against validated
reports for its exact base commit. Negative tests are the point: the runner must reject an altered Rust value, a
missing key, an empty suite, a skipped reference, a stale input hash and a changed case id. Extend the Chrome-Wasm
corpus only with independent inputs. You own rust/xtask/src/main.rs and ci/ while this lane runs.
```

**D. Measurement slice (P06, P07, P08, then P09)**
```
Lane D. Start only after lane B's spec is merged. Route typed scan import and the plane branch of the marked-
selection fit through a minimal command registry and the native CLI, reusing port-io STL, build_mesh_graph and the
merged plane code. One history step per command, source import reset/invalidation, command refusals and progress,
stale-completion rejection, undo/redo. Compare command traces with the original on independent sessions.
```

**E. Projects (P16, P17)**
```
Lane E. Start after lane B and P08. Read original schema-1 .scanruler archives (bounded decompression: 16 members,
64 MiB manifest, 1 GiB expanded), keep unknown plugin JSON and members, recompute results. Original files must
reopen with equal units, pose, IDs and recipes; a failed load leaves the current session intact. Do not invent a
new format.
```

**F. Viewer spike (P24)**
```
Lane F, prototype only. Prove native and Wasm rendering, picking and overlays on a large mesh graph and report
frame time and memory. The deliverable is evidence and a recommendation, not a UI. No new dependency without a
decision file and the purity check.
```

**G. Kernel spikes (P04, then P31 to P35)**
```
Lane G, prototype. Build a per-operation capability matrix for pure-Rust exact geometry, boolean/extrude,
STEP read/write and tessellation, and mesh algorithms, on native and Wasm: supported / partial / missing /
incorrect / timeout / unknown. ~/solidcore is a candidate, read for patterns, not copied. A Rust wrapper around
C or C++ does not count. Missing operations are written down as required work.
```

**H. Import breadth (`io.ply`, `io.obj`, `io.threemf`)**
```
Lane H, one format per PR. Port the original parser's accepted variants and its refusals, limits and progress,
producing the same owned mesh buffers as STL. Build generated and adversarial files (truncated, oversized counts,
NaN/Inf, hostile headers; zip bombs for 3MF) and compare original vs Rust on geometry, errors and limits. Codecs
(inflate, PNG/JPEG) follow plan/decisions/2026-11-02-pure-rust-dependency-default.md: write a decision file before
adding any crate.
```
