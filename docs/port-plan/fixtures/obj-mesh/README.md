# Fixed complete OBJ parser inputs

Authored/generated under AGPL-3.0-only. All 81 inputs and their source observations
precede Rust implementation: 53 complete outputs / 28 source refusals. Regenerate
with `python3 docs/port-plan/tools/generate_obj_mesh_cases.py`. Seeds remain
1/7/99/1039. No expected output comes from Rust.

Input packet: OBJ byte length (u32 LE), raw OBJ bytes, optional-companion flag,
then length-prefixed material names/text and decoded image names/u32 dimensions/
RGBA bytes. The original adapter executes complete pinned obj.ts and objTexture.ts
bodies with only annotations erased. Rust receives independent input, not any
original intermediate state.

Compare complete OBJ1 output: indexed kind, status, every float32 coordinate bit,
fan index, optional RGB byte and absent/missing/applied texture note, plus full
progress and exact source error text. Malformed geometry accepted by the source
parser (forward/out-of-range indices, narrowing overflow) remains observable;
downstream graph validation is separate.

Includes negative/forward/maximum sparse indices, strict decimal/safe-index rules,
Number rather than parseFloat grammar, radix/large integer conversion, signed
zero, UTF-8 replacement/BOM/whitespace/comment behavior, ignored normals/groups,
first-valid-textured-corner material binding, no retroactive UV binding, complete
decoded-image colors and notes and source refusal order. Huge positive sparse
indices do not allocate a correspondingly huge dense metadata buffer in Rust.

Both progress inputs execute their **real 1,048,576 line index**. One unknown
record emits the percentage; the other is a two-UTF16-unit emoji, which takes
the source's short-line continue before progress. Neither is reduced or replaced
by an injected callback. File/image decoding, public engine integration and actual
browser-Wasm runtime remain separate; parent io.obj stays planned.
