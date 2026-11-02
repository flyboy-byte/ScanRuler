# Image dependency/pixel audit contract — 2026-11-02

Unit io.obj.companions.image_audit. Candidate-only image=0.25.10, default features
off, PNG/JPEG/BMP/WebP selected; freeze the exact complete Cargo lock/feature/license
closure and inspect reached Rust/native-build/FFI paths. Application Cargo files
stay unchanged. Build the candidate in a temporary project natively and for Wasm.
This is an audit/spike, not a port of readCompanions or decoder acceptance.

Source uses the complete pinned scanFiles.ts/readCompanions/decodeImage in actual
Chrome, with bitmap colorSpaceConversion:none and premultiplyAlpha:none followed
by original canvas drawing/getImageData. Independent authored PNG/JPEG/BMP/WebP
inputs cover alpha, opaque/color modes, malformed/truncated files and seeds
1/7/99/1039. Inputs and source-only capture are committed before candidate pixel
comparison. Pillow is fixture-authoring reference tooling only, never a product
codec; preserve its version and encoded input hashes. No source output enters Rust.

Compare actual decoded presence, dimensions and every RGBA byte, with no tolerance
or normalization. Record candidate error messages separately: paired omission is
not successful pixel parity, and resource limits are host safeguards. Keep every
mismatch, full source/candidate bytes, fixed ID/seed and source/toolchain identity.
A candidate can pass license/purity/build review yet fail faithful browser pixels;
record that verdict rather than blessing or silently correcting its outputs.

Before any application adoption, the images unit still needs a faithful complete
Rust continuation, actual Wasm runtime comparisons, bounded decode/pixel/work,
filename/text/omission order and public integration. Codec-platform gaps belong
in the decision/finding and acceptance backlog, not a broader tolerance.
