<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Next pass: Codex tonight, Claude from Monday

Written 2026-11-02. Superseded by the next version of this file; keep old ones in git history, not here.
State at writing: `main` holds PRs #1 to #24 plus the operations docs; 70 units, none accepted.

## Tonight: one unattended Codex pass

Paste the common block and then the pass block below from `prompts/README.md`. Order of work (do in this
order; stop a lane when it is blocked and say why; do not start a new lane to avoid a hard one):

1. **Lane H, `io.ply`** then **`io.obj`** (geometry and MTL parsing first; image decoding deferred) then
   **`io.threemf`**. 3MF needs inflate: write `plan/decisions/<date>-3mf-inflate.md` first; prefer a small own
   implementation, otherwise propose a crate there and **do not add it** until the owner approves.
2. **Lane C, P36**: the generic paired runner and its negative tests, in `rust/xtask` and `ci/`. If lane H PRs
   also need `native-suites.json`, land those first and rebase; one lane owns `xtask` at a time.
3. **Lane A, evidence only**: P01 per-skip ledger and a reproducibility record from the hosted run, P03 fixture
   manifest design with the first generated files. Docs/evidence PRs; no change to the original tree.

Not tonight: lane B (the session spec wants a human-in-the-loop read), D and E (blocked on B), F, G
(prototypes, Claude/Monday), anything about licensing, naming or the rename (parked).

### Pass block (add to the common block)

```
This is an unattended pass. You may stack PRs in the order above because you cannot wait for merges: base each PR
on the previous one, and list the order in the first PR body. Keep every PR to one unit, tests green on its own
head (`cd rust && cargo xtask ci`, plus `cargo xtask oracle` where numerical). Hosted CI is authoritative for the
original's unit tests: the upstream job may fail ONLY on plugins/reverse/tests/reNetworkCreases.test.ts:209
(1500 ms wall time); report it, never change it. Do not merge, retarget or close anything. At the end write
docs/port-plan/SESSION-<date>-<agent>.md listing each PR, the exact tested SHA, what is proven, what is not,
failed attempts, and the next step. Stop when credit or time runs out; leave no half-edited generated file.
```

## Integration (whoever merges next, probably Monday with Claude)

Follow `AGENT-OPERATIONS.md` section 5. The stack from tonight merges bottom-up with `--merge`, retargeting each
child to `main`; the only expected conflicts are `CHANGELOG.md` (keep both) and generated files (regenerate).
Check each PR's unit JSON against its diff, confirm tolerances/seeds/cases were not loosened (`git diff` on
the comparison files), and read the failing-check logs before merging past the timing test.

## Monday with Claude (suggested order)

1. Integrate tonight's stack (above) and confirm `main` CI.
2. **Lane B**: write the `engine.session.contract` spec as a doc PR; this unblocks D, E and the MCP lane.
3. Review lane C's negative tests personally: the harness is only worth anything if it rejects wrong values.
4. Then lane D, or a kernel (G) / viewer (F) spike, whichever the Codex results make cheaper to start.

## Needed from the owner

- Any real scan or CAD file you own, into `fixtures/private/` (gitignored). Optional; nothing blocks on it.
- Approve or reject any dependency the pass proposes in a decision file.
- Stefan's reply (parked): nothing in this pass depends on it.
