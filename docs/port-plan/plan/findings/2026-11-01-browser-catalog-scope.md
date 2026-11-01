# Browser catalog validation boundary — 2026-11-01

Unit catalog.browser (P02), based on original headless capture PR #13. The headless runtime's registered metadata is not automatically proof of what a mounted browser page registers. This contribution directly reads the original live page registry only after actual viewport/session startup, in fresh core/all Chrome contexts.

The launcher is a separate inspectable source-oracle file copied untracked into the frozen checkout; there is no nested source-string serialization or schema substitution. Its output is canonicalized and validated outside the page, with exact original sources/locks/version. Command schemas, capability flags, browser/version/language context and source helper console outcome are retained. Source/UI failures cannot become zero-command passing captures.

Eight local validator tests and launcher syntax checks pass. Actual Chrome output is still pending; no browser baseline captured/accepted state is claimed yet. Production page/MCP equivalence and nested feature variants remain open.
