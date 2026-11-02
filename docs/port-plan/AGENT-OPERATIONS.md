<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Running several agents on the port (Claude, Codex, humans)

Updated 2026-11-02. State at writing: PRs #1 to #24 are merged into `main` (`0940fd2`), no PR is open,
every unit is implemented-or-planned and **none is accepted**. This file is the operating manual for
the next wave. It does not change any rule in `HANDOFF.md`, `PORTING-PROCESS.md` or `AGENTS.md`; where
they disagree, those win and this file is fixed.

## 1. Every agent starts here

1. Read `AGENTS.md` (or `CLAUDE.md`), then `HANDOFF.md`, `ROADMAP-TO-USABLE.md` (environment, milestones), then this file, then your unit's JSON.
2. `git fetch origin && git checkout -b <agent>/<unit>-<slug> origin/main`. Always branch from current
   `main`, never from another agent's branch. No stacks: if you need another agent's unfinished work,
   say so in the PR and wait for it to merge.
3. `gh pr list -R flyboy-byte/scanruler` and search open PRs for your symbols and paths. Pass `-R
   flyboy-byte/scanruler` on every `gh` call; the fork default can resolve to upstream.
4. **Claim by opening a draft PR first**: unit id, original symbols, files you will touch, how success is
   measured. Two agents never hold the same unit or the same file in `rust/crates/*/src`.
5. Do the work under the rules in `AGENTS.md` (faithful port, independent inputs, no loosened
   tolerances, no regenerated goldens, skips are not passes).
6. Local gate before asking for review: `cd rust && cargo xtask ci`. Add `cargo xtask oracle` when you
   touched a numerical unit and have Node 22 (hosted CI runs it regardless). A green gate is not parity.
7. Mark the PR ready only when the unit JSON carries the real tested SHA, evidence and open scope.

## 2. Lanes and their real dependencies

Dependencies below are copied from `plan/units/*.json` (`depends_on`) on 2026-11-02. Per `IMPLEMENTATION-PLAN.md`
they are **contract dependencies**: you may prototype ahead of an unaccepted dependency, but your unit cannot
be integrated or accepted past it. Say in the draft PR which dependency you are building ahead of.

| Lane | Units | Depends on | Touches | Notes |
| --- | --- | --- | --- | --- |
| A. Baseline (**critical path**) | P00, P01, P02, P03 (chain) | P00 has none | `evidence/`, fixtures, CI | Reproducible original run (locks, toolchains, per-skip ledger), finish catalogs (core and all-plugin exist; executed coverage and variants do not), fixture corpus with licences. Timing failures stay failures. Nearly everything below waits on this. |
| B. Session contract | P05 | P02, P03 | spec doc first, then `port-engine` | Agree the schema in a doc PR before anyone implements against it. Gatekeeper for D, E and H. |
| C. Paired harness | P36, then P37 | P36: P00 to P03; P37: P36 | `rust/xtask`, `ci/`, `port-wasm-probe` | The generic original-versus-Rust runner, then the exact-base ratchet. Prove it rejects a deliberately wrong Rust value. Browser-Wasm coverage extends the existing 114-input corpus with independent inputs only. |
| D. Measurement slice | P06, P07, P08 | P06: P05; P07: P06, P36; P08: P05 | `port-engine`, `port-cli`, `port-io` | Typed scan import plus plane fit through a command registry, with undo. Reuse the merged STL, mesh-graph and plane code; do not add a second pipeline. |
| E. Project compatibility | P16 | P03, P08 | `port-io` | Original schema-1 archives, bounded decompression, unknown plugin data preserved. |
| F. Viewer spike | P24 | P09 (integrates P07 and P08) | new crate or example | Large-graph render, picking, overlays, native and Wasm. Evidence, not a UI. Late in the chain; prototype only. |
| G. Kernel spikes | P04, then P31 to P35 | P04: P03, P36; P31: P04, P05; P34 also P06; P35 last | new crate; `~/solidcore` is a candidate | Per-operation matrix: supported, partial, missing, incorrect, timeout, unknown. No C or C++ wrappers. Prototyping now is allowed and encouraged; acceptance waits. |

Suggested order: **A first** (it unblocks the rest and nobody else can finish without it), with **B as a spec and
G as a prototype spike** in parallel, because they edit different files. Then C, then D
and E. Hot files that more than one lane edits (`rust/xtask/src/main.rs`, `ci/native-suites.json`,
`.github/workflows/*`) belong to one lane at a time; say in the draft PR if you need one.

## 3. What an agent never does without Logan saying so

Merge, push to `main`, force-push, delete a branch or tag, contact Stefan or anyone upstream, sign the
CLA, open a PR to `CNCKitchen/ScanRuler`, deploy, change repository settings, install system packages,
add a dependency (it needs the licence and purity review first), edit the original TypeScript, change a
tolerance or seed, or raise the 1500 ms threshold in `reNetworkCreases.test.ts`.

## 4. Generated and shared files (where merges actually collide)

| File | Rule |
| --- | --- |
| `docs/port-plan/plan/units/*.json` | Edit only your unit. Canonical. |
| `backlog.json`, `STATUS.md`, `status.json` | Generated. After editing units run `port_plan.py --root docs/port-plan render` and `python3 docs/port-plan/tools/ci_report.py status`. Never hand-merge; regenerate. |
| `plan/findings/`, `plan/decisions/` | New uniquely named files only; never edit an old one. |
| `CHANGELOG.md` | Every PR so far added an `Unreleased` line at the same spot, which conflicted twice. **Default: agents do not edit it; the integrator adds one line per merged PR.** Logan can overrule. |
| `README.md` status table | Integrator only, at merge time. |
| `AUDIT-QUEUE.md` | Historical for #1 to #24. New PRs are tracked by `gh pr list`, not by editing this file. |

## 5. Merge protocol (what the integrator does)

Only one session integrates at a time, and only when Logan asks.

1. For each ready PR, read the diff and the unit JSON; confirm the claimed SHA is in the PR.
2. Required hosted checks: Rust fast gate, Plan integrity, License check, native oracle and the Chrome-Wasm
   lane where touched. The upstream `test-and-build` job may fail on the wall-time test alone; read the
   log, confirm it is only that test, and say so in the merge note. Any other red check blocks.
3. `gh pr merge N -R flyboy-byte/scanruler --merge` (a merge commit, never squash: per-unit history and
   evidence SHAs must survive). Merge in dependency order, then re-check `main` CI.
4. On conflict, merge `origin/main` into the PR branch, resolve, regenerate (never hand-edit generated
   files), rerun the plan checker, push (no force), then merge.
5. After merging, confirm `git diff reference/2cd89c6 origin/main -- src tests plugins mcp scripts
   package.json package-lock.json` is empty and `main` CI is green.

The 2026-11-02 run of this protocol merged #1 to #24 in order with two CHANGELOG-only conflicts.

## 6. Which agent for which job

This split is a suggestion, not a measurement. Adjust it with experience and record what you learn.

- **Claude Code**: planning and specs (lane B), review and integration (section 5), anything that needs
  reading much of the original to decide what the port must preserve, docs and evidence reconciliation.
- **Codex**: bounded numerical or protocol ports against a written contract, harness building (lane C), where a failing differential test gives it a clear signal.
- **Either**, as an auditor of the other: findings from another model are input to verify, kept as separate
  files under `plan/findings/`, not merged into the plan on faith.

Whoever finishes writes what they learned (failed attempts included) into the unit JSON `open_items` or a
finding, so the next agent does not rediscover it.

## 7. Limits of this document

It organises work; it proves nothing. No lane here is done until the unit JSON says so with a tested
commit and real evidence. Case counts and merged PRs are not parity.
