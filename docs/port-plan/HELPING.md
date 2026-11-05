<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Helping with the Rust port

For a person (not an agent) who wants to take a piece of this work. Everything is proposed as a pull
request; Logan reviews and merges. You cannot break `main` by following this page.

## What this is

A pure-Rust port of ScanRuler, built and checked against the original program. The original is frozen
at tag `reference/2cd89c6` and is the judge: the port is right when it gives the same answers as the
original on the same input, including the same error messages. A passing build alone does not mean the
port is right. Background reading, in order: `docs/guide/01-what-and-why.md`, then
`docs/port-plan/ROADMAP-TO-USABLE.md` sections 2 and 3. Status today is generated in
`docs/port-plan/STATUS.md`.

## Set up

- Rust: the system `rust` package (1.99 or newer) with the `wasm32-unknown-unknown` target. No rustup.
- Node 22 is needed only to run the original for comparisons. If your Node is older, `cargo xtask oracle`
  re-runs itself under `fnm exec --using=22`; install it with `fnm install 22`.
- Python 3 for the plan checks (no packages). `npm install` once at the repo root so the original's
  TypeScript is available to the comparison scripts.
- Work from `rust/`: `cargo xtask ci` is the fast gate (format, lints, tests, plan checks, Wasm
  compile). `cargo xtask oracle` runs the comparisons against the original. `cargo xtask fuzz ply|stl`
  fuzzes a parser against the original. Run the first before every push.

## Pick something

```bash
python3 docs/port-plan/tools/next_task.py --root docs/port-plan
```

prints what is ready, what is claimed, and why. Take the first unclaimed unit, or ask Logan for one that
suits you. A unit is a JSON file in `docs/port-plan/plan/units/` with its scope, what counts as done, and
what depends on what.

Claim a unit by opening a **draft pull request from `main`** whose title starts with the unit id, for
example `io.obj.mesh: port the OBJ face parser`. Check the open PRs first. One unit per PR. If a unit
touches `rust/xtask`, `ci/native-suites.json` or `.github/workflows`, only one open PR may touch it at a
time.

## How a unit gets done

How much proof a unit needs is set by the verification budget in `PORTING-PROCESS.md` (2026-11-05): a
comparison against the original for anything a user sees, plain Rust tests for internal plumbing, and heavy
checks (fuzzing, sweeps, coverage) batched at milestone gates rather than in your PR.

1. Read the original's code for the unit (its `source_paths`) and its own tests.
2. Write the comparison cases first, from the original's behaviour, including bad input.
3. Port the whole body, not just the cases you thought of.
4. For a user-visible result, run the original and your Rust on separate copies of the same input and compare
   success or failure, full error text and output. Internal plumbing needs ordinary Rust tests only.
5. Show that the comparison fails when you deliberately make the Rust value wrong.
6. Record the result in the unit's JSON and regenerate the generated files:
   `python3 docs/port-plan/tools/port_plan.py --root docs/port-plan render` and
   `python3 docs/port-plan/tools/ci_report.py status`. Never hand-edit or hand-merge generated files.
7. Open the PR for review. The PR body is the report: what was compared, what was not, what is open.

## Rules that matter

- Do not edit the original TypeScript. It is the reference.
- Never loosen a tolerance or a random seed to make a comparison pass, and never produce expected values
  from the Rust code. Expected values come from the original.
- Skipped is not passed. Say what was skipped and why.
- Do not raise the 1500 ms limit in `reNetworkCreases.test.ts`. That upstream test is a known flake on
  slow machines; a red result from it alone is understood.
- New dependencies need a decision file first and must be pure Rust.
- Licensing, trademarks and renaming are parked (`LICENSING-AND-NAMING.md`). Leave them alone, and do not
  contact the original's author.
- Keep the SPDX header on every file you add.

## Getting stuck

Write what you tried and what you saw in the PR (or a finding file under `docs/port-plan/plan/findings/`
if it affects others), and stop. A recorded dead end is useful; a loosened check is not.
