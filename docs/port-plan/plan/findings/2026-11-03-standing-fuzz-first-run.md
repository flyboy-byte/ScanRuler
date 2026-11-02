<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Standing fuzz, first in-repo run (2026-11-03)

`cargo xtask fuzz <ply|stl> --seed N --cases M` replaces the scratch harness described in
`2026-11-02-ply-stl-independent-fuzz.md`. The generator and the comparison are committed:
`rust/crates/port-io/tests/fuzz_standing.rs` (seeded model-based files plus header/body mutation:
truncation, byte substitute/insert/delete/bit flip, a 47-token dictionary and digit-run edits to edge
numbers) and `docs/port-plan/oracle/fuzz-batch.cjs` (the frozen original, loaded from the pinned Git
object, many inputs per Node process, 1.5 s limit per input).

## Result

Candidate: this PR's head. Original: `2cd89c6769a10c27ee7754b477ada521c9ef5cb7`. Node 22.

| Unit | Seed | Cases | Identical success | Identical refusal | Host limit | Original timed out | Divergences |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ply | 1 | 10,000 | 3,638 | 6,205 | 96 | 61 | 0 |
| ply | 7 | 10,000 | 3,556 | 6,298 | 102 | 44 | 0 |
| ply | 99 | 10,000 | 3,687 | 6,175 | 88 | 50 | 0 |
| stl | 1 | 30,000 | 20,838 | 9,162 | 0 | 0 | 0 |
| stl | 7 | 30,000 | 20,893 | 9,107 | 0 | 0 | 0 |
| stl | 99 | 30,000 | 21,018 | 8,982 | 0 | 0 | 0 |

"Identical" means success or failure, progress text, error text and output bytes all equal.

## Classes reported apart from divergences

- **Host limit:** Rust refused for a caller budget (`ResourceLimit`, `Allocation`, `InputLimit`,
  `TriangleLimit`). Not a match and not a divergence. These are the open H1 to H3 classes.
- **Original timed out:** the pinned original did not finish in 1.5 s (H3). Not compared.

## The check can fail

`cargo xtask fuzz <unit> --selfcheck` flips one bit of the Rust output and requires divergences.
Seed 1, 300 cases: STL 194 divergences, PLY 108. Without that, a clean run proves nothing.

## Not done (still open)

- Reproducers are written whole to `--out` (default the system temp directory); they are not minimized.
- The scheduled hosted job is not added. `.github` and `rust/xtask` are one-PR-at-a-time files.
- OBJ is not covered; the generator is PLY and STL only. The old audit's exhaustive per-byte
  truncation and Unicode set are covered only as far as random mutation reaches them.
- Not an acceptance claim: this raises the verification level of `io.ply` and `io.stl.*` to fuzzed;
  coverage review is separate (`harness.coverage.*`).
