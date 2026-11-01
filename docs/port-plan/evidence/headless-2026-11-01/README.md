# Frozen original live headless catalogs — 2026-11-01

Actual successful [capture run 41352758112](https://github.com/flyboy-byte/ScanRuler/actions/runs/41352758112), candidate merge `8b9084f27f5d0678cf62d1c883c78dc0d13f8fb8` (PR #13's initial code), original ref `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, app 0.9.2. Node 22.23.4. The separate original npm run build:engine and adapter builds exited 0; their raw logs are retained here. Tracked original source and both lock blobs were verified clean before and after capture. Both original session.state calls returned viewport=false/scan=null.

| Original process | Installed plugins | Registered | Headless offered | Page-only |
| --- | --- | ---: | ---: | ---: |
| core | none (`?plugins=none`) | 72 | 6 | 66 |
| all | mesh-tools, reverse | 140 | 36 | 104 |

Core headless offers: guide.read, history.checkpoint, history.redo, history.restore, history.undo, session.state. No original headless scan.open support is inferred. Both processes are fresh; no singleton/plugin state is reused between captures.

The complete JSON includes every live CommandInfo schema/flag plus actual runsHeadless classification, all offered/page-only names, source/candidate identity, environment/locks and original/adapter module hashes. Saved as gzip with mtime=0 to avoid a 600 KB text diff; this is a lossless data snapshot, not hand-edited expected output. Manifest records raw/compressed SHA-256s, command metadata hashes and sizes. Decode with `gzip -dc core.json.gz > /tmp/original-core-catalog.json` (and likewise all.json.gz), then inspect fields. Generated source schemas/descriptions remain AGPL-derived under this repository's license; these are not a clean-room schema design.

registry.csv reconciles the original static commands.csv with the actual all-plugin registry: all 140 names match exactly, no missing or extra names; all 72 core names are present in all mode. It provides observed startup mode membership and flags without rewriting the historical static ledger's browser-unverified claims. Headless schemas do not prove the browser page registers identical metadata; actual browser core/all catalogs and 27 feature kinds' nested variants remain open under P02.

Canonical command hashes:

- Core: af1cf3d58378bebcf46d75c36ee9043a5d9c2b63cee697e24cd3d3f7e4248f13
- All: 19a2cd59faded4b4b6f0447d2aaeede556d410acaa62ab607b30c06eab22a2aa

Ten original validator tests passed hosted with the capture; the follow-up now has twelve local tests, adding full snapshot/hash validation and optional-field/schema-loss rejection. The workflow recaptures the pinned source and compares these command metadata hashes; changing a run timestamp/compiled context is permitted, losing an original command/property is not. This is an original metadata guard, not the unbuilt Rust numerical exact-base ratchet.

Reproduction and original API adapter boundaries: ../../oracle/HEADLESS-CATALOG.md. Outputs and negative tests are real original/capture-tool evidence only. No Rust command implementation or whole-app parity is claimed, and parent P00/P01/P02 stay incomplete. The generic Rust coordinator/tooling endpoint remains required.

## Actual original production bundle cross-check

[Run 41353681379](https://github.com/flyboy-byte/ScanRuler/actions/runs/41353681379)
also passed both fresh starts of the actual original dist-engine/engine.mjs.
Its complete offered schemas, page-only names, plugins and source startup state
match the adapter captures in core/all modes. built-core.json/built-all.json
retain the verification. The original engine byte hash is identical to the
initial snapshot's module: 03e98e13651d7de9f734526876de0f70163bc35ab69d6ad06d303954a29eac33.
This links the captured metadata to the actual built original public API, not
only the API re-export entry. Canonical schema fingerprints also matched the
frozen snapshots on recapture. No Rust engine was substituted.
