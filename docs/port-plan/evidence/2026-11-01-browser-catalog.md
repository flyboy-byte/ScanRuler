# Actual original browser catalogs — 2026-11-01

Hosted run: https://github.com/flyboy-byte/ScanRuler/actions/runs/41356099428. Candidate CI merge: 8f327ee96cebe12d0a5113daadb62f9ee8f3cf59; frozen original: 2cd89c6769a10c27ee7754b477ada521c9ef5cb7. Source launcher revision: 503e8554329c82f55826de456a3aadf36d8b7be0.

Actual Node 22/Chrome 156.0.8078.47 original Vite pages paired to the frozen original MCP server using its SDK and E2E helpers. No source edits or manual registration; clean pin, app version and lock hashes checked before/after. Fresh synthetic browser profile for each context; no user scan. Both original console/page-error checks pass, viewport is true and scan is null.

Core-only: 72 registered, 6 headless flags, 66 page-only flags, no plugins. All: 140 registered, 36 headless flags, 104 page-only flags; mesh-tools and reverse. Complete metadata hashes match the independent headless snapshots: af1cf3d58378bebcf46d75c36ee9043a5d9c2b63cee697e24cd3d3f7e4248f13 and 19a2cd59faded4b4b6f0447d2aaeede556d410acaa62ab607b30c06eab22a2aa respectively. Browser lists all commands; these flags describe capability rather than executed scan operations.

Full gzip JSON, environment/source identities and SHA256 manifest: browser-2026-11-01/. Reproduce with oracle/BROWSER-CATALOG.md; immutable metadata comparison is added to hosted recaptures. Initial readiness attempts 41354991898 and 41355455158 failed; source inspection showed core registration is lazy on agent bridge connection. Actual pairing resolved it without changing readiness thresholds or bypassing registration.

Bounded baseline-captured P02 child only. Production bundle, feature variants, wider agent operation coverage, P00/P01 prerequisites, Rust commands, browser-Wasm parity and general P36 remain open. No Rust/application acceptance claim.
