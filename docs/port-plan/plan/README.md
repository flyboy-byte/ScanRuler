# Canonical work units

`units/<id>.json` is the source of implementation status. P00–P39 are initial workstreams. Add dotted subunits with a parent ID for bounded contributor tasks; do not reserve a whole workstream just to port a few symbols.

Required fields: id, title, scope, acceptance, status, depends_on, reference_sha, evidence and implemented_commit. Additional fields identify source_paths/symbols/rust_paths, parent, soft_depends_on, open_items, owner, branch and claim_pr. Filename equals ID. Hard dependencies must form a DAG; soft dependencies document back-edges but do not authorize calling an unimplemented service or claiming full acceptance.

Statuses: planned, specified, baseline-captured, implemented, compared, integrated, accepted. Open blockers stay in open_items while preserving the last achieved stage. Accepted units have no unresolved items in their claimed scope and require accepted hard dependencies, a tested commit and real evidence. Small documentation/harness units use evidence appropriate to that scope; code that claims numerical parity needs real differential reports, not just metadata passing a checker.

Evidence entry: kind, location (artifact/report link), candidate_sha; differential entries also include reference_sha and case_ids. Reports themselves must carry the contract/fixture/seed/comparator identities from CI-AND-REGRESSIONS. The checker validates metadata only; actual CI must regenerate or validate results against trusted exact-base runs.

Check live metadata: `python tools/port_plan.py --root . check`. Self-test checker: `python tools/port_plan.py selftest`. Generate aggregate backlog: `python tools/port_plan.py --root . render`. In the future repository, point --root at docs/port-plan. Contributors edit affected unit files; maintainers generate aggregate snapshots in batches. Do not manually update both.

Keep findings and decisions in uniquely named individual files under plan/findings and plan/decisions; link the unit IDs and evidence. Templates are in templates/. No status transition creates a public claim, PR or issue automatically.
