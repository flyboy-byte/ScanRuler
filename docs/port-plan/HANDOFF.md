# Rust port handoff — start here

Updated 2026-11-02. This packet supersedes the preliminary migration phases in `../context/TANGENCY-RUST-ASSESSMENT.md`. It is the entry point to the plan and evidence inventory; the Rust scaffold is in `../../rust/`.

## User intent and current decisions

The user wants to port the **whole public ScanRuler repository** to Rust and explore its fit with Tangency Labs. Preserve the product's six workspaces and automation, not just fitting mathematics. The user sent Stefan a licensing enquiry after our discussion. Its exact text and any reply are not available. Do not contact him, sign a CLA, or assume new rights were granted.

Confirmed user target: **pure Rust throughout, including every geometry kernel**, aligned with Tangency's direction and pursued as the user's personal side project. Read `PURE-RUST.md`. Retained non-Rust kernels may be temporary comparison/migration aids only. Kernel, STEP/tessellation and mesh replacements are required work on the main dependency graph, not optional follow-ons.

An AGPL port can proceed while the licensing enquiry is pending. Source-informed port work remains AGPL-derived. Incorporation into permissive Tangency projects additionally depends on suitable rights and their project policy/acceptance. This is not a clean-room reimplementation: focused ScanRuler implementation source was read during planning. Do not later label this implementation or source-derived tests clean room. Tangency policy was not imposed as a prohibition on working in the user's standalone fork.

## Read order

1. This file for authorization, status and immediate work, then [ROADMAP-TO-USABLE.md](ROADMAP-TO-USABLE.md) (dev environment, milestones to a usable build, MCP and test-scan strategy); its section 2 is the workflow and merge protocol, and `tools/next_task.py` picks the next unit.
2. `PURE-RUST.md`, then `PLAN.md`, for the confirmed endpoint, milestones, critical path and release gates.
3. `ARCHITECTURE.md` for runtime boundaries and unresolved backend decisions.
4. `VALIDATION.md` for numerical, file, UI, protocol and performance acceptance.
5. `GAPS-AND-DECISIONS.md` for corrections to the first plan, blockers and decisions.
   Also read `DEPLOYMENT.md`: pure Rust can retain static browser hosting through Wasm and add native packages; no geometry server is required.
6. `PORTING-PROCESS.md` and `CI-AND-REGRESSIONS.md` for contributor claims, living-plan refinement, authoritative reference comparisons and our CI/regression gates.
7. Canonical `plan/units/<id>.json`, generated `backlog.json`, `commands.csv`, `features.csv`, `port-map.csv`, and `reference/` for the work inventory/evidence. `repo-bootstrap/` retains historical contributor/CI templates; active workflows live in `../../.github/workflows/` and are described in CI-AND-REGRESSIONS.md.

## Current implementation and evidence — 2026-11-01

Read [STATUS.md](STATUS.md) for the generated unit inventory, [AUDIT-QUEUE.md](AUDIT-QUEUE.md) for PR order and hosted outcomes, and [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md) for the next bounded increments. These are different views: canonical `plan/units/*.json` own lifecycle state; an inventory is not a runtime report or a completion percentage.

The pinned baseline remains `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, app 0.9.2. `main` is the port trunk, `upstream-main` the upstream mirror, and `.oracle` the frozen checkout; see UPSTREAM-SYNC.md. The original TypeScript source/tests and npm locks remain unchanged by the numerical port queue.

Rust library work exists: binary/ASCII STL, ASCII/both-endian binary PLY, welding/filtering, full mesh-graph construction, adjacency/components/selection and shell parity, bounds/centroids, normals/orientation, dense solves, TLS/clipped/robust planes and curvature discrimination, plus complete marked-selection plane element geometry/statistics. All numerical child units remain **implemented, not accepted**. `port-engine` has typed error envelopes only; sessions/registry/jobs/history remain unimplemented, the CLI prints a banner, there is no Rust UI and no end-to-end Rust scan/fit/project command workflow.

The native suite/case inventory is generated in STATUS.md; case counts include refusals and are not counts of successful numerical measurements. The existing tests compare actual values/state against independently executed frozen bodies. Their fixed cases, seeds and numerical budgets remain authoritative. Actual Chrome-Wasm comparisons cover TLS plane, complete mesh graph, marked plane and both PLY bodies (341 inputs: 287 successful outputs / 54 source refusals; one large PLY vertex workload explicitly native-only); see oracle/BROWSER-WASM.md and its immutable report. Other numerical units and broader runtime platforms/corpora remain open. The generic P36/P37 paired report/exact-base ratchet remains unbuilt; compilation alone never filled the runtime gap.

Original command baselines were captured in real Node 22 headless engines and Chrome pages, both core-only and all-plugin. Core: 72 registered / 6 headless-capable; all: 140 / 36. Full schemas and immutable hashes are under `evidence/headless-2026-11-01/` and `evidence/browser-2026-11-01/`. Metadata agrees across contexts; that does not prove all commands execute, feature variants are complete, or Rust implements them. Catalog children are baseline-captured; P02/G0 remain incomplete.

The unchanged original has both green hosted full runs and a repeatable intermittent timing failure in `reNetworkCreases.test.ts:209` (1500 ms limit). The audit queue retains exact runs/results. No threshold, seed or test is waived. Missing private scans remain skips. The older scaffold-only handoff and baseline observations are history, not current status.

Licensing, trademark and rename questions are **parked** by the owner until the project is more mature and Stefan has replied to Logan's email about the SolidCraft integration; see [LICENSING-AND-NAMING.md](LICENSING-AND-NAMING.md) (reference only). The next unit comes from `tools/next_task.py`.

## Working and validating

Local host: system Rust, no rustup; Node is oracle-only (user-space `fnm` Node 22; `cargo xtask oracle` selects it itself). Hosted jobs provision Rust stable and Node 22 and record versions. The hosted Rust toolchain is not yet locked to an immutable release; record that reproducibility gap under P00. Do not change local machine setup based on hosted installation commands.

```bash
cd rust
cargo xtask ci       # pin, generated plan/status, negative tooling tests, fmt/lints/tests, Wasm compile
cargo xtask oracle   # all required native original/Rust suites, no filtered cases
```

From the fork root, `python3 docs/port-plan/tools/ci_report.py status --check` checks generated inventory; `port_plan.py --root docs/port-plan check-generated` rejects stale backlog. After editing canonical units, run `port_plan.py --root docs/port-plan render` and `ci_report.py status`.

CI results are separated: Rust fast checks, native oracle assertions/raw evidence, original catalog contracts, original unit tests, original MCP, and original build/browser regression. Final `gate` and `test-and-build` require every respective lane to succeed. Original source unit failures no longer prevent MCP/browser evidence from running. See CI-AND-REGRESSIONS.md for artifacts and gaps.

## Review and next implementation

PR #31 ports the complete decoded-RGBA OBJ material/texture helper module, with
144 exact source observations. io.obj.mesh owns the parseOBJ continuation and
io.obj.companions owns file bundling/PNG-JPEG ingress; parent io.obj stays planned.
The independent fuzz agent writes only to /tmp; its separate PLY fixes must not be
folded into these OBJ changes.

PR #30 starts the engine foundation with the original twelve-code error envelope
and 52 exact field/JSON observations, preserving UTF-16 messages. Session ownership
and execution are still absent. Reuse this type in engine.session.contract; do not
use human Display text as the wire message or claim registry normalization exists.

PR #27 adds complete ASCII PLY header/body decoding under `io.ply.ascii`, with
60 exact independent cases. PR #28 completes both binary bodies under
`io.ply.binary` (168 cases) and adds `ply::read_ply` for all encodings. Existing
ASCII comparisons now check that full dispatcher as well. Parent `io.ply` remains
planned for broader acceptance; PR #29 adds bounded actual Chrome-Wasm PLY
comparisons while engine/session integration and the large vertex workload remain open.

Baseline reproducibility tooling and a complete local observation were added in
PR #26; see [the retained results](evidence/baseline-2026-11-02/README.md). MCP,
build, browser and boundary checks pass locally; the original unit run has four
failures and a worker timeout. The independent hosted main run passes. All
exclusions and six repeatable fixture recipes are inventoried. This does not
complete G0 or the full P03 corpus; do not relabel the local unit run green.

PRs #1–#30 are merged into main at `fabac0d`, with its checks passed. The next open queue starts at #31 for io.obj.texture; see AUDIT-QUEUE.md and inspect live checks. This continuation has not merged anything. New work follows ROADMAP-TO-USABLE.md section 2. No deploy, legal signature or upstream contact is authorized by this plan. Existing user authorization covers fork PR submissions and, when Logan asks for it, integration per ROADMAP-TO-USABLE.md section 2.

The marked-plane element continuation is now implemented under fit.plane.selection; public command/history integration is still absent. Prioritize reproducible reference/fixture contracts, the remaining P05 session/error/history contract and wider browser-Wasm coverage, then a bounded engine import/plane command slice. Use the implementation plan's concrete exit conditions. Keep the kernel/purity capability spike parallel and explicit; more helper ports cannot substitute for command/session or kernel evidence.

Update each affected unit with its actual tested SHA, evidence and open scope. Preserve old reports and failed attempts. Generated status never promotes acceptance. The portable `verify.py`/SHA256SUMS bundle validates the historical planning export; it is not the live checker. Use the commands above for this checkout.
