# Private Wasm probe may call the existing owned port-io crate

The old test probe depended only on `port-core`. PLY runtime comparison needs the
already implemented Rust parser, so add a workspace path dependency on `port-io`
to the private `port-wasm-probe` test crate. This is one edge between existing
owned crates, not a new third-party package. Cargo.lock adds only that edge.

Do not duplicate the parser, interpret geometry in the JavaScript launcher, or
bring the original TypeScript into the shipped Wasm module. Probe bounds remain
1 MiB input/output and 64 MiB linear memory; the module must import nothing. The
new raw-PLY codec distinguishes source refusals from host allocation/work limits.
Browser input is raw authored fixture bytes only, never reference outputs.

This is test infrastructure, not production ABI/session design or P35 whole-app
purity acceptance. Existing Rust licensing and unsafe-code rules apply. Evidence
belongs to `harness.wasm.ply`; every older browser comparison remains mandatory.
