<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Parser differential check, pin 0.9.2 against upstream 0.9.4

Supports [the drift ledger](../../plan/findings/2026-11-03-upstream-drift-0.9.2-to-0.9.4.md).
`diff-parsers.mts` runs `parseSTL`, `parsePLY` and `parseOBJ` from two source trees on the
same generated inputs and compares the full result (kind, positions, indices) or the error
text. `fuzz-2026-11-03.log` is the raw output of the two runs the ledger reports.

Reproduce, from a scratch directory outside the repository:

```bash
git -C <upstream-clone> archive 2cd89c6769a10c27ee7754b477ada521c9ef5cb7 src | tar -x -C pin/
git -C <upstream-clone> archive 5c1f2f9 src | tar -x -C current/
npm i tsx@4.19.2                      # not a dependency of this repository
npx tsx diff-parsers.mts pin/src current/src 20000                     # run A, hostile mix
NOBADCOUNT=1 PURE=1 npx tsx diff-parsers.mts pin/src current/src 40000 # run B, mostly well-formed
```

Seeds are fixed, so the runs repeat exactly. `PURE=1` makes numbers and indices valid;
`NOBADCOUNT=1` also keeps PLY list counts numeric and line ends to `\n` and `\r\n`.

Limits: the inputs are far smaller than the 16 MiB piece size of the new reader, so its
chunk-boundary logic is not exercised; there is no binary STL or PLY, no real file and no
3MF; the generator is rough (it can emit whitespace that is not JavaScript `\s`), and the
remaining PLY differences were not each traced. This is a screen for behaviour change, not
a parity proof.
