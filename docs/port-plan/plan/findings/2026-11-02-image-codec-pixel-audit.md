# Image candidate differs from original browser pixels — 2026-11-02

Unit io.obj.companions.image_audit, PR #39. Pinned reference 2cd89c6; the complete
original readCompanions/decodeImage executes in Chrome 156.0.8078.79. Candidate
image 0.25.10, defaults off, png/jpeg/bmp/webp, exact lock and 22 packages. Source
capture preceded the candidate spike. Independent Pillow-authored inputs carry
stable hashes/seeds 1/7/99/1039; no source result enters Rust.

| Evidence | Observation | Strength |
| --- | --- | --- |
| `2026-11-02-image-source.json` |56 original observations: 36 decoded images and 20 omissions; full pixels retained | OBSERVED |
| `2026-11-02-image-candidate.json` |8 exact decoded images, 28 pixel mismatches, 18 agreed omissions, 2 presence mismatches; no host refusals/crashes | OBSERVED |
| PNG alpha atlas, RGBA, LA, palette; WebP alpha | Original and raw candidate colors differ; fully transparent hidden RGB is erased by the source path | OBSERVED |
| Canvas alpha quantization as general explanation | Consistent with the RGBA/LA/WebP pattern; exact conversion has not been independently derived | INFERRED |
| `png-gray16` | Source channel 4 vs candidate 5 at byte 16; 159 differing bytes, maximum delta1 | OBSERVED |
| JPEG fixtures | All 11 successfully decoded JPEG cases differ; seeded maximum channel deltas reach 11 | OBSERVED |
| JPEG rounding/chroma/orientation cause | Exact mechanism not established; an alpha-only correction cannot address these opaque JPEG differences | UNKNOWN / INFERRED |
| `jpeg-truncated-last`, `webp-truncated-last` | Original omits each; candidate decodes each | OBSERVED |
| BMP | All 6 successful BMP cases match exactly; malformed BMP omissions also agree | OBSERVED, bounded corpus only |

No single tolerance or blanket “standard-compliant decoder” argument resolves
these byte-observable differences. The largest alpha-atlas difference is255,
including hidden RGB; the source is authoritative even when a candidate preserves
more encoded color information. Source/candidate complete bytes and first/count/
maximum differences are retained, never normalized. Agreed omissions are negative
observations, not successful pixel comparisons.

Dependency/build result is separate: all 22 declared license expressions offer an
option in CONTRIBUTING.md's allowlist, and actual shipped license files are
hashed. No foreign implementation files are present. The selected Rust backends,
Rust build scripts and FFI locations were inspected; both native and linked Wasm
build. The Wasm module imports nothing. Unsafe Rust inside dependencies is not a
C/C++ kernel, and this audit is not a memory-safety/security or P35 whole-product
acceptance claim.

Follow-up for io.obj.companions.images: reproduce full alpha conversion against
the complete 256×256 channel/alpha atlas; choose/port a source-faithful JPEG path;
match 16-bit conversion and truncated-image omission; then complete the ordered
MTL/image continuation with strict allocation/work limits and actual Wasm pixels.
Broader color profiles/Exif/animation/interlacing/rare-format corpus remains open.
Do not add this candidate to application Cargo files on the strength of a build.
