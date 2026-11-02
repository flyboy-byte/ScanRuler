# Original reproducibility observation — 2026-11-02 UTC

Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (0.9.2).
Capture implementation: `0a42dca` (full SHA in `local.json`).
This is baseline evidence, not Rust numerical parity or a G0 pass.

## Local observation

Node 22.23.4 through `fnm`, npm 10.9.9, system Arch Rust 1.99.1 and Chrome
156.0.8078.79; exact observed identities and lock hashes are in `local.json`.
The inventoried original source/tests/scripts/plugins/configuration and npm
manifests/locks have no byte changes against the frozen reference. Port-only
tooling commits occurred while these checks ran; original inputs did not change.

| Complete original command | Exit/result |
| --- | --- |
| Plugin boundary | 0 |
| Display-unit boundary | 0 |
| Unit suite, default scheduling and unchanged thresholds | **1**: 3,080 passed, four failed |
| MCP suite | 0 |
| Production build | 0 |
| Complete browser/production regression suite | 0 |

The four unit failures were two 20-second timeouts (`objTexture.test.ts` and
`volumeDeviation.test.ts`), the shell diagnostic assertion at
`plugins/reverse/tests/reBuild.test.ts:755` (the source reported its bisection
budget expired), and the unchanged 1,500 ms crease budget (observed 1,588 ms).
Vitest also reported an unhandled worker `onTaskUpdate` timeout. These remain
failures. They were not rerun or waived. No inference that all are harmless flakes
is needed to preserve this observation.

The console summary counts 24 skipped tests. Individual JSON entries additionally
contain one nested `pending` real-scan test, for **25 explained excluded entries**.
Both the raw summary and each individual status are retained without reconciling
them into an invented passing count. `local.json` lists each name and matching
source guard; the complete original guard inventory has 22 sites.

The first launch before `npm ci` returned 127 (`vitest` unavailable). That was a
setup failure, before any test executed; the lock-preserving installation then
enabled the single complete unit run above. Both original packages were installed
with `npm ci`; no lock was regenerated and no dependencies were upgraded.

Raw command logs and unit JSON are committed as deterministic gzip, with compressed
and decoded hashes in `raw-manifest.json`. These inputs were the default run with
private opt-ins absent, not a run on any user's private projects. The original
report includes local checkout paths, not private scan contents.

## Hosted observation, kept separate

`hosted-main.json` records the complete original CI at main
`551b3a956e37765b5de83838db1718e62f156e53`:
[run 41387393441](https://github.com/flyboy-byte/ScanRuler/actions/runs/41387393441).
All three original lanes and their aggregate passed. This is a different hosted
environment and checkout, whose original code is identical to the same pin; it
does not overwrite the failing local observation. Hosted logs are retained by
the existing workflow, not copied into the local raw manifest.

## Fixtures and remaining gaps

Six existing source recipes (ball pair, fillet ring, turned block, noisy closed
and open boxes, notched block) each generated identical bytes twice. Generator,
recipe, seed where applicable, mm units, source license, output hash and triangle
count are in `local.json`. No mesh bytes or expected results came from Rust.

Private root scans remain absent and their tests excluded. Optional project and
benchmark inputs are listed by source guard, without reading their environment
contents. These six recipes do not complete P03's adversarial/project/image corpus
or source dependency map. Hosted Rust remains floating stable; a fully immutable
toolchain/container and a fully green supported local original run remain open.
