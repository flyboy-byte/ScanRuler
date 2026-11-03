# Verification levels describe scoped evidence, not lifecycle completion

2026-11-03, before migration. harness.verification.levels is owned by Codex in
one PR over the combined port queue. No mathematical/source comparator changes.

The ordered labels are none, fixed-corpus, fuzzed, coverage-reviewed. Every unit
gets an explicit label. Implemented without sufficient tested evidence can still
be none; a label is never inferred solely from lifecycle or a prose claim.

Fixed-corpus requires pinned nonempty original/Rust differential metadata with
an exact tested candidate, or a structured fixed tooling proof. Fuzzed additionally
requires a structured exact-candidate campaign: full value/progress/error
comparison, positive successes, zero ordinary divergences, separate host-limit/
original-timeout counts, stable seeds, and a deliberate corruption canary that
produces divergences. Retain raw logs and current code/generator hashes; stale
or malformed evidence cannot support the higher label. Parser-family scope is
explicit. Historical prose remains visible but is not silently rewritten as a
machine-verifiable exact-candidate report.

Coverage-reviewed additionally requires an explicit comparison-only review record
for both original and Rust coverage, its reviewed case profile, current code hashes
and no unresolved/unknown reach items. Raw coverage percentages, compile success
or unexamined zero-count/branch records do not qualify. No current unit is assigned
this level. Accepted requires this highest label as well as all existing accepted
dependency, evidence and open-item gates; the label alone cannot grant acceptance.

Validators check evidence support/identity/integrity. They do not authenticate a
reviewer's identity, replace execution, invent exhaustive fuzz coverage or implement
the generic P36/P37 paired report/exact-base ratchet. Actual current-candidate
runtime gates remain mandatory. The status and task picker must show level and
its tested snapshot rather than imply the entire application is usable.
