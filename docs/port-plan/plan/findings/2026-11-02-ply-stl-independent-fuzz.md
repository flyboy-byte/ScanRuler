<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Independent PLY and STL mutation fuzz (audit, 2026-11-02)

Audited commit `fabac0d` against the pinned original `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`. The
original ran in-process under Node 22 from the pinned Git object (same loading as
`docs/port-plan/oracle/ply-ascii.cjs` and `stl-dispatch.cjs`); the Rust side called `read_ply` and
`read_stl`. Each case compared success/failure, full progress and error text byte for byte, and the
complete output payload. Evidence is local, not a hosted run; the generators and raw results were
scratch files and are not committed.

## Coverage

- About 182,000 PLY files: exhaustive truncation, byte substitution, deletion and insertion over eight
  seeds; a 58-value header-token dictionary; four seeded random model-based sets covering all sixteen
  scalar types, both endiannesses, n-gons, element order and float specials; a numeric token-grammar
  set; Unicode whitespace, BOM, invalid UTF-8 and the 64 KB header boundary; progress messages up to
  16.7 M elements.
- 100,000 STL files: truncation, count/size mismatches, `solid` precedence, NaN/Inf bits, Unicode
  whitespace, numeric-prefix forms, BOM, invalid UTF-8, the 512-byte head boundary.
- Rust side: 0 panics, 0 hangs, 0 allocation failures. STL: 0 divergences.

## Divergence D1 (fixed by the PR that carries this file)

The negative-count error text used Rust number formatting, not JavaScript's. Original
`Invalid typed array length: -3e+21` versus Rust `...: -3000000000000000000000`, and `-Infinity`
versus `-inf`. Both the ASCII and binary vertex paths. Regression test:
`rust/crates/port-io/tests/ply_js_error_text.rs`, which fails on the old code and passes on the fix.

## Open: host-boundary classes (not counted as divergences, not decided)

Rust's resource limits refuse some inputs where the original fails with a different JavaScript
error or succeeds:

- **H1** The original's own RangeErrors (huge or non-finite counts, later "Offset is outside the bounds"
  after a lazy huge allocation) collapse into `PlyError::ResourceLimit`. Thousands of cases. With
  generous limits 93 of 4,655 re-runs became identical successes.
- **H2** Intermediate values are limited: a first `face` element with a huge triangle total that a
  later duplicate `element face` discards is refused by Rust and accepted by the original. About 955
  random cases.
- **H3** About 2,000 inputs make the original loop for over 1.5 s; Rust refuses them in under 25 ms.

Whether to keep these refusals, document them per class, or match the original more closely is the
owner's call. Do not tune limits to hide them.

## Not covered

Files over 50 MB; real scanner files (none on this machine); real 32-bit Wasm memory behaviour; the
ASCII path beyond 3 M tokens; exhaustive NaN payload patterns; the 776 data-driven list-count cases
confirmed by hand on only three samples.
