<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Roadmap to a Rust build you can use and bug-test

Written 2026-11-02; refreshed 2026-11-04 (section 0). There is no issue queue or outside contributor pool here; the owner will rarely use
the app and bug-test it by hand. So the project must prove itself: the work queue is the plan units, the
bug finder is the comparison against the original, and real use is staged so it starts as early as
possible. This file fixes the layout, workflow and pipeline. It does not change any rule in `HANDOFF.md`.

## 0. Where we are (2026-11-04)

Status: **M2 libraries done, native scan-open/info and select/fit implemented (M3a/M3b), M3 still incomplete.**
Verify current main with `git log -1 origin/main`, `gh pr list -R flyboy-byte/scanruler`
and `STATUS.md`. A feature PR describes the code on its tree; Logan merges it.

- On this tree: STL, PLY, OBJ/MTL, inflate/zip and 3MF libraries; real scan/session
  ownership and runtime (#104), number formatting (#103), prepared rendering (#106),
  native `scan.open` plus `scanruler-rs info` for STL and all PLY encodings (#105),
  and real `scan.query`/marked/box plane fitting with CLI `fit-plane` (#108).
  Unsupported native command formats are explicit refusals; image decoding remains open.
- **Remaining saved code, not yet integrated:** global undo/redo and its CLI
  switches in `codex/m3-plane-finishing` / `codex/m3-cli-selections`. Query,
  selection and plane finishing are public in #108. The full 212-case master
  corpus remains: 205 controls run in M3b; seven global-history controls are
  explicitly deferred to M3c. They are not counted as passing M3b.
- Still missing for M3 everywhere: original-compatible project save/read and the
  new-suite roster integration. No unit or milestone is accepted by this update.
- Run the available CLI from `rust/` with
  `cargo run -p port-cli -- info /path/to/scan.ply [--units mm|cm|m|in]`.
  `cargo run -p port-cli -- fit-plane /path/to/scan.ply --sigma 0` fits a plane.
  Box/marks, normal filtering and orientation/extension flags are in `rust/README.md`.
  The engine owns geometry/state/history recording; the CLI only supplies inputs
  and formats results. Global history travel is not wired until M3c.

Review/merge steps, in order (each its own PR from `main`; merge the previous
before opening its successor where they share files):

1. ~~**M3a, open and info**~~ — actual implementation in PR #105, unit
   `engine.measurement.slice`; all 33 retained import controls, native wrong-state
   rejection, CLI tests, full fast/oracle and 62-suite exact artifact completeness
   pass on source `326b998`. Merged as `fa7fe06`; its code and original comparisons are retained in M3b.
2. ~~**M3b, select and fit**~~ — actual public code in PR #108: `scan.query`,
   marked/box selection, plane `element.fit_marked` with finishing, CLI `fit-plane`.
   All 205 current controls, actual wrong-state rejection and clean rerun pass
   on source `c4ef7e4`; the unchanged 33 import controls and full fast/oracle/
   62-suite artifact gates pass. Merge before opening M3c.
3. **M3c, undo and redo:** global history apply/undo through the runtime recorder;
   CLI `--undo`/`--redo`.
4. **M3d, save and read:** project-compatible save/read for the M3 slice (overlaps
   M6; the original `.scanruler` format is the judge).
5. **Roster:** register new engine suites, prepared rendering and the still-unregistered
   `io.inflate`/`io.threemf` suites in `ci/native-suites.json` and `cargo xtask fuzz`
   (one PR, since it holds `rust/xtask`). The parent slice stays planned/none until
   its full scope and required evidence are integrated. Existing gates stay intact.

`geometry.mesh.crease` is assigned to lane D. The old `ci.integration.nov03` batch
is integrated/closed against merged PR95; future M3/ratchet work belongs to its
current units rather than that historical batch.

M3 is done when Logan can run open, fit, undo and save on his own scan from `main`,
and the M4 ratchet rejects a wrong value (section 3). Open/info alone does not
complete M2 import breadth, M3, or the measurement-slice acceptance gate.

## 1. Development environment (decided 2026-11-02)

| Need | Choice | Why |
| --- | --- | --- |
| Rust | System Arch `rust` 1.99.1, no rustup, `wasm32-unknown-unknown` target | Decision `2026-10-31-no-rustup`. Verified working. |
| Node | **Oracle only.** Node 22 through user-space `fnm` (`fnm install 22`); the system Node 20 is left alone | The original needs Node 22+. `cargo xtask oracle` re-runs itself under `fnm exec --using=22` when the system Node is older, and says why if it cannot. |
| Original app | Frozen tag `reference/2cd89c6`; optional gitignored `.oracle/` worktree when you need to run the app itself | Native oracle tests read the pinned Git object, so most work needs no worktree. |
| Browser | Chrome, for the Wasm runtime lane | Hosted CI already runs it; local run is optional. |
| Python 3 | Plan and status checkers only | No dependencies. |

Local loop for any change: `cd rust && cargo xtask ci` (fast, any machine), then `cargo xtask oracle` for
numerical work. **Verified 2026-11-02:** `cargo xtask oracle` passed all 21 native suites on this machine
(Node 22.23.4 / Rust 1.99.1) before the OBJ units; the hosted `native-oracle` check is green on every merged PR
head and `ci/native-suites.json` lists 62 as of 2026-11-04. That is a local result, not parity: the harness still lacks the exact-base
ratchet (P37) and covers library functions, not application workflows.

## 2. Workflow (no prompt or handoff needed)

State lives in the repo: `plan/units/*.json` (what is done and what is open), `plan/lanes.json` (priority
between lanes), the PRs (who has claimed what), and this file. A session that is given no task runs

```bash
python3 docs/port-plan/tools/next_task.py --root docs/port-plan
```

and takes the first unclaimed unit it prints (ready, then startable, then prototype-only; a prototype-only unit
may be built ahead of its dependencies but cannot be integrated past them). Hot files shared between lanes
(`rust/xtask`, `ci/native-suites.json`, `.github/workflows`) belong to one open PR at a time.

1. **Claim** by opening a draft PR from `origin/main` titled with the unit id; check open PRs first.
2. **Compare before porting:** write the independent cases and tolerances first (from the original's own tests),
   port the whole body, run original and Rust on independent copies, compare values and state, and show the
   comparison fails on a deliberately wrong Rust value.
3. **Record in the unit JSON** (tested SHA, evidence, open scope), then `port_plan.py render` and
   `ci_report.py status`. Findings and decisions are new uniquely named files. The PR body is the report.
   No separate handoff file. Regenerate generated files, never hand-merge them.
4. **Stacking:** one unit per PR from `main`. An unattended multi-PR session may stack its own PRs in order and
   says so in the first PR body. **Land work, don't hoard it (2026-11-04):** a private integration or workbench
   branch is fine for trying combinations, but tested code goes into a public PR within a day, split by unit.
   A doc-only claim PR whose real code lives elsewhere is a sign to open the code PR, not to keep building.
5. **Merging** is the integrator's job and only when Logan asks: read the diff and unit JSON, require the
   hosted Rust, plan, license and (where touched) oracle and Wasm checks, accept the upstream job failing
   *only* on the known `reNetworkCreases.test.ts:209` wall-time test after reading the log, `gh pr merge N -R
   flyboy-byte/scanruler --merge` (never squash), retarget children to `main`, resolve CHANGELOG conflicts by
   keeping both, then confirm `main` CI and that the original tree has no diff against `reference/2cd89c6`.
6. **Never without Logan:** merge or push to `main`, force-push, delete branches or tags, contact Stefan or
   anyone upstream, sign the CLA, deploy, change repo settings, add a dependency (decision file first), edit
   the original TypeScript, loosen a tolerance or seed, or raise the 1500 ms threshold. Licensing, naming and
   the rename are parked (`LICENSING-AND-NAMING.md`).

## 3. The ladder to "I can try it" (milestones, in order)

| # | Milestone | Units | What you can do yourself | What proves it |
| --- | --- | --- | --- | --- |
| M1 | Reproducible baseline and corpus | P00 to P03, `baseline.reproducibility` | Nothing yet | Original runs reproducibly here; every fixture has a licence, generator, seed and hash; skips are listed per reason. |
| M2 | Import breadth | P06 plus `io.ply`, `io.obj`, `io.threemf` | `scanruler-rs info scan.ply` prints bounds, counts, components | Original-versus-Rust on the corpus including malformed files, then on your own scans (section 5). *2026-11-04: reader libraries and native STL/PLY `info` available; other command formats and texture image decoding (`io.obj.companions.images`) open.* |
| M3 | Session and commands, native CLI | P05, P08, P07, P09 | `scanruler-rs` opens a scan, fits a plane, undoes, saves | Command traces equal the original's on independent sessions. **First point you can dogfood.** *2026-11-04: open/info implemented in #105, select/fit in #108; global history and project IO remain open (section 0).* |
| M4 | Paired harness and ratchet | P36, P37 | n/a | A deliberately wrong Rust value is rejected; PRs are compared against the exact base. |
| M5 | MCP | P18 (needs P02, P08, P16) | Point Claude or Codex at the Rust MCP server | Section 4. |
| M6 | Projects and history | P16, P17 | Open an original `.scanruler` project | Original files reopen with equal results; failed load leaves the session intact. |
| M7 | Measurement families, then workspaces | P10 to P15, P20 to P25 | Use it for real measuring | Per-family comparison; R1 preview (P28). |
| M8 | Pure-Rust kernels and purity | P04, P19, P31 to P35, P29 | Everything incl. STEP and solids | Capability corpora and the transitive dependency audit (R2). |

M4 is listed after M3 because the harness can start as soon as P03 lands (see the lanes); it is the item that
makes M3 trustworthy, so build it in parallel and do not call M3 done without it.

~~Import checkpoint 2026-11-02 UTC: STL and PLY only, no engine import command.~~ Superseded by section 0
(2026-11-04): all four reader libraries are present; native STL/PLY scan.open and info are implemented in #105.

## 4. Getting MCP solid

Finding (2026-11-02, verified in `mcp/tests/client.js` and `main.test.js`): the original's MCP tests start the
server as a **child process over stdio** with the standard MCP client SDK, and call tools by name. They are
black-box. So the Rust MCP server is judged the same way:

- **Paired traces.** One stdio client drives the original `scanruler-mcp` and the Rust binary through the same
  scripted tool sequences (tool list and schemas first, then each tool, including refusals and `--tools`
  filtering). Compare tool lists, input schemas, results, errors and image blocks. `mcp/src/commands.json`
  (generated from the app's registry; core 72 / all 140 commands, 6 / 36 headless) is the authoritative list.
- **Reuse the original suite as a contract.** Its 20 test files are the specification of framing, jobs,
  pairing, relay, trace and update behaviour. Port each file's cases as paired scenarios; none of it is
  dropped silently.
- **Headless first.** The Rust engine can serve the headless-capable commands without a browser, so M5 can
  start once M3's commands exist. The browser bridge, relay and pairing come with the web build.
- **Open:** whether the Rust helper keeps the legacy wire format byte-for-byte or ships a new one (P18 says
  legacy compatibility; do not change it without a recorded decision).

## 5. Meshes and scans for testing a finished version

Evidence tags as in `playbat/docs/ledger.md`: `DOCUMENTED`, `REPORTED`, `INFERRED`, `UNKNOWN`.

| Tier | Source | Status |
| --- | --- | --- |
| T1 Generated, licence-clean | Deterministic generators (planes, spheres, cylinders, cones, tori, brackets, fillet rings, shells with known flaws). The original's `scripts/e2e-fixtures.mjs` already generates binary STL this way; reuse its recipes, add ours. Every file gets a manifest entry with generator, seed, hash, units and expected values derived independently. | Plan (P03). `DOCUMENTED` that the generator pattern exists. |
| T2 Adversarial | Truncated, oversized counts, NaN/Inf, duplicate and degenerate faces, inverted winding, huge coordinates, zip bombs for 3MF, header lies. | Plan (P03 and each `io.*` unit). |
| T3 Real scans | The original's strongest tests need `ballbar.stl`, `side bracket left.stl` and `block-marius.stl` at the repo root; they are gitignored and **not on this machine** (`DOCUMENTED`, searched 2026-11-02). They belong to the original author, so we do not have or redistribute them. | **Blocked on you.** Any real scan you can produce (your own printed parts scanned, or CAD you own) can fill T3 locally under a gitignored `fixtures/private/`, reported as *skipped* wherever absent. |
| T4 Public datasets | Candidates exist for meshes and CAD, but each model has its own licence. | `UNKNOWN`: nothing downloaded; verify licences per file before any entry. |
| T5 CAD for STEP | Needed when the kernel lane (M8) starts. | Later. |

Anything the Rust code reads from T3 stays out of the repository and out of CI logs. Skips are never passes.

## 6. Dependency policy for the missing formats (default, revisable)

3MF needs zip and inflate; textured OBJ and the flat 2D workflow need PNG and JPEG decoding. Default: write
the logic ourselves where it is small and security-sensitive; otherwise allow a **pure-Rust** crate that
passes the licence policy and the P35 purity audit, one crate per PR with its decision file. A crate that
wraps C or C++ never counts. No candidate has been audited yet; none is named here on purpose.

## 7. Parked

Licensing, trademarks and the rename (`LICENSING-AND-NAMING.md`) wait for the project to mature and for Stefan's reply to the owner's email. Nothing is distributed meanwhile.

## 8. What is deliberately not here

No release dates, no completion percentage, no promise that a milestone is near. The honest status today
is generated in `STATUS.md`; as of 2026-11-04 native scan.open/info and select/plane flows are implemented in #105/#108;
global history travel, project reader/writer and UI remain open (section 0). Library implementation is distinct from acceptance.

## 9. Open owner decisions (nothing blocks the queue)

- **PLY host-limit differences** (`plan/findings/2026-11-02-ply-stl-independent-fuzz.md`, H1 to H3): Rust refuses some
  hostile inputs where the original fails differently or succeeds. Keep, document per class or match more
  closely. Do not tune limits to hide them.
- ~~**Zip/inflate and PNG/JPEG decoding**: own code or an audited crate.~~ Decided 2026-11-03: own code
  (`plan/decisions/2026-11-03-own-inflate-zip-png-jpeg.md`). Inflate, zip and 3MF are on `main`; PNG and JPEG are
  the open unit `io.obj.companions.images`, judged against Chrome canvas (`plan/findings/2026-11-02-image-codec-pixel-audit.md`).
- **Zip entry size cap** (`plan/findings/2026-11-03-threemf-first-run.md`): Rust refuses some mutated archive entries
  over its host cap that the original reads. Same choice as the PLY host limits: keep, document per class or match.
- **Parent units** `io.ply` and `io.obj` stay `planned` until their P03 corpus entries exist; their children are implemented.
- **Independent fuzz is the standard second check for an import unit**: seeded mutation of header, truncation and body
  against the original, in a scratch directory, before the unit leaves `implemented`.

## 10. Verifying a large amount of ported code (decided 2026-11-02)

The original is the judge; the risk is input it was never asked about. Strategy, in the order to build it
(units in lane C):

1. **Standing fuzz** (`harness.fuzz.standing`): seeded mutation of every parser against the original, a repeatable
   command plus a scheduled job. The scratch PLY/STL run found a bug that about 1,300 fixed cases missed.
2. **Coverage of what is compared** (`harness.coverage.original`, `harness.coverage.rust`): which original branches
   and which Rust lines the comparison cases never reach. Unreached original behaviour is unverified and becomes the
   next corpus target.
3. **Prove the checks can fail**: every unit already needs a deliberately wrong Rust value rejected; add occasional
   mutation testing as an audit (tooling only, needs the owner's OK to install).
4. **Verification levels** (`harness.verification.levels`): none, fixed-corpus, fuzzed, coverage-reviewed;
   `accepted` needs the last. Implemented never means verified.
5. **Composition checks**: session command traces, paired MCP traces and opening original `.scanruler` projects
   (sections 3 and 4) catch what per-function parity cannot.
6. **Rotate checkers**: whoever did not write a unit fuzzes and audits it. Review for hostile input: allocation sized
   from a header, `as` casts, indexing, unbounded loops. Lints already forbid `unsafe`, `unwrap` and `panic`.
7. **Use it** from M3 onward.

