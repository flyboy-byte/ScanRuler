# Recording trace seed label correction

2026-11-02. The history-record live adapter identity.txt hard-coded seeds
1,7,99,1109, while the unchanged generator, manifest and case IDs all use
1,7,99,1039 (twelve cases per seed). This is an audit metadata typo, not a
geometry/state divergence or a different corpus. All 110 original/Rust cases
and immutable archives remain unchanged, including their historical identity text.

New live reports derive the seed list from the actual fixture IDs. An ordinary
Rust regression asserts the fixed four-seed set. No source/Rust output is
regenerated, and no seed is added, removed, substituted or relabelled in a case.
The source-only archive and manifest hashes remain authoritative for old reports.
