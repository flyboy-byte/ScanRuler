# Hosted CI run of the original, 2026-10-31

Run: https://github.com/flyboy-byte/scanruler/actions/runs/41243336713 (workflow "CI", job `test-and-build`, ubuntu-24.04, **Node 22**), commit `fabc28e01c832deccf167e3ecc423c3f8f1f049b` on the fork's `main`.

The original's source tree at that commit is identical to the pinned reference: `git diff --stat 2cd89c6 fabc28e` over `src plugins tests mcp scripts package.json package-lock.json index.html vite.config.ts tsconfig.json` is empty. The port added only `rust/`, `docs/`, `.github/`, root docs, README and `.gitignore`.

| Step | Result |
| --- | --- |
| `npm ci` | pass |
| `check-plugin-boundary.mjs`, `check-units.mjs` | pass |
| `npm test` (vitest) | **371 files passed, 13 skipped (384); 3084 tests passed, 24 skipped (3109)**; duration 488.9 s |
| `npm --prefix mcp test` | 123 tests, 123 pass, 0 fail, 0 skipped |
| `npm run build` (tsc + vite) | pass |
| `npm run e2e:ci` | every suite printed ALL PASS; 15 suites logged; production smoke: `ALL PASS`, console errors none |

Skipped test files (13) are those needing the uncommitted private scans (`ballbar.stl`, `side bracket left.stl` / `block-marius.stl`) or that are benchmark-gated: `tests/align.test.ts` (2 of 3), `tests/ballbar.test.ts`, `tests/agentBallbar.test.ts`, and reverse-plugin files such as `reGrowth`, `reJoycon`, `reJoyconReads`, `reClampLevels`, `reLampSplit`, `reNetworkBench`, `reToolBytes`. Files with partial skips (`reSurfaceStrays`, `reTubeTrace`, `reShortEdges`, `reLoftFold`, `reduce`, `reCutElsewhere`, `reStepsCommands`, `wireSlots`) are counted in the 24 skipped tests. **These are skips, not passes**: any parity claim on them needs the scans or a generated substitute.

## What this is not

- Not a local baseline: nothing was run on this machine, and local Node is 20.20.2.
- Not a full P01: `npm run build:engine` (headless), the live command catalogs (P02), lockfile hash verification, browser version and machine specs are not captured. Raw log not archived (re-fetch with `gh run view 41243336713 -R flyboy-byte/scanruler --log`; GitHub expires logs).
- Says nothing about the Rust port. It shows the original, unchanged, is green in a clean Node 22 environment.
