# Bounded live original headless catalog capture — 2026-11-01

Unit catalog.headless (P02), independent branch from main. Earlier numerical ports retain missing G0/P02 evidence; their helper comparisons do not establish complete command coverage. This contribution deliberately observes the pinned original rather than inferring a command inventory from the Rust implementation.

The original registry's public headless API filters metadata to offered commands and lists unavailable names only. An untracked two-export adapter, built by the original Vite config, exposes that same live registry's full CommandInfo metadata and runsHeadless classification. Tracked reference sources/locks remain clean and are verified before/after build/capture. Separate processes run documented core-only query and all-plugin startup. This is a headless-runtime registry, not an asserted browser-page catalog.

Original npm run build:engine is a separate required step. Capture is blocked on actual Node 22 frozen-engine startup and session.state invariants; local Node 20 only tested the ten pure validator cases. No actual catalog result is claimed before hosted evidence. Node code here is original-oracle coordination only, with whole-port Rust tooling still a required later task.

Browser core/all-plugin registries, static ledger reconciliation and nested feature variants remain open. The canonical unit is added without rewriting backlog.json on this independent branch, following contributor guidance to regenerate aggregates in the integration batch. The live checker reads canonical units and passes; do not mistake the older generated index for the new unit's state.
