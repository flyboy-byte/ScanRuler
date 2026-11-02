# Fixed OBJ texture/helper inputs

Authored/generated under AGPL-3.0-only; no private scan/image data. The 144 inputs
were committed and independently source-captured before Rust implementation.
Generate with `python3 docs/port-plan/tools/generate_obj_texture_cases.py`.

Cases cover the original five functions: baseName, parseMtl, mtllibNames,
sampleTexture and bakeObjTexture. Include Unicode/contextual lowercase and both
path separators; MTL options/partial arguments/duplicate replacement order;
library splitting; texel centers, U wrapping/V row clamping, near-half rounding,
alpha independence, nonfinite coordinates, zero/short images; complete missing,
applied and absent notes, precedence/fallback/duplicates, first-seen image order,
untextured 180 colors and absent UV/material entries. Seeds 1/7/99/1039 drive both
sampler and complete bake inputs. No expected result is generated from Rust.

The binary test input's first byte selects one helper group; strings and arrays
are length-prefixed LE, UVs are f64 LE and RGBA bytes are authored directly.
OTX1 output compares exact strings, ordered maps/notes and every color/output
byte, including untouched bytes outside a sampler write. No numerical tolerance,
reordered filenames or color normalization. Image dimensions are u32; actual
decoder/file-name interoperability remains in the companion-ingress unit.

Source adapter runs the entire frozen objTexture.ts module after annotation
erasure. Source outputs never enter Rust. Native/Wasm compilation does not count
as browser runtime; parser, file bundling and PNG/JPEG decoding are separate
continuations. Parent io.obj stays planned.
