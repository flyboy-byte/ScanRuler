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

## 2. Workflow

Work comes from `plan/units/*.json`, taken in the lane order in `AGENT-OPERATIONS.md` (claim with a draft
PR, branch from `main`, one unit per PR, merge commits, the integrator merges). The loop for an agent:

1. Read the unit and its source paths in the original.
2. Write the comparison first: independent input cases, with the observable outputs, **before** the port
   (preregister tolerances from the original's own tests).
3. Port the whole body, run original and Rust on independent copies, compare values and state.
4. Prove the comparison can fail (alter a Rust value, see it rejected).
5. Update the unit JSON with the tested SHA, evidence and open scope; regenerate the aggregates; open the PR.

Nothing is accepted by a count of PRs or cases.

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

## 7. What is deliberately not here

No release dates, no completion percentage, no promise that a milestone is near. The honest status today
is in `STATUS.md`: 21 units implemented, none accepted, no Rust engine, project reader or UI yet.
