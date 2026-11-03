# Workspace names and installed metadata

2026-11-03. PR #83 ports complete publicWorkspace/internalWorkspace/workspaceNames.
Only elements maps to measure and measure to elements; every other UTF-16 string
is unchanged. Name enumeration copies the four source base names, then every
present workspace id in current installed order. Empty ids and duplicates are
preserved; plugins without a workspace contribute nothing. There is no new alias,
availability check, sorting or deduplication. Every call reads current metadata
and returns independent arrays/text.

306 independent source-first cases cover 18 raw strings over core-none, actual
baseline-default and 15 additional data-only probe-plugin masks. The real original
state.ts and plugin discovery/installation run unchanged. Probes are input modules
with metadata only, not behavioral stubs. Native inputs independently describe the
same ordered metadata boundary; native discovery/installation is not claimed.
Compare public/internal results, complete installed metadata, caller-modified name
arrays, fresh subsequent names and live metadata changes. Unknown/case-distinct,
empty, control/non-BMP/unpaired strings and absent/empty/duplicate workspaces are
included. Both original state/plugin registry modules remain byte-exact in retained
source maps; every probe input is hashed and checked against its mapped source.

The 3.4MB original graph hit Node's unchanged 2GB heap limit when reimported per
string, leaving 229 complete records. The reference now uses one isolated import
per installed configuration, restoring metadata between strings. All 306 inputs
stay required, and every byte of the prior incomplete output prefix matches. No
heap/timeout increase, numerical change, seed/case drop or source-function stub
was used. See the separate finding for the attempt/result record.

Native plugin/name/output-unit admission and fallible copying happen before a
partial name array can escape. Tests verify strict mapping, borrowed unchanged
strings, exact base copy bounds and preserved empty/duplicate entries. These limits
are host refusals, not new source policy. Installed metadata is plain owned typed
data; arbitrary JS getters/proxies and plugin discovery/runtime remain separate.
Full state readout, workspace commands, browser integration and M3 are still open.
Run cargo xtask ci/oracle from rust/; all previous cases remain required.
