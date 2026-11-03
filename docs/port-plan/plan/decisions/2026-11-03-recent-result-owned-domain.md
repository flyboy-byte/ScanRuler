# Reuse owned data for the retained-result branch

2026-11-03. The native schema Value already represents command data losslessly,
including undefined, bytes, nonfinite numbers and UTF-16. Reuse it for this bounded
keptResult domain rather than invent a parallel object/JSON store. Count exact
source JSON units and normalize into an independent owned tree. The original's
arbitrary-host-object exception classification is an explicit missing continuation,
not a claimed early-return port. Resource limits must not become 'not JSON'.

This child can be implemented and compared on owned data while its parent session
remains specified and unaccepted. Recent history snapshots will own/share the kept
tree according to their own source identity rules; this helper only creates it.
