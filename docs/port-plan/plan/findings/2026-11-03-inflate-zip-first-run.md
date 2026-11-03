<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Own DEFLATE and ZIP reader against fflate (2026-11-03)

`rust/crates/port-io/src/inflate.rs` reproduces `fflate` 0.8.3 (the version in the original's
`package-lock.json`; the oracle script refuses to run if the installed version differs) as the
original's 3MF reader calls it: `unzipSync(data, { filter })` with the filter `/\.(model|rels|config)$/i`.
No dependency was added (decision `2026-11-03-own-inflate-zip-png-jpeg`).

## Behaviour reproduced on purpose

Bytes past the input read as zero; stored blocks ignore `NLEN`; Huffman tables are built without
validation (bad trees collide instead of failing); the output has the entry's declared size, writes past
it are dropped and the result is the first `min(written, declared)` bytes; an empty compressed stream
yields declared-size zeros; distance symbol 31 copies zeros; a stored block that does not fit is a JS
`RangeError`; the zip64 branch, the 65,558-byte end-record search, the Latin-1 versus UTF-8 name flag,
and repeated names (first position, last value).

## Result

`pinned_zip_inflate_comparison` (ignored; seeds 1, 7, 99; 5,000 archives per seed). Archives are made by
fflate itself (levels 0, 1, 6, 9; names, empty and random contents); each is read unmutated and in one
mutated copy (byte edits, and header fields set to edge values, aimed at the end record, the entry data
or anywhere). Compared: entry names, order and bytes, or the error code and text.

| Archives | Identical success | Identical refusal | Host limit | Divergences |
| --- | --- | --- | --- | --- |
| 30,000 | 25,158 | 4,825 | 17 | 0 |

Refusals seen, all identical to the original: `invalid zip data` 4,462; `invalid distance` 166;
`unexpected EOF` 88; `invalid block type` 20; `invalid length/literal` 18; unknown compression type.
With `SCANRULER_ZIP_SABOTAGE=1` (one byte appended to the first Rust entry) the same comparison reports
721 divergences in 200 archives per seed, so it can fail.

## Classes reported apart

- **Host limit:** Rust refuses an entry whose declared size exceeds 64 MiB, or more than 2^20 entries,
  and the original either succeeds or fails with an allocation error (`Array buffer allocation failed`).
  Not matches, not divergences.

## Not done (open)

- Not in the native suite roster (`ci/native-suites.json`), so the hosted native-oracle job runs it only
  as an extra ignored test; no case inventory is registered.
- Not wired to `cargo xtask fuzz`; not fuzzed with the structure-aware generator beyond the mutations
  above. Hand-built zip64 and encrypted entries are reached only by random mutation.
- Compression ratio and total output are bounded only per entry; `io.threemf` must add the original's
  own limits and the XML work.
- Browser (Chrome) execution of fflate is not compared; the oracle is Node 22.
