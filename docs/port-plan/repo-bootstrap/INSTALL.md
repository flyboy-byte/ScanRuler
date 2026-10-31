# Activate in the fork after obtaining a full checkout

Nothing in this kit has been installed remotely. Preserve existing project files and review conflicts before copying templates.

1. Place this planning packet at `docs/port-plan/` (omit its bundled repo-bootstrap subtree if desired). Keep `plan/units/` and `tools/port_plan.py`; unit files are canonical.
2. Adapt this kit's AGENTS/CONTRIBUTING, PR/issue template and plan.yml into repository-root paths. Existing upstream documents are source history; reconcile them intentionally rather than blindly overwriting.
3. Verify action pins, branch name and paths. Run the live plan check and selftest locally. The packet SHA256SUMS/verify.py describe a frozen export and are not a requirement that living work remain unmodified.
4. Implement the Rust workspace, pinned toolchain, reference-pin contract and xtask harness/gates. Then adapt/enable port-gate.yml.example. It must fail if prerequisites/baselines are absent. Add actual reports/artifacts and cross-platform/browser jobs from CI-AND-REGRESSIONS.
5. Only after local validation and user authorization, publish the branch/templates and set repository branch rules. This kit creates no GitHub issues, claims, secrets or branch protection automatically.

The Python live checker is temporary bootstrap tooling; migrate its behavior into Rust xtask under P39 for whole-repository completion. It checks metadata/dependencies, not actual geometry results. Product acceptance is the real CI's responsibility.
