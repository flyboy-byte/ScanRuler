<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# #58 review regressions: allocation admission and UTF-16 directories

2026-11-03. Both findings reproduced before changing the port. No original
TypeScript, dependency, tolerance, seed or timing threshold changed.

The tests use reduced generated fixtures in `rust/crates/port-io/tests/threemf.rs`:

- Two eight-byte declared DEFLATE outputs with a twelve-byte aggregate cap must
  stop before allocating the second output. Empty decoded vectors retain their
  declared capacity; using their final lengths bypassed the old check. Repeated
  names are charged too. An invalid second stream proves budget admission runs
  before decompression, rather than only observing an eventual error afterward.
- Stored model/config entries exercise exact-boundary success and one-byte-over
  refusal; a filtered image spends no part budget. Stored entries also obey the
  documented per-entry cap before copying.
- A lone high/low surrogate in a directory, with slash/backslash and a trimmed,
  case-folded basename, must still apply a matching supplied image. A lone
  surrogate in the basename remains unmatchable by a UTF-8 companion name.

`unzip_filtered_bounded` checks allocation admission before every kept entry's
copy/inflate. It charges actual stored bytes or declared DEFLATE capacity,
including superseded duplicate names. This cumulative allocation/work budget is
an explicit host refusal absent from the original, not claimed source parity.
`Limit(PartBytes)` remains distinct from per-entry/count `Limit(Archive)` and
source damaged-ZIP errors. Names, archive input and other metadata are not
included in this byte budget; entry count and input ownership remain separate.
The previous unbounded-total ZIP API remains available for its other callers.

Texture fallback splits UTF-16 before testing basename validity, so malformed
directories cannot suppress a valid basename. Replacement text is used only for
the discarded directory and existing full-name host-budget accounting; it can
never turn an invalid basename into a matching replacement character.

## Retained evidence

All six earlier seeds (1, 7, 99, 1039, 4242, 31337), all 4,000 generated packages
per seed and all 90 earlier named cases remain. Four new named cases append at
indices 90–93, preserving earlier indices and random inputs. Both sides receive
independent inputs; no original result enters Rust.

- Before fixes: two focused tests failed. Full comparison: 18 divergences, the
  three successful directory fallbacks per seed (Rust omitted colours), with
  10,260 equal successes, 14,267 equal refusals and 19 separate host refusals.
- After fixes: 24,564 packages; 10,278 equal successes (2,252 coloured, 1,094
  textures applied, 1,888 missing), 14,267 equal refusals, 19 host refusals,
  no original timeouts, **zero divergences**. Full positions, indices, colours,
  texture notes, error text and progress remain compared.
- Same full corpus with `SCANRULER_3MF_SABOTAGE=1`: all 10,278 successful
  packages reported position divergences, 14,267 equal refusals and 19 host
  refusals. The harness self-check expects detection; it is not a clean parity run.
- `cargo xtask ci` passed (including all 168 Python tests, formatting, Clippy,
  workspace tests and Wasm compilation). Full candidate oracle result is reported
  in the PR body after the committed implementation is tested.

Commands from `rust/`:

```sh
cargo xtask ci
SCANRULER_3MF_SEEDS=1,7,99,1039,4242,31337 SCANRULER_3MF_CASES=4000 fnm exec --using=22 cargo test -p port-io --test threemf pinned_threemf_comparison -- --ignored --nocapture
# Add SCANRULER_3MF_SABOTAGE=1 for the wrong-value self-check on the same corpus.
```

Full logs remain under `.port-run-artifacts/nov03-threemf-review/` in the original
checkout. The red logs are retained, rather than overwritten by green runs.

| Log | SHA-256 |
| --- | --- |
| `red-tests.log` | `9f47b346f8c8de67063da6255bda0f6b9c10c33d783e02e2d621cdf989442014` |
| `red-comparison.log` | `444e28050a3f18ecb441a4765be7c5d02a4d35be0757f0d4ffc57610599bd908` |
| `green-comparison.log` | `2eb8411f292dcbda8fd8e4c77b26db557205303dfd8276f0c603638fd5b6fedb` |
| `canary.log` | `1cd5ac4e4375d8ef21c32cedb41760d2d5829126836aa6acb61731cfefa154c9` |
| `ci.log` | `67476f2c39b417dd8e0c5ea0453a35c2ba274cfc8df61a16b0f2670a5756d86d` |

## Open scope

No acceptance claimed. 3MF is still unregistered in the formal native case
inventory and fuzz command, and lacks Chrome evidence and packed-image decoding.
No real private-scan rerun was made in this follow-up; the earlier 107-file result
is historical, not a fresh claim. M3 still needs native source reaction ownership,
real command/session state, import, plane fitting, undo and compatible project save.
