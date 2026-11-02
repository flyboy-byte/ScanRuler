# First binary STL decoder port — 2026-11-01

Unit `io.stl.binary`, parent P06. Tested implementation commit: `5ee3881d887a4720c6a7cf5ca7a675333e6e8e62`, fork base `80c2b388d0b58370ffdddd613d12f3c75e9fe83c`. Reference: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`, `src/core/parsers/stl.ts::parseBinary`.

## Implemented operation

`port_io::stl::read_binary_triangles` decodes the complete helper body with an explicit count, owned f32 positions, original coordinate order and progress cadence/rounding. Normals and attribute values are ignored. Count/header/ASCII dispatch is deliberately outside the helper contract, not silently dropped from a claimed full loader. The format dispatcher continuation is `io.stl.dispatch`.

The Rust API adds an explicit caller triangle budget, checked sizes and fallible allocation. Short coordinate input returns `Truncated` rather than a JS DataView `RangeError`; no partial mesh escapes. Those are documented host boundary errors. No filtering, unit conversion or geometry changes are introduced.

## Independent comparison

`oracle/binary-stl.cjs` reads the source from the frozen Git object, extracts the helper body and removes only its TypeScript signature. It runs that body in Node, with independently supplied input bytes. Rust runs before the original; original outputs are never used as Rust inputs. Comparator `io.stl.binary.bits.v1` compares the full coordinate byte arrays and UTF-8 progress text exactly, including lengths; no numerical tolerance or normalization.

Ten successful cases: `empty`, `one`, `seed_7`, `seed_99`, `progress`, `progress_twice`, `ieee754`, `trailer`, `missing_final_attributes`, `empty_no_header`. Eight have nonempty coordinate outputs. Seeds and per-case SHA-256 input/output identities are in `2026-11-01-binary-stl-cases.json`. Two progress cases cross the callback boundary, including both 50% and 100% messages. The largest has 524,290 triangles / 18,874,440 coordinate bytes. IEEE coverage includes signed zero, infinities, subnormals, quiet and signaling NaNs.

`truncated` compares Rust's safe error against the original RangeError; it is not counted as successful value parity. Ordinary Rust tests also exercise overflow and caller budget failures before progress.

Raw inputs and both outputs are retained locally under `rust/target/stl-diff/run-1793560095513643890/`. Subsequent runs use new directories, preserving earlier evidence. CI uploads `binary-stl-comparison` (seven-day retention), including raw bytes and identities even on comparison failure. Regenerate with the commands below after artifact expiry; checked-in hashes identify this local run, not a replacement for executing CI.

## Validation and limits

On system Rust/cargo 1.99.1, Linux 7.2.11-zen1-1-zen, local Node 20.20.2:

```bash
cd rust
cargo xtask ci
cargo test -p port-io --test binary_stl pinned_original_comparison -- --ignored --nocapture
```

Both exit 0 at the tested implementation commit. Fast gate: fmt, clippy, six ordinary workspace tests, plan validation/self-tests and wasm32 library check. The live oracle test is explicitly ignored in the ordinary gate and then run separately. CI provisions Node 22 for the separate comparison; that hosted result is not claimed here. No new Rust dependencies.

Not accepted: browser-Wasm runtime comparison, generic P36 harness/ratchet, G0 catalogs and shared P05 contracts remain open. There is no user-facing import command/session integration yet. No full P06, G2 or whole-app parity claim.

## Preserved finding / next action

The initial Rust bounds check incorrectly rejected a 133-byte one-triangle input. The original never reads the final two attribute bytes, so even 132 bytes succeed. Its zero-count helper also succeeds without reading any header. Both are now named regression cases; the genuinely truncated case ends at 131 bytes. No source expectation was weakened.

Next: implement `io.stl.dispatch` with original binary-before-ASCII precedence and precise ASCII numeric semantics, then mesh welding/indexing and the fixed-selection plane slice. Complete P00–P03 and browser-Wasm harness evidence before promoting acceptance.
