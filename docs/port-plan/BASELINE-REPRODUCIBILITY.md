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
