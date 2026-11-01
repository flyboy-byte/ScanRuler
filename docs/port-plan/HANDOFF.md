# Rust port handoff — start here

Updated 2026-11-01. This packet supersedes the preliminary migration phases in `../context/TANGENCY-RUST-ASSESSMENT.md`. It is the entry point to the plan and evidence inventory; the Rust scaffold is in `../../rust/`.

## User intent and current decisions

The user wants to port the **whole public ScanRuler repository** to Rust and explore its fit with Tangency Labs. Preserve the product's six workspaces and automation, not just fitting mathematics. The user sent Stefan a licensing enquiry after our discussion. Its exact text and any reply are not available. Do not contact him, sign a CLA, or assume new rights were granted.

Confirmed user target: **pure Rust throughout, including every geometry kernel**, aligned with Tangency's direction and pursued as the user's personal side project. Read `PURE-RUST.md`. Retained non-Rust kernels may be temporary comparison/migration aids only. Kernel, STEP/tessellation and mesh replacements are required work on the main dependency graph, not optional follow-ons.

An AGPL port can proceed while the licensing enquiry is pending. Source-informed port work remains AGPL-derived. Incorporation into permissive Tangency projects additionally depends on suitable rights and their project policy/acceptance. This is not a clean-room reimplementation: focused ScanRuler implementation source was read during planning. Do not later label this implementation or source-derived tests clean room. Tangency policy was not imposed as a prohibition on working in the user's standalone fork.

## Read order

1. This file for authorization, status and immediate work.
2. `PURE-RUST.md`, then `PLAN.md`, for the confirmed endpoint, milestones, critical path and release gates.
3. `ARCHITECTURE.md` for runtime boundaries and unresolved backend decisions.
4. `VALIDATION.md` for numerical, file, UI, protocol and performance acceptance.
5. `GAPS-AND-DECISIONS.md` for corrections to the first plan, blockers and decisions.
   Also read `DEPLOYMENT.md`: pure Rust can retain static browser hosting through Wasm and add native packages; no geometry server is required.
6. `PORTING-PROCESS.md` and `CI-AND-REGRESSIONS.md` for contributor claims, living-plan refinement, authoritative reference comparisons and our CI/regression gates.
7. Canonical `plan/units/<id>.json`, generated `backlog.json`, `commands.csv`, `features.csv`, `port-map.csv`, and `reference/` for the work inventory/evidence. `repo-bootstrap/` supplies installable contributor/CI templates; none are activated remotely.

## Baseline and evidence

- Fork: https://github.com/flyboy-byte/ScanRuler; upstream: https://github.com/CNCKitchen/ScanRuler.
- Frozen port baseline: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, app 0.9.2, MCP helper 0.5.0. It matched upstream when initially compared. Do not silently change the port target to current main.
- Git tree: 1,246 tracked files. `port-map.csv` maps every baseline file to a proposed disposition; mappings are rule-based and require review.
- Static command ledger: 72 core command descriptors from `mcp/src/commands.json`, 49 reverse registrations and 19 mesh registrations. **140 candidate commands, not a runtime-verified complete catalog.** Capture live core-only, all-plugin and headless registries before implementation coverage is claimed.
- 27 feature interfaces extending FeatureFlags were identified. Their variants, parameter schemas and behavior are not fully audited. `features.csv` is a starting ledger, not proof of parity.
- Source review was selective. Full immutable source links/blob IDs are in `reference/source-review.json`; quoted source-derived observations are in `reference/findings.md`. Core descriptors are saved as `reference/core-commands.json`.

## What exists and what does not

The repository now contains a Rust workspace with `port-core`, `port-engine`, `port-io`, `port-cli` and `xtask`. Core scaffolding includes units, identifiers and rigid transforms; the engine and IO crates are empty and the CLI prints a banner. See `../../rust/README.md`. No original-versus-Rust comparison has been established, so these modules do not establish numerical parity.

`cd rust && cargo xtask ci` exists: it checks the reference pin, live plan and checker self-tests, formatting, clippy, workspace tests and compilation of the library crates for wasm32. `.github/workflows/rust-port.yml` runs this fast gate. `cargo xtask diff` and `diff-ci` remain unimplemented. A Wasm compile check is not a browser execution check.

A hosted Node 22 baseline of the unchanged original is recorded in [evidence/2026-10-31-hosted-ci-baseline.md](evidence/2026-10-31-hosted-ci-baseline.md). It includes original unit/MCP tests, build and browser suites, with skips retained. It is partial P01 evidence, not a local baseline or a Rust comparison. Headless build, live catalogs, lockfile verification and environment details remain open as specified in P00–P03. Historical environment observations in that evidence are not a current machine inventory.

## Next implementation session

1. Read `UPSTREAM-SYNC.md` and inspect current branch, worktree status, open PR claims and the relevant canonical unit files. Use an isolated branch from `main` for a bounded contribution to the fork. Do not recreate the workspace or change the frozen reference.
2. Inspect the current environment rather than assuming the planning host's tools or network failures still apply. Use the system Rust toolchain on Logan's Arch machine; do not install rustup (decision `plan/decisions/2026-10-31-no-rustup.md`). Record `cd rust && cargo xtask env`; the reported known Wasm target still needs `cargo xtask wasm` to prove compilation. CI uses its own runner toolchain.
3. Finish the remaining P00 prerequisites: Node 22 for the original oracle, recorded browser/tool versions, and lock hashes checked against `reference-pin.json`. Run the original from the frozen `reference/2cd89c6` worktree in `.oracle/`, following `UPSTREAM-SYNC.md`; the root TypeScript source remains untouched.
4. Complete P01's missing baseline evidence, then P02's live core-only/all-plugin/headless catalogs and P03's fixture/dependency audit. Preserve hosted evidence and individual failures/skips. Do not count private scans as passing. Keep unit files authoritative and regenerate `backlog.json` after status edits.
5. With G0 evidence in place, prove pure-Rust kernel capabilities separately and implement the plane-fitting slice with independent original/native/browser-Wasm comparisons. The original headless engine cannot open scans; use a browser oracle or direct pure-core harness for measurement comparisons. Do not select tolerances from observed Rust discrepancies.

Local source inspection, planning and implementation are authorized. Public actions require the owner's explicit instruction for that session; instructions to open a PR authorize that PR, not deployment, legal signatures or contact with the upstream author.

## Completion and handoff discipline

Update a canonical unit file only with the actual achieved state, tested commit, evidence and open items; generate the aggregate backlog rather than maintaining two status systems. Stefan's pinned behavior/math are authoritative; source changes and deliberate behavior improvements are separate reviewed work. Mark an item complete only with the actual commit and evidence path. Update the exact baseline, statuses, blockers and next command after every substantial session. Retain failed/skipped evidence. Preserve immutable upstream and keep cutovers reversible. A runnable partial port is valuable but must name missing workspaces, commands, feature types and external dependencies.

For living plan validation, run `python tools/port_plan.py --root . check` from this packet, plus `python tools/port_plan.py selftest`. Per-unit files are canonical and support future status changes. The separate packet verifier checks this frozen all-planned export only; do not use its snapshot expectations as the living implementation CI.

For the historical frozen packet's integrity, run `python docs/port-plan/verify.py` from the repository root. Its snapshot assumptions may no longer hold after living-plan changes; use the live checker for current work. The archive is portable; the verifier uses paths relative to itself.
