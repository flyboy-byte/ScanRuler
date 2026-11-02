# Whole-repository Rust port plan

Updated 2026-11-01. Current library evidence: [STATUS.md](STATUS.md). Remaining execution increments: [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md). The roadmap gates below remain unpassed unless canonical units establish acceptance.

## Product contract

Port the baseline public product: 3D Measure, Deviation, Wall thickness, 2D Measure, Reverse engineer and Mesh Tools; file import/export; projects and recovery; agent guides and MCP; native/web distribution, documentation and release tooling. The original product is browser-first; desktop is an added target, not an excuse to lose browser capability. The closed beta workspace mentioned upstream is absent and cannot be ported from this fork.

All published functionality at the pinned commit is in scope. An omitted item requires an explicit ledger entry and prevents a whole-port parity claim. New features, regulatory/certified metrology claims, Tangency upstream changes, automatic CAD editing from inspection data and an unrelated general CAD platform are outside the first complete application release. Required pure-Rust geometry/kernel replacements remain in scope.

The confirmed completion target is **pure Rust throughout, including geometry kernels**, as clarified by the user. This is a personal side project directed toward Tangency's Rust ecosystem. See `PURE-RUST.md` for the precise dependency boundary and required geometry workstreams.

- **R1 — Internal application-parity checkpoint.** All six workflows and application/runtime behavior are validated. Any retained external geometry or Node/TS development runtime makes this an intermediate preview, not completion of the user's request.
- **R2 — Required whole-port endpoint.** Application, exact CAD kernel, mesh algorithms, formats, UI, MCP and active executable tooling are Rust. The shipped dependency/artifact audit excludes retained C/C++ geometry, even through Rust FFI or Wasm, and excludes original TS feature implementations and Node sidecars. Platform interfaces/minimal browser glue and non-code assets are explicitly accounted for.

Mixed-language previews and the immutable original application are useful migration/reference tools. They do not satisfy R2. Kernel/mesh/STEP replacement starts early; do not build a production native OCCT shim as a prerequisite. There is no optional external-kernel endpoint. A complete app with incomplete Rust kernel coverage remains incomplete overall.

## Repository and change strategy

Use the user's existing fork, preserve its upstream history and LICENSE/notices, and pin the reference commit. Keep Rust work under a new `rust/` directory initially. The root TS app remains runnable as the comparison oracle. Add tests/adapters without rewriting original algorithms until their replacement passes. Commit each cutover with its old/new evidence and one-step rollback. Do not delete upstream tests at the time the Rust path first passes them.

`origin` is flyboy-byte/ScanRuler and `upstream` is CNCKitchen/ScanRuler. Preserve immutable reference `2cd89c6…`; discover any fork changes before creating work. Public naming/trademark replacement is a release prerequisite; internal crate names are provisional. No new official Tangency project or maintainer endorsement is assumed.

Upstream changes go into a drift ledger: commit, affected feature, bug/security relevance, decision to backport or defer. No automatic moving target. A rebaseline requires rerunning affected oracles and updating catalogs and project fixtures; security/data-loss fixes receive priority review. ~~This planning packet is outside Git~~ — resolved: canonical planning/evidence and the Rust workspace are tracked in this fork.

## Critical path and milestones

| Gate | Deliverable and mandatory evidence | Depends on |
| --- | --- | --- |
| G0: baseline reproducible | Full checkout, pinned tools, all upstream checks run, skip/failure register, catalogs captured, fixture licenses recorded | Supported network/toolchain; P00–P02 |
| G1: boundaries proven | Pure-Rust exact/mesh/STEP candidates assessed native/Wasm on an extruded profile/Boolean/STEP cycle; missing operations and resource behavior recorded | G0, P03–P04; does not wait for the whole fitting engine |
| G2: first usable measurement slice | Plane fit with fixed selection, file import, commands/state, project reopen and browser display; native and Wasm match analytic and upstream results | G0, P05–P09; may progress alongside G1 |
| G3: measurement engine parity | All fit families, regions, alignment, deviation, thickness, dimensions/tolerances, sections and calibrated 2D; legacy UI uses Rust paths | G2, P10–P15 |
| G4: persistence and automation parity | Projects including plugins/legacy versions, edited-copy remapping, recovery/undo, MCP pairing/jobs/files and native headless session work | G2 plus relevant G3/G5 features; P16–P18 |
| G5: reconstruction/mesh parity | All 27 starting feature kinds and their variants, solver, rebuild/rollback, surface/network modelling, mesh operations and export tested | G1, engine, persistence contracts; P19–P23 |
| G6: Rust UI and internal R1 parity checkpoint | All six workspaces on native/web, full interaction ledger, supported hardware/browser matrix, packaging/licenses/docs, no TS app or Node runtime required | G3–G5, P24–P28; frontend prototypes start during G2 |
| G7: required pure-Rust R2 completion | Exact kernel, STEP/tessellation and mesh capability corpora pass; dependency/tooling purity audited; complete product acceptance repeated | G6 and P29; required P31–P35 streams start during development |

The schedule is a dependency graph, not a strict single-file queue. Prove high-risk pure-Rust kernel and viewer boundaries early. Port reconstruction in bounded groups while fitting/UI mature; do not leave all modelling risk until the end. Do not parallelize ownership of session state, wire schemas or kernel arenas without agreed interfaces and isolated work branches. This plan does not instruct spawning agents.

## First vertical slice: current boundary

Bounded numerical libraries have already progressed while P00–P05 remain incomplete. That is useful partial work, not a passed prerequisite gate. Steps 2/3 below now have library implementations; CLI/session/project/browser integration remains open. Follow IMPLEMENTATION-PLAN.md for concrete remaining PR scopes.

1. Define units, frames, buffers, IDs, result/error envelope and the project compatibility reader contract.
2. Read a binary STL and a controlled indexed mesh. Import into Rust-owned storage with stable vertex IDs; compare bounds and topology to the baseline parser.
3. Fit a plane on an explicitly named selection. This isolates math from region growth. Compare exact synthetic planes, seeded noisy planes and degenerate input against an independent oracle and the original.
4. Run the same operation in native CLI and Rust/Wasm behind the old browser UI; check result, selection overlay, cancel and re-run.
5. Save/reopen the measurement recipe, undo it, redo it, and produce a CLI/MCP result. Use Rust-native headless support as an extension; do not pretend upstream scan commands work in its existing headless mode.
6. Add cylinder fit and surface region growth only after the plane slice passes. Commit each dependency separately so a numerical or rendering mismatch can be located.

The slice is a vertical product test, not a promise to implement the whole workflow in one commit.

## How implementation advances

Canonical work status lives in plan/units/<id>.json; backlog.json is a generated index. PORTING-PROCESS.md defines claims, refinement and evidence-backed transitions; CI-AND-REGRESSIONS.md defines our original-versus-Rust gate. Every feature row needs: source paths, behavior contract/defaults, native/Wasm support, input/output/error cases, project/history impact, oracle and UI coverage, backend requirements, owner/branch and evidence. The row progresses `planned → specified → baseline-captured → implemented → compared → integrated → accepted`. Preserve failing comparisons; no automatic tolerance loosening. Parent workstreams remain planned where their full scope is open; bounded children record their actual implemented/baseline-captured state. Generated STATUS.md is an inventory, not automatic acceptance.

Before replacing a path, compare it in shadow mode where possible: original result remains authoritative while Rust computes separately. One result is applied; never let two stores mutate the document. Behind an internal switch, select TS or Rust per completed operation. Remove a switch only when browser/native parity, undo/project behavior and failure cases pass. Do not double-execute mutations while comparing them; replay each against separate identical sessions.

The Rust frontend replaces workflow by workflow using the same engine; it does not receive a second business-logic implementation. Early prototypes must demonstrate a million-triangle viewport, exact pick-to-vertex mapping, section clipping, field colors and text labels before all panel layout work starts.

## Tangency integration track

Keep Tangency integration optional and out of the port's critical path while licensing is unresolved. A file/MCP experiment can establish value: SolidCraft nominal STEP → scan inspection → structured report; reconstructed STEP → new SolidCraft import. Preserve units, frame, source revision and tessellation provenance. STEP carries geometry, not recovered editable history.

If Stefan grants suitable rights, document their exact scope, third-party exceptions and downstream distribution rights; then verify Tangency's acceptance and current policy. Permission to translate or use commercially alone does not settle permissive redistribution. A SolidCore adapter remains subject to capability evidence even if licensing is solved.

## Estimation and stop rules

Do not estimate from file counts or agent concurrency. After G2 and G1, record actual effort spent on math, platform glue, debugging, test baselines and kernel failures. Estimate remaining feature families with optimistic/likely/adverse ranges and explicit dependencies. Re-estimate after the first surface/network and native-viewport spikes; those are unlike simple plane fitting.

If the pure-Rust kernel cannot yet preserve operations, handles or export, keep that capability incomplete and implement or validate the missing Rust functionality. Reference-kernel fallback is not a substitute for R2. If large-mesh memory exceeds limits, fix ownership/copying before porting more algorithms. Missing privacy-sensitive fixtures cannot be resolved by copying an unknown-license scan. A failed gate is evidence and a next task, never a reason to label a subset “whole port.”
