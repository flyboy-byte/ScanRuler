# M3 test-tool adapters (2026-11-05)

The scheduled M3 batch adds xtask-only dependencies on the existing internal port-engine and port-io crates. These are AGPL-derived workspace crates, with no new third-party dependency or application runtime edge. They let tool-owned adapters compare PNG captures and the private real scan without editing Claude-owned implementation files. Existing corpus generators/comparators and all seeds/tolerances remain authoritative. Native product crates do not depend on xtask.
