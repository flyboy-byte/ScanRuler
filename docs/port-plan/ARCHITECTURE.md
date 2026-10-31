# Runtime and compatibility architecture

Status: proposed contracts based on selective source review. No Rust implementation or kernel binding has been built. Source details refer to commit `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`.

## Keep the first workspace small

Start with `port-core` (types, buffers, transforms, geometry/fitting modules), `port-engine` (session, commands/jobs/history), `port-io` (formats/project compatibility) and a small CLI application. Keep browser adapters in a separate package/module. Split fitting, image/2D, reconstruction, rendering and external backends into crates when dependency separation or build times justify it. Do not create fifteen empty crates before a tested slice exists.

Dependencies point inward: UI/MCP/platform I/O → engine → core. Pure core has no DOM, egui, Three.js, filesystem or mutable process singleton. Format parsers return owned data; they do not mutate active sessions. Kernel adapters depend on neutral geometry requests, not the UI. A plugin contributes commands, project/history sections, scene layers and capabilities through Rust interfaces; disabling it must preserve unowned project data.

Use a single mutable session owner with serialized command commits and immutable revision snapshots for reads. A job computes against a captured revision and asset generation. Its result commits only if that precondition still holds. This allows Rust-native multiple sessions even though the original headless code uses one engine per process. Multi-session support is an implementation improvement, not required wire compatibility in the first preview.

## Data and numerical fidelity

The source stores meshes/normals as Float32Array and indices/adjacency as Uint32Array; poses are row-major f64 rotations/translations. Source `rigidApplyToPoints` bakes transforms into f32 points. Preserve these semantics for initial differential tests. Use f64 in solvers and accumulated poses; do not silently change the source point representation to f64 and call numerical differences a regression fix. Higher-precision source storage is a separately versioned improvement with its own memory and compatibility evidence.

Proposed types: Millimetres, Radians/Degrees at explicit API boundaries, Pixel coordinates, ScanFrame/ReferenceFrame/SheetFrame, RigidTransform, ScanId, MeshRevision, ElementId, FeatureId and JobId. Do not confuse an element ID with its displayed number or a kernel handle. Validate JSON-safe integer IDs, overflow and counter restoration; JS number semantics differ from Rust integer arithmetic. Arrays crossing the wire carry component count, scalar type, byte order and length; dimensions must be checked before allocation.

Use stable source vertex IDs for selections. Import welding, seam splitting, normal generation and mesh edits can renumber vertices. Carry explicit maps for each transition and preserve their invalid entries; brushes and saved regions must map to the geometry that was measured. Source/render meshes are distinct objects even when their positions match. Do not derive measurement vertex IDs from render triangle order.

## Ownership and memory

Rust owns measurement buffers once an operation is cut over. Browser JS keeps handles plus render buffers, not a second authoritative mesh. Transfer immutable buffers once when possible. Wasm views cannot be held across operations that grow memory; reacquire views or copy documented render exports. Avoid aliasing Rust mutable storage with a JS view while Rust may write. Use generational asset handles so a freed buffer cannot be read as another scan.

A starting memory model for one graph is approximately `28V + 12T + 4A` bytes for f32 positions/normals, u32 CSR offsets/neighbor entries and u32 triangles, where A is the number of adjacency entries. Add each per-vertex f32 field (4V), RGB field (3V), BVH, textures, original files, render seam copies, worker transfers and kernel heaps. This is an accounting template, not a measured ceiling. Include preview/undo/rebuild peaks, not just the idle graph. Do not retain an entire mesh clone per command for history; share immutable assets and record recipes/reference deltas.

Native can use a bounded pool; browser first ships a single worker without requiring SharedArrayBuffer. Threaded Wasm is optional and requires tested cross-origin isolation/deployment headers. Resource caps, cancellation checkpoints and progress behavior are platform adapters over the same algorithms. No assumption that Rayon works unchanged in all browser deployments.

## Commands, history and background work

Keep existing command names/input defaults/output keys/errors where compatibility is promised. Source has twelve named error codes, mutation serialization, a separate scene queue, interrupting stop commands, deferred rebuild settling and recent job outcomes. Its default command timeout is 10 minutes and explicitly permits partial change. Preserve the externally observable status contract; do not infer stronger rollback guarantees from AbortSignal alone.

Preserve the original observable transaction/history/timeout contract during the port. Prepare/compute/validate/commit is an internal structure, not permission to change partial-timeout or retry behavior. Stronger atomicity is a possible separate improvement only after explicit reviewed approval and comparison/migration evidence. Diagnose late-job behavior against the pinned reference; document any intentional exception rather than silently changing it. A backend that cannot be cancelled cooperatively needs a bounded worker/process strategy with the same externally declared outcome.

Define distinct lanes: mutation/settle, scene capture, immutable reads, and immediate cancellation. Long reads capture the relevant revision; a scene screenshot captures both the intended camera and scene revision and restores temporary view state. Batch semantics must be recorded per operation: grouped history, deferred build, duplicate-name preflight and partial failure are not equivalent to one all-or-nothing transaction.

Temporary drafts/previews are not persistent model state. Closing/committing/discarding a draft, making a measurement and navigating undo have different history rules. Implement scoped history participants for measurement, flat sheets, reconstruction and edited meshes; restore atomically and rebuild the necessary caches. Preserve the distinction between an original scan, edits awaiting adoption, adopted edited scan, reduced copy and trimmed replacement piece.

## Pure-Rust geometry backend: required

The user confirmed pure Rust throughout, including geometry kernels. `PURE-RUST.md` is authoritative. SolidCore is the first exact-kernel candidate, subject to per-operation evidence. No assumption of OCCT equivalence is made. P31/P32 cover foundational and advanced exact geometry; P33 covers pure-Rust STEP/tessellation; P34 covers mesh operations; P35 audits dependencies/artifacts.

Retain the original application for differential comparison. Its `occt-wasm`, `manifold-3d`, `meshoptimizer` and `meshstep` backends may be used as reference tools, never as the finished Rust implementation. Avoid investing in a production C/C++ native shim or shipping a Node geometry sidecar. A temporary bridge is justified only for a concrete oracle/transition experiment and stays labelled disposable.

The Rust backend protocol carries version, capabilities, request ID and session epoch. Geometry handles are generation-checked and never persisted as raw pointers or session hashes. Preserve face/edge history and stable reference resolution across feature edits. Backend cache keys include tolerances, implementation version and feature/input revision. Native and Wasm run the same Rust kernel modules under platform-specific execution adapters.

G1 records verified/missing capabilities for profile/extrude/revolve/Boolean/history/tessellate/STEP/release plus cancellation and memory behavior. Missing capabilities become Rust work items. The measurement engine may proceed independently, but final modelling/format parity requires them. Unsupported intermediate cases return explicit errors; they may not silently fall back to non-Rust code or approximate exact solids with a mesh.

Dependency review inspects transitive implementation and shipped assets: a Rust crate can still wrap C/C++, bundle a foreign Wasm module or call a subprocess. Numerical and codec dependencies are included in this audit. OS/GPU/browser bindings are platform interfaces; hiding geometry logic in their glue fails the endpoint. Completion requires capability evidence as well as an all-Rust dependency graph.

## Projects and schemas

Keep a compatibility reader/writer for legacy `.scanruler` schema 1, app marker `ScanRuler`, row-major pose data, optional legacy defaults and all plugin sections. Display/product branding is independent from an internal file-format identifier. New branding must not cause old files to be rejected. Persisted reverse data is version 3; version-1 circular patterns and pre-version-3 mirror body references require actual migrations, not only serde defaults.

The archive saves original files and recipes; derived fields/BVHs/meshes are generally rebuilt. Imported reconstruction STEP files are packed into a plugin member. Notes and plans are separate sections. Preserve unknown plugin JSON and unowned archive members byte-for-byte when possible, including a load/save cycle with that plugin disabled. A scan replacement follows documented invalidation rules; it must not accidentally keep unrelated plugin state.

Source archive limits are 16 members, 64 MiB manifest and 1 GiB total expanded bytes. Reproduce valid baseline archives; enforce bounded actual decompression as well as declared sizes. Reject duplicate/invalid members, missing files and hostile numeric values. Native saving uses a prepared temporary file and atomic replacement where supported; browser download/OPFS behavior needs an adapter. Do not overwrite a user's only legacy project as part of a migration.

## MCP and rendering adapters

MCP command tools replace dots with underscores; file inputs/outputs are adapted to paths and binary payloads; image outputs are treated specially. The page link uses a JSON announcement followed immediately by one binary WebSocket frame, not base64 JSON. Preserve pairing, version negotiation, relay/shared-server behavior, jobs/progress/late results, traces and path handling before replacing the original helper. Tests must cover the old helper against the Rust page and the Rust helper against the original page, where the legacy protocol is claimed supported.

The Rust UI targets egui/wgpu provisionally because that aligns with Tangency's current stack. Validate picking/overlays, depth/clipping, sRGB field colors, camera/navigation schemes, 2D calibrated text and export before committing to that choice. Screen visual parity is judged by workflow and correct data, not exact React CSS reproduction. Browser clipboard/download/persistence/device permission APIs and native dialogs/SpaceMouse need platform-specific adapters. No browser feature may quietly disappear because desktop egui is easier.
