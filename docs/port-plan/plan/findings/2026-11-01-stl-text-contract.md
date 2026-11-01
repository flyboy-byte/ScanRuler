<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# STL text compatibility findings — 2026-11-01

Unit `io.stl.dispatch`, pinned original `2cd89c6`, implemented/compared natively at `77683bf`. Evidence: `evidence/2026-11-01-stl-dispatch.md` and its case manifest.

- The ASCII reader is a permissive regex scanner, not a grammar validator: embedded `notvertex` matches, numeric suffixes are accepted, and invalid numeric tokens become NaNs without rejecting an otherwise complete triangle count.
- Rust whitespace and whole-string float parsing are unsuitable direct replacements. All 25 JS whitespace cases, excluded U+0085/U+180E/U+200B/U+001C, unfinished exponents and hexadecimal-looking prefixes are now independent paired cases.
- A whitespace prefix of 507 bytes still allows the full `solid` token into the 512-byte head. At 508 bytes it is cut off; the same ASCII-looking file instead uses binary fallback. Preserve this source behavior rather than inferring format from the full file.
- The prior binary helper accepts absent final attribute bytes, but the dispatcher uses complete 50-byte record counts on fallback. A 132-byte non-solid input therefore produces zero triangles instead of one. Both contracts remain covered separately.

These are source contracts, not proposed source fixes. Do not replace the reader with a stricter STL parser or silently normalize non-finite values. The next welding port must preserve exact-bit identity; whole-app sanitation belongs at its actual source call boundary.
