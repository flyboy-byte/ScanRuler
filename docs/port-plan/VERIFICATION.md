# Verification evidence labels

Updated 2026-11-03. Lifecycle and verification answer separate questions. A unit can
be implemented while its verification is none. Neither label declares the application
usable. The generated [status](STATUS.md) shows both, with separate implementation
and higher-proof tested snapshots. `next_task.py` shows labels on tasks/dependencies.

| Label | Required support |
| --- | --- |
| none | No qualifying machine-checked evidence recorded; makes no parity claim. |
| fixed-corpus | Nonempty pinned original/Rust case metadata at the implementation SHA, or a retained passing named tooling unittest run at that SHA. |
| fuzzed | Fixed basis plus a scoped exact-candidate campaign, positive successful full comparisons, zero ordinary divergences, retained logs and a corruption canary with actual differing output/state. |
| coverage-reviewed | All previous support plus explicit review of both comparison-only coverage profiles, named case profile/reviewer, no unresolved or unknown reach, and retained profile hashes. |

Accepted requires coverage-reviewed and all previous acceptance/dependency/open-item
checks. No current unit is coverage-reviewed or accepted. A raw coverage report,
compilation, status-only comparison or prose summary cannot supply a higher label.

The migration retains every lifecycle, numerical case, seed and tolerance. Forty-four
native differential units have fixed-corpus support; four of those parser child units
have additional fuzz support. Tooling without a retained structured run stays none;
implemented alone is insufficient. This PR's verification tool has its own retained
fixed test profile. Historical reports remain unchanged.

The fresh [PLY/STL campaign](evidence/verification-fuzz-2026-11-03/report.json) tests
candidate `60bf04b91d8ee0e304dcb4c22e27e91b9c1b63b8`: seeds 1, 7 and 99,
1,000 inputs each per family, plus seed-1 corruption campaigns. Both ordinary
families have zero divergences. Host refusals and source timeouts remain separate
from successful comparisons. The report names scope, invocation, counters, compressed
full logs, hashes and minimized canary input/output/error/progress payloads. This is
finite family-level mutation coverage, not exhaustive parser coverage. Parser parents
and unrelated import units do not inherit the label.

High-level proof hashes cover the owned production modules, their actual parser
and number-formatter dependency closure, and generator/comparator. Unrelated new
modules do not invalidate a parser run. Changing those hashed files makes the proof
stale: rerun it or explicitly downgrade with an explanation. Validators check metadata
and integrity, not authenticity or runtime execution. The fixed-corpus legacy path
checks existing pinned case metadata; it does not newly authenticate old artifacts.

Reproduce metadata/negative controls from the repo root:

```sh
python3 docs/port-plan/tools/port_plan.py --root docs/port-plan selftest
python3 -m unittest discover -s docs/port-plan/tools -p 'test_*.py'
python3 docs/port-plan/tools/port_plan.py --root docs/port-plan check-generated
python3 docs/port-plan/tools/ci_report.py status --check
```

Run live native/original checks separately with `cd rust && cargo xtask ci` and
`cargo xtask oracle`. The generic paired diff/exact-base ratchet, complete session,
commands and M3 remain open; these labels do not replace those gates.
