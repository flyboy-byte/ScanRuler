# Pending port-stack integration

2026-11-03. Codex combines #73 with #74, then #69 and #64, on an isolated
integration branch. Parent PRs remain independently reviewable. No main/PR merge
is performed by this operation. Existing source feature bodies are unchanged.

The first union retains all108 canonical units and all44 mandatory native suites
with6854 case identities, including the old311 UTC records and new1600 period
cases. Conflicts are resolved by unioning distinct module/suite/lane declarations
and preserving both sides' changelog/audit text. Generated status/backlog are
regenerated from canonical units; inventory assertions use the computed union.
Shared suite definitions must be byte-equivalent; missing or changed prior cases
cannot be accepted as a resolution. The CI scheduling/seed-label refinements
are integrated separately and revalidated at the final combined head.

Complete session/feature commands, actual new engine browser execution and the
generic paired diff/exact-base ratchet remain open. Integration does not promote
unit lifecycle or verification strength. The PR must retain full gate evidence
at its actual head before review readiness.
