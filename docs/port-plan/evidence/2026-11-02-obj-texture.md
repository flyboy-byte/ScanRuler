# Complete OBJ texture/material helpers — 2026-11-02

Unit `io.obj.texture`, implementation `219059c2c719879bfce24fb75cfe69cdf3ef6abb`.
Reference `2cd89c6769a10c27ee7754b477ada521c9ef5cb7::src/core/parsers/objTexture.ts`.

All five helper bodies are present: baseName, parseMtl, mtllibNames,
sampleTexture and bakeObjTexture. **144 complete original/native helper-state
comparisons pass**, with seeds 1/7/99/1039. Fixed inputs and original observations
were committed/captured before Rust implementation. Every string, ordered map,
note, color byte and complete sampler buffer is exact, including untouched output
bytes. A changed sampled color bit and missing note data are rejected. There is
no numerical tolerance, color normalization or source-result input to Rust.

Retains filename matching and original names in notes; MTL option grammar and
replacement/insertion order; normalized duplicate file last-value precedence;
material-map first definition precedence across libraries; fallback single/loose
images; missing/applied/absent distinctions; first-used image ordering; untextured
180 bytes; U wrapping, V row clamping, alpha independence and source bilinear
operation order/rounding. Undefined channels/nonfinite samples propagate to the
original Uint8 zero conversion, not a fabricated valid pixel.

Inputs use already decoded RGBA, valid UTF-8 names and u32 dimensions. Zero/short
rasters and nonfinite direct-helper UVs are compared too. This does not implement
image-file decoding or claim arbitrary JS UTF-16 filename/non-u32 raster-object
interoperability. Those are companion/platform acceptance boundaries. Unicode
context/combining cases pass on recorded runtimes; exhaustive Unicode case-table
version equivalence remains open.

Large buffers reserve fallibly. Caller byte/name/file/material/vertex/work caps
are explicit host refusals, with no silent truncation or partial output on a
sampler bound error. Ordered lookups charge byte visits to bound adversarial work;
large-file/performance acceptance remains open. Bounded Unicode lowercase uses
the standard Rust implementation; no additional dependency or foreign code.

Adjacent JSON retains all input/output hashes, complete original/Rust output hex,
compiler/Node identity and exact implementation SHA. The original adapter executes
the entire frozen helper module with annotation erasure only. No PLY implementation
file, source app, seed, tolerance or previous comparison is changed.

Native ordinary host-guard tests and Wasm compilation pass. Actual browser-Wasm
helper execution, full OBJ parser, file-set/image ingress and public engine
commands remain open. The parent `io.obj` remains planned; no numerical unit is
accepted and this is not a claim that textured OBJ import already works in Rust.
