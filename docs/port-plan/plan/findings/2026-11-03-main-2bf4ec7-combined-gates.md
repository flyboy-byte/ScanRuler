<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Merged-main combined gate result — 2026-11-03

Tested the clean merged tree `2bf4ec72babc35f27cc212e79e1bb57e68f3982a` before implementation.
The checkout was refreshed with `git switch main` and `git pull --ff-only`;
`gh pr list -R flyboy-byte/ScanRuler --state open` returned no PRs.
Reference remains `2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (app 0.9.2).

## Commands and observed results

From `rust/`, in order (Node 22 through fnm, system Rust):

1. `fnm exec --using=22 cargo xtask ci` — exit 0. Plan checks, all 168 Python
   tests, formatting, Clippy, workspace tests and Wasm compilation passed.
2. `fnm exec --using=22 cargo xtask oracle` — exit 0. All 57 required registered
   native suites discovered and passed. The command also runs the unregistered
   3MF and ZIP/inflate ignored comparisons: 3MF 1,419 identical successes,
   1,850 identical refusals, one separately reported host limit, zero divergences;
   ZIP/inflate 7,516 identical successes, 1,480 identical refusals, four separately
   reported host limits, zero divergences. These default runs do not replace the
   larger retained 3MF review corpus.
3. `python3 ../docs/port-plan/tools/ci_report.py evidence --candidate 2bf4ec72babc35f27cc212e79e1bb57e68f3982a`
   — exit 0; raw candidate/reference identity and required files complete for all
   57 registered suites / 22,938 declared case identities including refusals.

The working tree remained clean throughout. No gate failed, no source or
expectation changed to obtain this result. This is a local merged-tree result,
not a hosted result, browser run, acceptance claim or exact-base ratchet.

## Retained local records

Logs and the complete generated raw-evidence inventory are retained at
`.port-run-artifacts/nov03-main-2bf4ec7-validation/` (ignored; no private scans).
The inventory points to the raw `rust/target/*-diff/run-*` artifacts.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `main-2bf4ec7-ci.log` | 30952 | `8a55964ab352ada42f53136e5f46b7653952af6c048b8bc3c3882e5f4c13bf0e` |
| `main-2bf4ec7-oracle.log` | 220768 | `b4497f781c12364832e9f745ec75e148b13a5f72f4cd88b62dfef5735f8144f3` |
| `main-2bf4ec7-evidence.log` | 80 | `44501d17c1f6955efb466e5542ae2d3796d04e19505c396d24c110433f4f8bc8` |
| `native-evidence.json` | 26278518 | `5abe961d42e9a5ce4c9a249e0ab36dec2b35b98aef75d0d561f03fa953b43e04` |

## M3 still lacks

The CLI still prints its banner. Source-ordered retained-body reactions,
real command execution and session state, a real import command, plane fit,
undo and project-compatible save still need wiring and paired workflow evidence.
The two #58 3MF review findings are next, before `engine.runtime.reactions`.
No CHANGELOG entry here; the session entry belongs in its last PR.
