<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Roadmap to a Rust build you can use and bug-test

Written 2026-11-02. There is no issue queue or outside contributor pool here; the owner will rarely use
the app and bug-test it by hand. So the project must prove itself: the work queue is the plan units, the
bug finder is the comparison against the original, and real use is staged so it starts as early as
possible. This file fixes the layout, workflow and pipeline. It does not change any rule in `HANDOFF.md`.

## 1. Development environment (decided 2026-11-02)

| Need | Choice | Why |
| --- | --- | --- |
| Rust | System Arch `rust` 1.99.1, no rustup, `wasm32-unknown-unknown` target | Decision `2026-10-31-no-rustup`. Verified working. |
| Node | **Oracle only.** Node 22 through user-space `fnm` (`fnm install 22`); the system Node 20 is left alone | The original needs Node 22+. `cargo xtask oracle` re-runs itself under `fnm exec --using=22` when the system Node is older, and says why if it cannot. |
| Original app | Frozen tag `reference/2cd89c6`; optional gitignored `.oracle/` worktree when you need to run the app itself | Native oracle tests read the pinned Git object, so most work needs no worktree. |
| Browser | Chrome, for the Wasm runtime lane | Hosted CI already runs it; local run is optional. |
| Python 3 | Plan and status checkers only | No dependencies. |

Local loop for any change: `cd rust && cargo xtask ci` (fast, any machine), then `cargo xtask oracle` for
numerical work. **Verified 2026-11-02 on this machine:** `cargo xtask oracle` passes all 18 native suites on
Node 22.23.4 / Rust 1.99.1. That is a local result, not parity: the harness still lacks the exact-base
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
   says so in the first PR body.
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
| M2 | Import breadth | P06 plus `io.ply`, `io.obj`, `io.threemf` | `scanruler-rs info scan.ply` prints bounds, counts, components | Original-versus-Rust on the corpus including malformed files, then on your own scans (section 5). |
| M3 | Session and commands, native CLI | P05, P08, P07, P09 | `scanruler-rs` opens a scan, fits a plane, undoes, saves | Command traces equal the original's on independent sessions. **First point you can dogfood.** |
| M4 | Paired harness and ratchet | P36, P37 | n/a | A deliberately wrong Rust value is rejected; PRs are compared against the exact base. |
| M5 | MCP | P18 (needs P02, P08, P16) | Point Claude or Codex at the Rust MCP server | Section 4. |
| M6 | Projects and history | P16, P17 | Open an original `.scanruler` project | Original files reopen with equal results; failed load leaves the session intact. |
| M7 | Measurement families, then workspaces | P10 to P15, P20 to P25 | Use it for real measuring | Per-family comparison; R1 preview (P28). |
| M8 | Pure-Rust kernels and purity | P04, P19, P31 to P35, P29 | Everything incl. STEP and solids | Capability corpora and the transitive dependency audit (R2). |

M4 is listed after M3 because the harness can start as soon as P03 lands (see the lanes); it is the item that
makes M3 trustworthy, so build it in parallel and do not call M3 done without it.

Import checkpoint 2026-11-02 UTC: the STL and all three PLY encoding bodies now
have bounded native source comparisons. PLY does not yet have real browser-Wasm
coverage or a public engine command, and `scanruler-rs info` is still unimplemented.
These library bodies do not complete M2 or replace the session work needed by M3.

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
is generated in `STATUS.md`; there is still no working Rust engine command flow,
project reader or UI. Library implementation is distinct from acceptance.
