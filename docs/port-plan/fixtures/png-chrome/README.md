<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# PNG cases judged by Chrome

Inputs and Chrome 156 results for `io.obj.companions.images.png`. The judge is the original's
`decodeImage` (`src/app/scanFiles.ts` at the pin) in actual Chrome, run by
`docs/port-plan/oracle/image-decode.mjs`. No Rust output is in these files.

| File | What |
| --- | --- |
| `generate.py` | Authoring tool, Python standard library only. `named`, `sweep SEED N`, `struct SEED N`. |
| `capture.py` | Regenerates the three committed sets and captures Chrome's results (Node 22 via fnm, Chrome). |
| `named.bin.gz` | 223 hand-built probes; names in `named.names.json`, in order. |
| `sweep-s1.bin.gz` | Every truncation and 300 byte flips of each of six base images (2,909 cases). |
| `struct-s11.bin.gz` | 400 random images, each with five CRC-fixed mutations (2,400 cases). |
| `*.chrome.bin.gz` | Chrome's results for the set of the same name. |
| `manifest.json` | Case and result hashes, Chrome, Python and zlib versions, seeds. |

Batch format: u32 count, then per case u32 length and the bytes. Result format: u32 count, then
per case u8 status (1 decoded, 0 omitted, 2 timed out) and, when decoded, u32 width, u32 height,
u32 length and the RGBA bytes. Both gzip, mtime 0.

`cargo test -p port-io --test png` compares Rust with these offline. The ignored
`pinned_png_comparison` (opt-in with `SCANRULER_PNG_LIVE=1`) generates fresh seeds and runs Chrome now (see the finding
`plan/findings/2026-11-04-png-chrome-port.md`). Python's zlib compresses the image data, so a
different zlib can produce different bytes from the same seed; the committed files and their
hashes are authoritative.
