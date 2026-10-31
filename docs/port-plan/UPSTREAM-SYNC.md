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

Run the original from a worktree of the tag, not from `main`, so merging upstream can never change
what the Rust is compared against:

```bash
git worktree add ../scanruler-ref reference/2cd89c6   # once; needs Node 22 there for npm ci
```

Record the oracle's commit in every result (`reference-pin.json` names it).

## Bringing in upstream changes (a rebaseline, never automatic)

1. `git fetch upstream && git checkout upstream-main && git merge --ff-only upstream/main && git push origin upstream-main`
2. Triage `git log reference/<old>..upstream-main` into a drift ledger (`plan/findings/`): commit,
   affected feature, security or data-loss relevance, backport or defer. Security and data-loss
   fixes first.
3. Only for a chosen rebaseline: branch from `main`, `git merge upstream-main`, resolve (expect
   `README.md` and `.gitignore` at most), update `reference-pin.json` and `port-core`'s
   `REFERENCE_SHA`, tag the new `reference/<sha>`, rerun the oracle baseline and catalogs, and
   re-compare every accepted unit. Affected units fall back to their last verified state until
   they pass again.
4. Until then `main` need not contain upstream's newer commits, and the port still targets the
   pinned one.

Never rebase `main` onto upstream, and never commit port work to `upstream-main`.
