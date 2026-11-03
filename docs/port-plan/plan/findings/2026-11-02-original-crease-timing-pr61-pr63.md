# Unchanged original crease timing failures retained

2026-11-02. PR61 original source run41585589657/job125750379923 measured
1697.32208ms against1500ms. PR63 run41585626487/job125750439192 measured
1505.715157ms against1500ms. Both failed only reNetworkCreases.test.ts:209;
370 files passed and13 skipped. All Rust, metadata, source browser/MCP and
license lanes passed. A single failed-lane rerun was requested for each.
No threshold, fixture, implementation, seed or CI assertion changed. The PR
bodies retain original failure links even if the reruns pass. This is the known
original timing flake, not evidence of source/native numerical disagreement.
