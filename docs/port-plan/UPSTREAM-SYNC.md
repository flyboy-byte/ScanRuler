# Branches, the frozen oracle and upstream updates

Decided 2026-10-31. The fork's purpose is the full port, so `main` is the port trunk. Upstream
(`CNCKitchen/ScanRuler`, remote `upstream`) is tracked, never contributed to from here.

| Ref | Role |
| --- | --- |
| `main` | Port trunk. Rust lands here by PR inside the fork. Root TypeScript stays unmodified. |
| `upstream-main` | Mirror of `upstream/main`. Fast-forward only. Holds no port work. |
| tag `reference/2cd89c6` | The frozen oracle (app 0.9.2). Immutable. |
| `port/*` | Short-lived work branches. |

## Running the oracle

Run the original from a worktree of the tag in `.oracle/` (inside the repo, so no extra folders), not from `main`, so merging upstream can never change
what the Rust is compared against:

```bash
git worktree add .oracle reference/2cd89c6   # once, inside the repo (gitignored); needs Node 22 there for npm ci
```

Record the oracle's commit in every result (`reference-pin.json` names it).

## Bringing in upstream changes (a rebaseline, never automatic)

1. `git fetch upstream && git checkout upstream-main && git merge --ff-only upstream/main && git push origin upstream-main`
2. Triage `git log reference/<old>..upstream-main` into a drift ledger (`plan/findings/`): commit,
   affected feature, security or data-loss relevance, backport or defer. Security and data-loss
   fixes first. The first entry is
   [2026-11-03-upstream-drift-0.9.2-to-0.9.4](plan/findings/2026-11-03-upstream-drift-0.9.2-to-0.9.4.md).
3. Only for a chosen rebaseline: branch from `main`, `git merge upstream-main`, resolve (expect
   conflicts in `README.md`, `CHANGELOG.md`, `.gitignore` and `ci.yml`; see "When the pin is far
   behind" below), update `reference-pin.json` and `port-core`'s
   `REFERENCE_SHA`, tag the new `reference/<sha>`, rerun the oracle baseline and catalogs, and
   re-compare every accepted unit. Affected units fall back to their last verified state until
   they pass again.
4. Until then `main` need not contain upstream's newer commits, and the port still targets the
   pinned one.

Never rebase `main` onto upstream, and never commit port work to `upstream-main`.

## When the pin is far behind

Read this before a rebaseline once the pin is more than a few releases old. The gap on
2026-11-03 was two releases and a credits commit, so several points below are expectations
and not experience.

**History is release snapshots.** Upstream is published from a private development
repository as one commit of the whole tree per release (its CONTRIBUTING.md, "How you are
credited"). `git log` lists releases, not fixes: a fix cannot be cherry-picked or bisected,
only extracted by hand from a snapshot diff. Triage from the CHANGELOG entries, which name
the area, date and often the issue, and confirm against the diff. Triage every release in
the gap, even though a merge needs only the newest tree. The pin stays reachable because
this fork holds the `reference/2cd89c6` tag and its objects: do not delete the tag, and do
not rely on upstream still having that commit.

**Parity has a version.** While the pin is old, "matches the original" means "matches
0.9.2". Say the version in every claim, status page and release note. People compare with
the current app, not the pin, so keep the ledger's list of behaviours where the current app
differs from the pin up to date, and never claim parity with "ScanRuler" unqualified.

**Known upstream fixes are known bugs in the oracle.** A port faithful to the pin
reproduces bugs upstream has since fixed (in the first ledger: a 3MF memory bomb, a worker
queue that hangs, notes from a project file read as instructions, a PLY cursor rewind). For
each, decide and record in the unit: reproduce it (files or users rely on the behaviour),
fix it as a named deliberate divergence with a minimized fixture and a test that states what
the original does, or refuse the input. Security, denial-of-service and data-loss fixes are
not left for a rebaseline: they go into the Rust unit when it is built, and the comparison
excludes exactly those inputs, by name. Do not loosen tolerances or drop cases to hide a
divergence.

**Compatibility surfaces move first.** Some things leave the repository: project files
(`schemaVersion`; `PROJECT_SCHEMA` in `src/app/project.ts`), files the app writes (STEP,
STL, 3MF, PLY), and what agents use (command names and schemas in
`mcp/src/commands.json`, the `scan-measuring` skill, the MCP tools and the shape of their
results, the plugin API). Check these for every upstream release even when nothing else is
rebaselined. A newer project schema changes the M6 target: the Rust reader must open both
versions or refuse the newer one clearly.

**Pinned dependencies are part of the oracle.** The DEFLATE/ZIP port reproduces `fflate`
0.8.3, and `meshstep` and `occt-wasm` are pinned exactly upstream (a bump has a fuzz-golden
and benchmark checklist in its `docs/development.md`). A bump changes behaviour with no change
under `src/core`, so read the lockfile diff first. None changed from 0.9.2 to 0.9.4.

**A rebaseline invalidates evidence, so its cost follows accepted units, not the size of
the gap.** It changes `REFERENCE_SHA`, `reference-pin.json`, both catalogs and every
fixed-corpus and fuzz result recorded against the old pin. The cost is roughly the number of
accepted units whose area drifted. Rebaseline at a milestone boundary or before many units
in drifted areas are accepted, not continuously and not after. Until then, do not mark a
unit accepted in a drifted area without naming the version it was accepted against.

**Do not merge upstream into `main` until the oracle is decoupled from the repo root.** The
section above says the oracle runs from `.oracle`. On 2026-11-03 only `original-catalog.yml`
checked the pin out into `.oracle`; `rust-port.yml` checks out the PR tree with no `.oracle`
step, and the `source-*` lanes in `ci.yml` run the root TypeScript. Which path the Node side
of `cargo xtask oracle` reads was not traced. Until it is traced and pointed at `.oracle`,
`git merge upstream-main` would change what the Rust is compared against, which this document
exists to prevent. Do that first.

**Expect conflicts in files the fork edited.** Of the files upstream also has, the fork has
edited `README.md`, `CHANGELOG.md`, `.gitignore` and `.github/workflows/ci.yml`. `CHANGELOG.md`
conflicts every time: each port PR adds an Unreleased bullet where upstream adds release
sections (it cost a fresh CI run three times on 2026-11-03: #64, #70 and #75). `ci.yml` is
restructured here (source-tests, source-mcp, source-browser and gate lanes, each with its own
budget), so an upstream edit to it, such as the 20 to 30 minute budget in 0.9.4, is
hand-ported and not merged. `CLAUDE.md` and `AGENTS.md` exist only in the fork and do not
conflict.

**A rebaseline is a pull, not a contribution.** It needs no CLA and no contact with the
maintainer, and CLAUDE.md still says not to contact him or sign anything without the owner's
word. Upstream is AGPL-3.0-only and the port is AGPL-derived, so merged upstream code keeps
its provenance in git. Licensing and naming stay parked.

**Cheap checks for every release**, from a checkout with the `upstream` remote:

```bash
git fetch upstream
git diff --stat reference/2cd89c6 upstream/main                    # size and shape of the gap
git diff reference/2cd89c6 upstream/main -- package.json package-lock.json mcp/package-lock.json   # pins
git diff reference/2cd89c6 upstream/main -- mcp/src/commands.json  # command surface
git diff --stat reference/2cd89c6 upstream/main -- src/core/parsers src/core/project src/app/project.ts   # formats
git show upstream/main:src/app/project.ts | grep -n PROJECT_SCHEMA # project schema number
```

For the ASCII parsers there is a differential harness in
[evidence/upstream-drift-2026-11-03/](evidence/upstream-drift-2026-11-03/); extend it rather
than writing a new one.

**Signs to decide early.** Write the decision, with its date and reason, in the ledger even
when it is "not yet": a `PROJECT_SCHEMA` change; a bump of `fflate`, `meshstep` or
`occt-wasm`; a security advisory or fix touching a format we read or the MCP server; a new
workspace or plugin; a changed command schema; drift in an area where units are about to be
accepted.
