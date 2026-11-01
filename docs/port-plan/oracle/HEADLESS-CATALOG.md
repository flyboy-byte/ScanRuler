# Frozen original headless catalog capture

This is original-oracle tooling only, not Node product/runtime code or Rust parity. It closes a bounded part of P01/P02: build the frozen original headless engine and capture its live full registry and offered/unavailable classification in fresh core-only/all-plugin processes. It does not capture a browser page or enumerate all feature parameter variants.

Requirements: Node 22, complete frozen checkout at reference-pin.json's exact commit, its npm dependencies installed, clean tracked source/locks, Git. The generated untracked entry only re-exports the original startEngine/listCommands/runsHeadless APIs. Original Vite config builds it; no source formulas/bodies or tracked files are changed. The separate original npm run build:engine command must also succeed. Failures remain failures, not empty catalogs.

From the fork root (after preparing .oracle per UPSTREAM-SYNC.md):

```bash
node --test docs/port-plan/oracle/catalog-contract.test.mjs
(cd .oracle && npm ci && npm run build:engine)
node docs/port-plan/oracle/build-catalog-adapter.mjs .oracle
node docs/port-plan/oracle/capture-headless-catalog.mjs .oracle core rust/target/original-catalog/core.json
node docs/port-plan/oracle/capture-headless-catalog.mjs .oracle all rust/target/original-catalog/all.json
```

Separate processes matter: original engine/stores/plugins are singleton module state. Core mode uses the original documented location query `?plugins=none`; all mode uses Node's normal no-location startup, with both baseline plugins enabled. Runtime schemas/flags are copied from actual registry APIs, not parsed from declarations or substituted from static commands.csv. The original session.state command must return viewport=false and scan=null.

Each raw JSON records frozen and candidate SHAs, app/Node/V8/OS/locale, verified lock Git blobs plus SHA-256s, both built module hashes, installed plugins, all registered command schemas/flags, offered names, page-only names and canonical command hash. Object keys are canonicalized; array order is retained. The full registry is the headless runtime's registry, not proof that a browser page registers the same commands. The original's public headless API supplies only unavailable names; the thin API re-export makes its full registry metadata observable without changing source.

Validation rejects zero commands, duplicate names, missing schemas/flags, offered/page list inconsistency, missing source startup invariants and incorrect plugin selection. Nine invalid mutations are tested alongside a valid synthetic validator case. Synthetic validator fixtures are not claimed as original results. Local Node 20 can test that pure validator; capture itself refuses anything other than Node 22.

The independent original-catalog.yml workflow checks out the exact frozen ref separately, runs these steps and retains raw files/build failures for 30 days. Save successful generated JSON snapshots with source/run provenance before artifact expiry. Do not claim G0/P02 acceptance until browser contexts, static-row reconciliation, nested feature variants and the remaining original baseline/tool prerequisites are covered. Active oracle coordination will need the planned Rust tooling/purity work before whole-port completion.

## Captured baseline

Successful Node 22 run and complete schemas are saved under
[headless-2026-11-01](../evidence/headless-2026-11-01/README.md): core 72/6/66 and
all-plugin 140/36/104 (registered/headless/page-only). All 140 static command
names reconcile exactly. Twelve validator/snapshot tests now guard full hashes
and optional-field/schema loss. After capture, run:

```bash
node docs/port-plan/oracle/verify-catalog-snapshot.mjs rust/target/original-catalog
```

This hashes complete command metadata with only object-key canonicalization,
retaining all array order and values. Run context can differ, source metadata
cannot silently shrink. A new pin or changed baseline requires separate source
evidence/review, not blessing Rust-produced outputs.
