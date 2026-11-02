<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Original source coverage bundle

Clean tool checkpoint96c776c2abe15e184b1dd39c21a13387c88d59fa; complete native
input/result checkpointfa711afd2275c22279b287ede832c165c51b4a1e, identical Rust
tree. Node22.23.4, TypeScript5.9.3.1422 exact full output/status/sidecar replays,
eight mapped sources,18 explicitly unmapped source paths. No acceptance claim.

raw.json.gz contains full pinned source/emitted JavaScript/compiler maps, detailed
V8 ranges and every case input/output/sidecar hash. The gzip mtime is zero.
manifest.json hashes the raw archive and generated JSON/Markdown reports. The
report is regenerated and mutation-tested by original-coverage.test.cjs, required
in catalog-contracts CI. [Reproduce](../../oracle/ORIGINAL-COVERAGE.md).

Unreached locations are corpus/continuation targets. This is an observed V8 block
union, not static condition/edge coverage, and excludes independent fuzz/browser
campaigns and untrustworthy helper source mappings. All prior comparisons remain
mandatory. No source/Rust answers are injected into the other implementation.
