# Image candidate trial, not application adoption — 2026-11-02

Owner review: Logan; unit io.obj.companions.image_audit, PR #39. Original behavior
remains authoritative. The source picker includes PNG/JPEG/BMP/WebP; reducing the
scope to PNG/JPEG would lose existing supported file paths.

Decision: test exactly `image = =0.25.10` with default features disabled and only
png/jpeg/bmp/webp. Resolve its complete locked graph, inspect licensing and
Rust/build/FFI paths, compile native/Wasm and compare raw decoded pixels against
the already captured full original bitmap/canvas path. This decision permits a
candidate-only temporary audit project. It does **not** add a dependency to the
application workspace or approve decoder compatibility.

The candidate provides these four formats through one narrow API; its defaults
include additional formats and threading, so defaults are inappropriate here.
Primary references: [crate docs](https://docs.rs/image/0.25.10/image/),
[pinned manifest](https://docs.rs/crate/image/0.25.10/source/Cargo.toml).
The exact lock, selected features, checksums, actual license-file hashes and
build-script/FFI observations belong to the audit evidence, not this assumption.

A pure-Rust/licensing/build result is distinct from pixel compatibility. Alpha
rounding, 16-bit conversion, JPEG/chroma/orientation, profile/gamma handling,
malformed omission and resource behavior must retain actual source results.
Every difference stays visible; no tolerance, lookup-answer injection or silent
pixel normalization. The source's canvas continuation cannot be omitted just
because a library returns decoded RGBA. A failed compatibility trial requires a
bounded follow-up or a different candidate, not acceptance of changed output.

No C/C++ codec enters the delivered product. Pillow/native browser codecs are
reference/fixture tools only. No CLA, naming/licensing-policy change, release or
external contact is part of this decision. Final audit verdict and unresolved
scope will be recorded after measurements; image implementation remains planned.
