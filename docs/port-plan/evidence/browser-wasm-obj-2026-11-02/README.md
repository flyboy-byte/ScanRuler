# Actual Chrome OBJ runtime proof — 2026-11-02

The manifest freezes the exact executed candidate/reference/module and runtime
roster. `report.json` records all 564 independently executed comparisons;
`cases.csv` is an index. `raw.json.gz` is deterministic gzip of UTF-8 JSON with
all raw input/original/native/Wasm bytes as hex, including PLY progress sidecars
and envelopes. Decompress and replay using `browser-wasm-contract.test.mjs`.
Inputs are authored fixtures; source results never enter Rust or the page.

482 complete outputs/helper states and 82 source refusals pass; 18 protocol
checks pass. All old 341 browser cases remain. OBJ1 and OTX1 results compare
byte for byte, including IEEE float32 bits, full sampler buffers, colors,
ordered notes, progress and refusal messages. Historical proofs are unchanged.
No module imports or page errors. The browser-observed module hash matches.

Three declared exclusions remain native-only, never counted as browser passes:
PLY million-vertex progress and both full-size OBJ million-line progress inputs.
Limits stay 1 MiB input/output and 64 MiB memory; no image-file codec, public
command/session/UI integration or unit acceptance is claimed. Node22/TypeScript
and the launcher are temporary reference/test tooling, not a shipped runtime.

Clean tested checkpoint: `239012c9648f645a3039856e5c511857fb62e691`. All 23 native suites /1,191
identities and their exact-candidate raw inventory passed before the actual Chrome
run. Codec implementation is `6bcab1d1a436fb757ce2023a001f70d3972b5a05`;
subsequent checkpoint changes add replay tests, proof and documentation.
