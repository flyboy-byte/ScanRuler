# Reproducing the frozen original

Unit: `baseline.reproducibility`. Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

This unit records the supported original environment, actual full-suite results,
conditional test exclusions and fixture provenance. It does not advance numerical
acceptance or change the reference. Missing private scans remain missing; benchmark
opt-ins remain separate from the default test run.

## Scope and fixed exit conditions

- Verify the original source, tests, scripts, plugin implementations, package
  manifests and locks are byte-identical to the pinned Git tree.
- Run complete unit, MCP, build and browser checks under Node 22. Record every
  command's exit status, tools, raw log hashes and individual pending tests.
- Link each pending original test to its source guard and an explicit reason.
  An unmapped pending test is an inventory failure, not a generic fixture skip.
- Record generated fixture recipes, source and asset hashes, seeds and licensing.
  Do not check in private assets, environment variable contents or private paths.
- Preserve hosted and local observations separately. A timing failure remains a
  failure; no retry or changed threshold turns it into a passing observation.

The follow-on engine/session unit still requires independent source state traces;
this baseline is not evidence that Rust implements application commands.

## Reproduction and inventory

Use `fnm exec --using=22` for each Node/npm invocation on this host. Install both
locked packages with `npm ci` and `npm --prefix mcp ci`. Run the unchanged plugin
boundary and display-unit scripts, complete `npm test`, `npm --prefix mcp test`,
`npm run build`, then `CHROME=/opt/google/chrome/chrome npm run e2e:ci`.
On another host, point `CHROME` at its installed Chrome executable.

For individual unit outcomes, add Vitest's `--reporter=default --reporter=json
--outputFile=/tmp/scanruler-baseline/unit-results.json` after `npm test --`.
Keep each command's complete log and actual exit code, including failures.

`ci/original-exclusions.json` inventories every original conditional skip site.
The checker rejects added/removed guards and unexplained individual pending tests,
including a new excluded suite inside a previously known file. Reasons describe the
default run with optional inputs absent; private opt-in runs need their own report.
It does not read or publish the contents of optional environment variables.

```sh
fnm exec --using=22 node docs/port-plan/oracle/baseline-fixtures.mjs > /tmp/fixtures.json
fnm exec --using=22 python3 docs/port-plan/tools/baseline_capture.py \
  --units /tmp/scanruler-baseline/unit-results.json \
  --runs /tmp/scanruler-baseline/runs.json --fixtures /tmp/fixtures.json \
  --out /tmp/scanruler-baseline/report.json
```

`runs.json` is an array of six operator-observed records: `id` (`plugin-boundary`,
`display-units`, `unit`, `mcp`, `build`, `browser`), `command` (argument array),
`exit_code` (integer), and `log` (local complete nonempty log path). Capturing a
report records failures; `all_original_commands_passed: false` must remain false.
Logs and reports must come from the same checkout/run; the capture tool inventories
them, and is not an independent attestation of the operator's command execution.

The fixture adapter executes six fixed recipes twice and requires identical STL
bytes. Their generating source is pinned original AGPL-3.0-only code; license and
hashes are recorded. They are reusable baseline assets, not the complete P03
adversarial corpus or substitute real scans. Cross-platform trigonometric bytes
can differ; compare the recorded runtime and do not rebaseline source expectations.
