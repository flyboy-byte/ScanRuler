# Full comparison artifacts exhausted the RAM disk

2026-11-03. Parallel local #86/#87 native gates exhausted the 7.5GiB /tmp tmpfs.
The #86 PLY ASCII artifact write failed with StorageFull/OS28; #87's reference
builder failed during its temporary build. These were failed runs, not divergences
or passing evidence. Completed #85 proof remained intact. No cases, seeds,
tolerances, timeout budgets or production/source code changed in response.

Retained completed/incomplete target directories moved intact to ignored project
SSD scratch .port-run-artifacts/, with original rust/target paths symlinked to them.
Duplicate large command-label transcripts were removed only after matching their
recorded hashes; complete source transcript and paired checked-in evidence remain.
New worktrees use ignored .port-worktrees/ on the SSD. Retry unchanged affected
heads and require fresh complete exact-head raw artifacts; an interrupted run
cannot satisfy the completeness gate. These are local scratch paths, not product
configuration or a new CI storage requirement.
