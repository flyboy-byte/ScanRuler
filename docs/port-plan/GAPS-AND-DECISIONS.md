# Gap review, decisions and remaining risks

## Corrections to the preliminary plan

| Gap in the first assessment | Evidence or reasoning | Correction / work item |
| --- | --- | --- |
| A headless measurement prototype was proposed without checking the upstream oracle | `src/headless/engine.ts` and `mcp/src/engine.js` explicitly exclude scans and viewport operations | Native Rust may add scan headless support; compare measurements with a browser or pure-core harness. P02/P06/P09. |
| MCP scope was treated as one generic command list | Saved catalog has 72 core entries; plugins add 49 reverse and 19 mesh registrations statically | Reconcile live page/all-plugin/headless schemas and flags. No completeness claim from 140 static rows. P02/P18. |
| CI was summarized incompletely | Actual CI separately runs MCP package tests, unit-display/plugin checks and selected browser suites | Capture/run the actual workflow; preserve separate lockfiles and record skips. P01. |
| “Keep the existing kernels” lacked a runnable Rust integration strategy | Source loads Emscripten/JS packages; ReKernel has handles, histories and custom retry/validation behavior | Assess pure-Rust exact/mesh/STEP capability early. Keep reference bridging disposable; no production native OCCT shim. P04/P19/P31–P35. |
| Pure Rust and Rust application were conflated | OCCT/Manifold/meshoptimizer remain non-Rust even with Rust wrappers | Distinct R1/R2 dependency manifests and completion gates. P28/P29. |
| Blanket f64 recommendation would alter parity and memory | Source meshes are f32; poses/solves are f64, and baked transforms round points to f32 | Preserve source semantics initially; precision upgrades are a separate change. P05/P07. |
| Project compatibility was framed as just zip + JSON | Top schema 1, reverse PART_VERSION 3, optional legacy defaults, unknown plugin members, packed STEP sources and edited-copy maps | Explicit migration and round-trip matrix; do not discard unrecognized state. P03/P16. |
| Data ownership/cancellation were generic | Workers, snapshots, drafts, committed state and shape arenas have different lifetimes; upstream timeout can partially change state | Version/epoch tokens, one session owner, scoped history, explicit timeout-compatibility exception. P05/P08/P17. |
| UI and modelling risk arrived too late | The viewer supports brushing, clipping, 2D, split views and reconstruction picks; modelling includes 27 feature interfaces and freeform networks | Early viewport and kernel proofs, reconstruction in grouped increments. P04/P20–P25. |
| No exhaustive migration ledger | File count was only a size estimate | All 1,246 tracked files mapped; commands/features have separate ledgers and acceptance states. P02/P03/P28. |
| Performance lacked workload and resource accounting | Millions of vertices, multiple workers/kernels, f32 graphs, undo and GPU copies | Fixed corpora, native/web baselines, cold/warm measurements, memory/cancel gates. P03/P10/P24. |
| “Full port” had no release/tooling definition | Whole repo includes helper runtime, docs/assets, scripts, browser APIs and deployment workflows | Scope R1/R2; retire runtime Node/TS by R1 and explicitly track remaining development tooling. P26–P28. |
| Fast-moving upstream could change success criteria during work | Frozen fork is one commit; dependency and app versions can advance independently | Immutable reference, drift log and deliberate rebaseline. P00/P28. |
| License negotiation could unnecessarily stop standalone progress | User can port under AGPL already; email grants no rights until answered | Standalone AGPL work proceeds. Tangency merge/relicense track remains conditional. P00/P30. |
| Old plan could overpromise effort or kernel readiness | Byte counts and primitive demos do not measure feature parity | Estimate after representative spikes; SolidCore integration has its own capability corpus. P04/P29. |

## Decision register

| ID | Decision / current working assumption | Status and revisit condition |
| --- | --- | --- |
| D01 | Target the complete pinned public product, all six workspaces; keep private beta out | Adopted from user request; rebaseline only deliberately |
| D02 | Standalone AGPL port while licensing enquiry is pending | Authorized direction; re-evaluate distribution rights on actual reply |
| D03 | Pure Rust throughout including geometry kernels is the required endpoint; retained kernels are reference/preview tools only | Confirmed explicitly by user; supersedes former external-kernel-first default |
| D04 | Retain original UI as transition/oracle, then replace it with Rust UI | Proposed; early viewer spike must validate native and web |
| D05 | egui/wgpu frontend candidate; minimal Rust workspace first | Proposed, not an installed dependency or proven frontend choice |
| D06 | Rust owns committed session state; browser has one authority per migrated operation | Proposed architecture; prove in plane slice |
| D07 | Preserve legacy project writer/reader semantics; new branding separate from legacy app marker | Required compatibility goal; test actual old-project readers |
| D08 | Preserve pinned reference transaction/history/timeout behavior; structural changes cannot silently change observable results | Confirmed reference-first port policy; stronger semantics require a separate reviewed change |
| D09 | Tangency linkage initially by standard files/MCP, no automatic upstream plugin assumption | Recommended experiment; no maintainer acceptance claimed |

When revising a decision, record why and which evidence invalidated it. Do not overwrite historical facts such as which source was reviewed or which commit was tested.

## Current blockers versus future risks

- **B01, observed environment blocker:** configured proxy not reachable from shell clone attempts. Connector reads work. Blocks obtaining/running a full reference checkout through normal shell network access. Resolve using supported environment configuration; do not bypass the proxy.
- **B02, observed tooling gap:** Rust tools not found on PATH; available Node 24 differs from upstream CI's Node 22. Verify/install via supported setup before claiming builds. Tool absence on PATH is not proof no installation exists elsewhere.
- **B03, unmeasured coverage:** no baseline runtime, catalog capture, browser replay or performance measurements yet. Static extraction is not a test run.
- **B04, technical risk:** pure-Rust exact-kernel, STEP and mesh capability incomplete/unverified. Dedicated early capability gate and required P31–P35 workstreams; no assumption that SolidCore is already a drop-in replacement.
- **B05, external decision:** Stefan licensing reply and Tangency acceptance unknown. Blocks permissive integration claims, **does not block AGPL port work**. No inbox access/contact was performed here.
- **B06, test-data/device gaps:** original optional private scans and target device hardware unavailable. Synthetic fixtures cover core work, but missing hardware/manual evidence stays explicit in release coverage.
- **B07, product risk:** pure-Rust kernel replacement may be much larger than app port and may not reach all geometry cases. Required backend roadmap on the main dependency graph, feature capability reports and final R2 gate.
- **B08, maintenance risk:** translated feature behavior can diverge from upstream fixes. Freeze baseline and review high-impact upstream changes, preserving provenance for backports.

## Source and licensing handling

The user reported sending the enquiry; no reply has been supplied. Do not infer an endorsement, pricing or permission to distribute under MIT/Apache. Keep original AGPL source attributions and dependency license notices for translated code/tests/guides. Publish with different product branding when release is authorized; do not replace legacy file markers just for branding.

The architecture/source findings here were informed by AGPL implementation review. A license negotiation could authorize reuse under other terms, but until then code cannot be assumed eligible for SolidCraft. Do not send this as a ready-to-merge Tangency implementation plan or import copyleft snippets into its repos. A future clean-room track needs separately established provenance and project approval; changing crate names does not create that separation.

## Handoff gaps still to close

The next session must capture full source and runtime catalogs, feature schemas/variants, complete import graph, actual backend package implementation/build versions, archived project fixtures, UI/device coverage and performance data. This packet gives concrete tasks for those unknowns rather than substituting guesses. No migration percentage or delivery date is justified yet.
