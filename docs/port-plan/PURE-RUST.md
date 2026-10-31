# Required pure-Rust endpoint

User clarification: “Pure Rust throughout, including geometry kernels end goal pure rust, per tangency and as a sidequest for me.” This is the required completion target. The former external-kernel-first default is superseded. Intermediate previews are allowed; retained kernels never qualify as finished work.

## Meaning of pure Rust

Application, numerical algorithms, CAD kernel, mesh algorithms, import/export implementations, UI and MCP product logic are Rust. C/C++ geometry behind Rust bindings or compiled to Wasm still fails the endpoint. No OCCT, Manifold, meshoptimizer, meshstep implementation or Node geometry sidecar may remain in the delivered product. Evaluate all transitive dependencies, not only root Cargo manifests. Audit numerical and codec libraries too; a permissive license does not prove a dependency is implemented in Rust.

Ordinary OS/GPU/browser interfaces and minimal browser bootstrap/binding glue are platform boundaries. They must not conceal modelling, fitting, parsing or mesh algorithms. Rust's compiler/toolchain and the operating system are not being rewritten. Data, documentation, assets and CI configuration remain their appropriate formats. Active repository build/test/release tools must also meet the whole-port tooling goal; frozen original JS code is permitted solely as a reference oracle.

## Geometry work starts early

Do not spend a milestone building a production native OCCT shim. Keep the original application available as a comparison oracle; add a temporary reference bridge only when needed for a specific comparison. It is disposable tooling, not the target backend.

SolidCore is the first exact-kernel candidate because it fits Tangency's Rust direction. Assess its native and wasm32 behavior on the required operations rather than assuming its current API is ready. Record a capability table per operation: supported and verified, partial, missing, numerically incorrect, timeout, or unknown. Missing behavior creates a scoped Rust kernel task. No silently skipped features, approximate mesh substitute for an exact operation, or fallback to OCCT in a release labelled complete.

The project's native/web engine and format contracts must remain independent of any one kernel's raw handles. Stable modelling references, tolerances and project compatibility remain product responsibilities even when a kernel operation returns a shape.

## Required workstreams

| Workstream | Scope | Gate |
| --- | --- | --- |
| P31: exact-kernel foundations | Analytic/NURBS geometry, evaluation/projection/intersections needed by initial features, topology, profiles, primitive/extrude/revolve, query and handle ownership | Independent geometry checks and native/Wasm behavior on the same fixtures |
| P32: robust modelling operations | Boolean union/cut/intersection, fillet/chamfer, shell/offset/draft, sweep/loft, healing/stitching and operation history for persistent references | Feature capability corpus, edit/rebuild, topology/volume checks and bounded failure behavior |
| P33: STEP and tessellation | Pure-Rust STEP read/write, trimmed/periodic surfaces, names/units, seams, tessellation and error reporting | Independent round trips, reference mesh error budgets and unsound/unsupported diagnostics |
| P34: mesh operations | Triangle-mesh booleans, simplification/reduction, repair/fill/offset/shell and spatial queries where needed | Known geometry and adversarial scan corpus; bidirectional error and manifold/closedness checks |
| P35: dependency purity audit | Runtime and shipped artifacts, transitive code, Wasm imports/assets, subprocesses and active tooling | No hidden non-Rust numerical/geometry implementation or original TS/Node product runtime |

These streams are on the dependency graph before final application acceptance. They are not an optional sequel after the app is considered finished. P29 is the final pure-Rust release audit, dependent on the completed kernel/mesh/import streams and whole-application validation.

## Progress that is useful for a personal side project

Keep increments independently runnable: pure-Rust scan import and fitting; alignment/deviation; native/web measurement; exact extrusion with round-trip STEP; constrained scan-derived sketches; robust solid operations; freeform patches; complete mesh tools. A missing advanced kernel operation should not block improving the measurement engine, but it does block declaring full feature parity.

Prefer reusable Rust modules with native/Wasm tests over custom integration with today's SolidCraft internals. SolidCore improvements identified here can be described as precise geometry gaps. Any upstream contribution must comply with that project's rules and source-provenance/licensing requirements. The existing ScanRuler source review remains source-informed; the pure-Rust language goal does not itself change licensing or make a later Tangency merge automatic.

No delivery date is set. Track completed capabilities and actual effort per increment. If robust blends or trimmed STEP take substantially longer than the UI, preserve that evidence and continue independent work without weakening the final requirement.
