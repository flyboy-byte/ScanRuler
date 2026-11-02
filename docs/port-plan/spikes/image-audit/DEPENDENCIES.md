# Exact selected dependency review — 2026-11-02

Evidence: `evidence/2026-11-02-image-dependencies.json`, candidate.lock and the
capture tool. All 22 published package checksums, feature/dependency edges,
manifest/source-tree hashes, shipped license files and build scripts are frozen.
Native and Wasm resolution have identical selected graphs. Root image0.25.10 has
only bmp/jpeg/png/webp; defaults, rayon, AVIF/native AVIF and nasm are not selected.
Each declared license expression offers an option on the repository allowlist;
actual MIT/BSD/Zlib texts were inspected and hashes retained. Adoption would still
retain notices/attribution. No application Cargo file changes in this audit.

| Path | Review of actual selected code |
| --- | --- |
| png → flate2 | any_c_zlib is absent; Rust backend/runtime detection selects Rust miniz/zlib-rs implementations. Optional C backend module is cfg-gated out. |
| zlib-rs | Implementation is Rust. The Rust allocator is selected. C ABI callback signatures and alternate OS malloc/free paths are allocator interfaces, not a C compression kernel. Wasm links without imports. |
| crc32fast build.rs | Invokes rustc --version and emits Rust intrinsic cfg flags; no native compiler/link step. |
| num-traits build.rs → autocfg | Compiles a Rust expression to detect compiler support; no foreign kernel. |
| bytemuck extern-C matches | Implements Rust traits for C ABI function-pointer types; no C implementation or call imported. |
| image extern-C occurrence | Documentation example, not a decoder FFI dependency. |
| pxfm cmake/dylib occurrences | Offline coefficient-generation examples in comments; runtime coefficients/functions are Rust. No build invocation or foreign linked library. |
| moxcms, pxfm, zune-jpeg, CRC/Adler paths | Rust implementations with architecture intrinsics; unsafe Rust is present in dependencies. That is distinct from a C/C++ kernel and is not a memory-safety audit result. |

No C/C++/assembly library implementation files or native-build dependencies are
in this selected package source inventory. This mechanical scan is supporting
evidence; the cfg/build/FFI review above and linked Wasm import inspection are
separate checks. The conclusion is limited to this exact graph/toolchain, not
future updates or all image features. Neither purity nor an allowed license
changes the negative pixel-compatibility verdict.
