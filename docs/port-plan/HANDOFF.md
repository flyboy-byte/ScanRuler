# Rust port handoff — start here

Updated 2026-10-31. This packet supersedes the preliminary migration phases in `../context/TANGENCY-RUST-ASSESSMENT.md`. It is a plan and evidence inventory, not a port implementation.

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

This directory contains plans, ledgers, source references and a verifier. No Rust workspace, port branch, prototype or application benchmark exists. Nothing was pushed or published. The earlier documentation snapshots in `/workspace/context` are not Git checkouts.

Environment observations: shell clone requests failed connecting to the configured `proxy:8080`; GitHub connector reads worked. Node 24.19.0, npm 11.9.0 and Chromium 154 are on PATH. `cargo`, `rustc`, `wasm-pack` and `trunk` were not found on PATH. Upstream CI uses Node 22. Local Node 24 is not an exact CI reproduction. No upstream tests, Rust tests or application sessions have been run. Planning artifact consistency and the live-plan checker's negative self-tests are verified. These are not application or numerical-parity checks.

## First implementation session

1. Read the current environment skill/status and network policy; use the configured proxy normally. Restore supported network access if it is still unavailable. Do not unset/bypass the proxy as a workaround. Missing network is an environment prerequisite, not a reason to fabricate a baseline result.
2. Obtain the full fork checkout, including assets, both lockfiles and test fixtures present in Git; verify its baseline commit. Inspect applicable AGENTS.md and contribution rules in that checkout. Set `origin` to the user's fork and `upstream` to CNC Kitchen; work on a local `port/rust-baseline` branch. Do not push merely because this plan names a branch.
3. Establish Node 22 for the upstream oracle and a pinned stable Rust toolchain with native and wasm32 targets. Record compiler, OS and browser versions. Complete P00–P03 in `backlog.json` before choosing numerical tolerances or selecting pure-Rust geometry implementations.
4. Run the complete upstream baseline from VALIDATION.md and capture all-plugin/page/headless catalogs. Generate the first redistributable fixtures. Missing private scans must appear as skips, never passes.
5. Prove pure-Rust kernel capabilities and missing operations early and separately, then implement the plane-fitting slice with native and browser-Wasm comparison. The upstream headless engine cannot open scans; use a browser oracle or direct pure-core harness for measurement comparisons.

The already-authorized next work is local planning, source inspection and implementation toward the port. This turn did not authorize sending messages, signing legal terms, deployment or publishing. Follow later explicit user instructions if they expand that scope.

## Completion and handoff discipline

Update a canonical unit file only with the actual achieved state, tested commit, evidence and open items; generate the aggregate backlog rather than maintaining two status systems. Stefan's pinned behavior/math are authoritative; source changes and deliberate behavior improvements are separate reviewed work. Mark an item complete only with the actual commit and evidence path. Update the exact baseline, statuses, blockers and next command after every substantial session. Retain failed/skipped evidence. Preserve immutable upstream and keep cutovers reversible. A runnable partial port is valuable but must name missing workspaces, commands, feature types and external dependencies.

For living plan validation, run `python tools/port_plan.py --root . check` from this packet, plus `python tools/port_plan.py selftest`. Per-unit files are canonical and support future status changes. The separate packet verifier checks this frozen all-planned export only; do not use its snapshot expectations as the living implementation CI.

For this packet's integrity, run `python /workspace/port-plan/verify.py`. After moving the packet, run `python <new-path>/verify.py`. The archive is portable; the verifier uses paths relative to itself.
