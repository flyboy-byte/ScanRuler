<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# fixtures/

Committed fixtures are **generated, licence-clean and deterministic**: every file has a manifest entry (generator,
seed, hash, units, independently derived expected values). The manifest and generators are built under P03; this
folder is where they will live.

`fixtures/private/` is gitignored. Put real scans or CAD you own there for local tests. Anything read from it
stays out of the repository, CI logs and PRs, and a test that needs it is reported **skipped**, never passed.
The original author's `ballbar.stl`, `block-marius.stl` and `side bracket left.stl` are his private files; we do
not have or redistribute them. See `docs/port-plan/ROADMAP-TO-USABLE.md` section 5.
