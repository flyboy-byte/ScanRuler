<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Bounded fuzz reproducer reduction

Run `cd rust && cargo xtask fuzz stl --seed 1 --cases 20 --selfcheck --minimize --out /tmp/scanruler-reduce`.
Replace `stl` with `ply`; omit `--selfcheck` for actual fuzzing. Selfcheck deliberately
corrupts Rust output and is labelled in the reduction record.

`--minimize` reduces only the first divergence, using deterministic chunk deletion
and at most 128 candidate evaluations. Original input files and the existing
case schedule are retained. This is bounded reduction, not a claim of a globally
minimal input. PLY reductions can be slower because each independent source
process transpiles the pinned parser.

A candidate must preserve both engines' success/refusal status and exactly which
of status, progress, error message and output bytes differ. Host-limit refusals
and source timeouts never count as preserved mismatches. The final candidate is
rechecked independently before saving. A source crash or nondeterministic result
fails the reduction rather than becoming a passing comparison.

The `*-reduced/` folder contains the input, complete original/Rust output bytes,
progress, messages and status, plus seed, case index, original/reduced lengths,
attempt count, budget, mismatch signature and sabotage flag. The original failing
input remains beside the folder. No expected outputs enter either parser.

The reducer does not alter the parser, tolerate a mismatch, discard a seed, or
replace the regular native/browser comparison gates. OBJ and scheduled fresh
seed execution remain with the parent fuzz unit.
