<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# PNG decoding against Chrome — 2026-11-04

Unit `io.obj.companions.images.png` (child of `io.obj.companions.images`). Author: Claude.
Judge: the original's `decodeImage` in actual Chrome 156.0.8078.79 (Linux, headless), through
`docs/port-plan/oracle/image-decode.mjs`. No acceptance claim.

## What the original does

`decodeImage` calls `createImageBitmap(blob, { colorSpaceConversion: 'none', premultiplyAlpha:
'none' })`, draws the bitmap on a 2D canvas and reads `getImageData`. Every failure is caught and
the image is left out. So the observable result per file is: omitted, or width, height and RGBA.
There is no ScanRuler decoding code to translate; every rule below was derived from Chrome's
outputs (black box), and the zlib thresholds were then confirmed by reading the MIT/Apache-2.0
`fdeflate` 0.3.7 and `png` 0.18.1 sources that match what Chrome 156 ships. No code was copied.

## Rules reproduced (each pinned by named probes)

- Canvas round trip: premultiply `(c·a + 127) / 255`, then un-premultiply in float32,
  `round_half_even(p·(1/255) · 1/(a·(1/255)) · 255)`, clamped; alpha 0 gives 0, alpha 255 passes.
  Exact for all 65,536 pairs (the audit's `png-alpha-atlas`).
- 16-bit samples keep their high byte; 1/2/4-bit gray scale by bit replication; gAMA, sRGB and
  iCCP are not applied.
- Chunks: critical chunks need a valid CRC, ancillary ones with a bad CRC are skipped; IHDR first
  and once; unknown critical chunks fail; PLTE needs 1 to 256 whole entries; IDAT must be
  consecutive; the image is complete once the 8-byte header of the chunk after the IDAT run is
  present (any type, length or CRC), so a file with no chunk after IDAT is left out.
- Palette indices past PLTE take the last entry and its tRNS alpha.
- tRNS: before PLTE it is ignored; at 16 bits the chunk must be exactly the key; at 8 bits or fewer
  a longer chunk is accepted and samples compare with the key's low byte.
- APNG: a valid acTL (length 8, CRC, frames 1 to 2^31 − 1, plays ≤ 2^31 − 1) before IDAT and no
  fcTL before IDAT make frame 0 the first fcTL and its fdAT run, placed on a transparent canvas,
  and IDAT is not decoded. fcTL length, CRC, sequence, region, dispose and blend are checked even
  without acTL; fdAT CRCs are not.
- zlib: the header is checked and Adler-32 is not. Before the image's bytes are complete every
  stream error and end of data fail. After that nothing more is written; the decoder follows only
  block ends (15 bits buffered), block headers (10), stored headers, dynamic headers (17, then
  3 per code-length code, at least 7 per code-length symbol), using only the rest of the IDAT chunk
  where the image finished. Huffman sets must be complete; a distance set may be empty or a single
  one-bit code.

## Results

Offline (`cargo test -p port-io --test png`, committed in `docs/port-plan/fixtures/png-chrome`):

| Set | Cases | Decoded | Omitted | Host limit | Mismatches |
| --- | ---: | ---: | ---: | ---: | ---: |
| named | 223 | 140 | 82 | 1 | 0 |
| sweep-s1 | 2,909 | 137 | 2,772 | 0 | 0 |
| struct-s11 | 2,400 | 1,334 | 1,010 | 56 | 0 |
| image-audit PNG fixtures | 13 | | | | 0 |

A wrong first byte in every decoded image is caught (`comparison_rejects_a_wrong_pixel`).

Live, fresh seeds through Chrome (`pinned_png_comparison`, seeds 2,3,4, 1,500 flips per sweep base,
2,000 structural images), release build, 74 s:

| Set | Cases | Decoded | Omitted | Host limit | Mismatches |
| --- | ---: | ---: | ---: | ---: | ---: |
| sweep seeds 2, 3, 4 | 30,327 | 1,513 | 28,814 | 0 | 0 |
| structural seeds 2, 3, 4 | 36,000 | 20,270 | 14,928 | 802 | 0 |

With `SCANRULER_PNG_SABOTAGE=1` (seed 5, smaller sets) 1,138 of 4,109 cases were flagged and the
run reported the sabotage as detected.

During development, fourteen probe and fuzz sets (about 70,000 cases) reached zero mismatches
outside the host-limit class after the rules above; each earlier mismatch class is a named probe.

Run it again:

```bash
cd rust
cargo test -p port-io --test png                       # offline, no Chrome
cargo test --release -p port-io --test png pinned_png_comparison -- --ignored --nocapture
SCANRULER_PNG_SABOTAGE=1 cargo test --release -p port-io --test png pinned_png_comparison -- --ignored
```

## Host classes and open items

- Host limit: `PngLimits` (2^26 pixels, 256 MiB input by default) refuses what Chrome may decode
  or fail differently; counted separately and only accepted where Chrome also left the image out.
  Pixels are allocated after inflation, so a header claiming more than its data costs only what
  the data decodes to.
- Inferred: a very large IDAT chunk could reach Chrome's decoder in several pieces, which would
  move the post-image cut-off; untested.
- Not yet wired into `readCompanions`/`scanCompanions`; JPEG, BMP, WebP and the ordered companion
  continuation remain in the parent unit. One Chrome version, no Wasm run.
