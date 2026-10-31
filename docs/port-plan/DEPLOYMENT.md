# Deployment implications of the Rust port

Investigated 2026-10-31. This extends the port plan; no site, hosting account or deployment configuration was changed. Build/deployment recipes below are proposed until an actual Rust build passes the deployment tests.

## Main conclusion

Rust is an implementation language; web versus desktop is a distribution/runtime choice. The required pure-Rust implementation can target both native executables and browser WebAssembly. A Rust browser build can retain the current static-hosted, local-computation model. A Rust port does not require a cloud geometry server, user uploads, containers or a database.

Recommendation: one Rust core/engine with native and browser adapters. Preserve a static browser app for easy access and provide native app/CLI/MCP packages for desktop use. Treat hosted geometry computation as a separate future product decision, outside the current scope.

## What the existing deployment actually does

At the frozen ScanRuler commit, `wrangler.jsonc` serves `./dist` as Cloudflare static assets with SPA fallback; there is no custom Worker script. Vite creates the files. Geometry runs in the user's browser, with downloaded Wasm dependencies, rather than on Cloudflare. The documented R2 file store supports optional public short links; it is not the geometry engine or mandatory private project storage.

Those observations come from the pinned repo, not an audit of scanruler.com's live account settings. Do not reuse the author's Cloudflare resource names, domain or file bucket. A fork needs its own branding and deployment resources when publication is authorized.

## Deployment options

| Target | Build/distribution | Where geometry runs | New work |
| --- | --- | --- | --- |
| Rust browser app | Cargo/wasm-bindgen/bundler produces HTML, Wasm, worker/loader files and assets; host static output | User's browser | Wasm size/startup, browser adapters, asset paths/MIME/cache, GPU feature detection, memory and optional threading headers |
| Rust native desktop | Per-platform executable/app package with assets and notices | User's machine | Windows/macOS/Linux CI, platform libraries, signing/notarization where applicable, installers, updates, dialogs/storage/input |
| Rust CLI/MCP | Native executable per supported platform; can share engine/library with desktop | User's machine | Packaging/discovery, stdio protocol, file access and optional connection to running app |
| Local browser UI served by native helper | Rust helper serves bundled static app over localhost, if this mode proves useful | Browser, or native engine through an explicit local bridge | Origin/permission/pairing tests and version-matched assets; separate architecture choice, not assumed necessary |
| Hosted Rust compute API | Browser uploads work to a managed service | Server | Authentication, uploads/storage/retention, resource isolation, queues and operating cost; not required by the port and not currently planned |

The official eframe template demonstrates native/web targets and publishing Trunk's output as a static site. This establishes feasibility of the distribution pattern, not that the unbuilt port already works in browsers. Trunk is a candidate, not a committed version/tool selection.

## What changes for the browser host

1. **Build pipeline changes, hosting can stay.** Replace the Vite application build with pinned Rust/Wasm build tooling. A proposed web command is `trunk build --release`; the final workspace/public URL configuration must be tested. CI may build elsewhere and upload static artifacts, so the serving host need not contain Rust compilers. Cloudflare is optional; choose hosting based on required headers/assets rather than language.
2. **Serve a coherent asset version.** HTML, loader, workers, Wasm, fonts and shaders must come from one build. Serve Wasm with the appropriate content type; verify every request. Hash immutable assets, revalidate entry HTML, retain or correctly retire old-version assets, and test an already-open tab during update and rollback. Existing SPA fallback can turn a missing Wasm/worker URL into HTML; detect this and give a useful update/reload error rather than an unexplained instantiation failure.
3. **Threads change headers, not compute location.** Start with the planned single-worker mode. Shared-memory Wasm threading requires cross-origin isolation and supported browser behavior. Typical deployment uses COOP `same-origin` plus COEP `require-corp` (or a tested alternative), and compatible resource/CORS policy. Cloudflare static assets support `_headers`. Test actual `crossOriginIsolated` state, workers and external file/asset loads. Do not enable headers blindly: they affect embedding, popups and cross-origin resources. A nonthreaded fallback stays useful.
4. **Browser limits remain.** Rust/Wasm does not automatically remove browser memory, GPU, device-permission or file-access restrictions. Measure real peak memory and large-file behavior; do not promise native-sized datasets in every browser. No assumed performance or binary-size win from the language alone. Geometry jobs stay off the UI thread.
5. **Offline is explicit.** Once loaded, local computations need no geometry server, but offline reload/install requires cached application assets and tested storage behavior. A PWA/service worker is optional planned work, not an automatic Rust feature. Version cache entries and test safe upgrades without losing projects. Browser storage is origin-specific; changing domain requires deliberate project export/import or another approved migration.
6. **Privacy remains a runtime choice.** For the static local-processing target, scans stay local unless the user selects remote loading/sharing or an agent integration sends data to its provider. Rust itself neither uploads data nor guarantees privacy; test network behavior. Do not introduce a backend or account requirement merely to deploy Wasm.

## Native distribution implications

Native builds can use ordinary filesystem APIs, native threads and OS integrations. They may handle larger workloads more comfortably, but quantify that on target hardware. They require separate per-OS/architecture artifacts and install/update testing. Signing/notarization and release-channel choices are packaging decisions, not kernel requirements. Offline local operation should not depend on a website being reachable; bundle required data/assets and make remote integrations explicit.

The native Rust MCP helper can replace the current Node package. A web page does not become a local MCP process just because its engine is Rust: browser restrictions still apply. The static browser version may still need the separately installed Rust helper and pairing/permissions for agent access. A native CLI can instead expose stdio directly; bridging to a visible desktop session remains an explicit supported mode. A Cloudflare-hosted API must never be mistaken for automatic access to the user's local files or devices.

## Release acceptance additions (P26/P27/P35)

- Produce separately versioned native packages and a static browser artifact from the same source revision and engine contracts. Record kernel/features supported in each; different capability sets prevent full cross-target parity claims.
- Test production static hosting, root and subpath asset URLs, Wasm MIME, worker/font/shader requests, compression/cache behavior, stale tabs, atomic updates and rollback. No remote geometry process should be required by the static target.
- Test no-isolation single-worker mode and, if shipped, isolated threaded mode. Verify cross-origin model links, browser pairing and GPU capability/fallback behavior on the real served origin.
- Record cold download/compile/startup separately from warm operation timing. Test offline claims at reload as well as after startup. Preserve project data through updates.
- Audit native and browser artifacts separately for hidden foreign geometry Wasm/native libraries or subprocesses. Both must meet the confirmed pure-Rust endpoint; browser packaging does not excuse retained C++ kernels.
- Verify installers/portable binaries, native device/dialog behavior, CLI/MCP setup and update compatibility on supported systems. No publishing occurred during this investigation.

## Sources

- Frozen upstream [Cloudflare configuration](https://github.com/flyboy-byte/ScanRuler/blob/2cd89c6769a10c27ee7754b477ada521c9ef5cb7/wrangler.jsonc) and [development/deployment documentation](https://github.com/flyboy-byte/ScanRuler/blob/2cd89c6769a10c27ee7754b477ada521c9ef5cb7/docs/development.md).
- Official [eframe template](https://github.com/emilk/eframe_template): native/web compilation and static web deployment pattern.
- Official [Trunk configuration](https://trunk-rs.github.io/trunk/guide/configuration/index.html): output paths, public URL, asset hashes and tool pinning.
- Official [Cloudflare static-asset headers](https://developers.cloudflare.com/workers/static-assets/headers/): header configuration for static responses.
- [MDN cross-origin isolation](https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated): conditions for shared-memory browser APIs.

External documentation was checked on the investigation date; implementation must pin actual tool versions and verify their behavior. Architecture recommendations and release tests above are planning conclusions, not observed results for a Rust port.
