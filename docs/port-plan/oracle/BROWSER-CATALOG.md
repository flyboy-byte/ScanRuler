# Original browser registry capture

Bounded P02 original-oracle tooling only. Based on the independent headless-catalog PR #13, not the numerical port stack. It starts the frozen original Vite app using its own launchApp/finish Chrome helpers in fresh core/all processes and browser profiles. It never calls registerCoreCommands/installPlugins itself or substitutes static/Rust descriptors.

Before writing a capture, the app must have an installed live registry, expected original plugins, a real session.state result with viewport=true and scan=null, and a passing original console/page-error policy. Metadata is read from actual registry exports inside the live page. Every browser-registered command is listed; headlessFlagNames/pageOnlyFlagNames describe the original capability flag, not whether a command was successfully executed against a loaded scan.

From the fork root, Node 22 and Chrome required (CHROME override supported):

```bash
node --test docs/port-plan/oracle/browser-catalog-contract.test.mjs
(cd .oracle && npm ci)
(cd .oracle/mcp && npm ci)
node docs/port-plan/oracle/capture-browser-catalog.mjs .oracle core rust/target/original-browser-catalog
node docs/port-plan/oracle/capture-browser-catalog.mjs .oracle all rust/target/original-browser-catalog
```

Prepare the exact .oracle checkout per UPSTREAM-SYNC.md first. Tracked source/locks are verified clean before and after capture; the launcher is copied untracked only to resolve the original Vite/puppeteer dependencies. Browser setup uses the original helper's standard synthetic-profile preferences, the original local MCP pairing flow and no user files; the fresh synthetic profile accepts the same agent-data preference as the original E2E test. Source failures remain errors, with screenshots/HTML/console diagnostics retained by the original helper. Each workflow capture retains raw JSON and diagnostics 30 days.

Eight synthetic validator cases cover: valid context, rejection of headless viewport/loaded scan/console failure/dropped offered name/incorrect capability partition/empty registry, and schema-loss detection. These synthetic validator tests are separate from actual browser results. Hosted run [41356099428](https://github.com/flyboy-byte/ScanRuler/actions/runs/41356099428) successfully captured both contexts with Node 22 and Chrome 156.0.8078.47. Core: 72 registered / 6 headless flags / 66 page-only flags. All: 140 / 36 / 104. Both prove viewport=true and scan=null, with the original helper passing. Immutable gzip JSON and raw/compressed hashes are in evidence/browser-2026-11-01/. Full ordered schema hashes equal the independently captured original headless registry metadata; that does not equate command execution capabilities. Snapshot tests and future hosted captures compare the complete original metadata, not counts alone. Node 20 can test the pure validator but cannot run capture.

This observes the original development-server page, not the production bundle or a Rust browser. Nested feature variants, wider G0/P01 evidence and Rust coordinator/purity work remain open. Parent P02 cannot be marked accepted from this harness alone. Canonical units own status; regenerate backlog.json during integration rather than competing with the stacked port index.

Attempt log: hosted runs 41354991898 and 41355455158 failed readiness with no core commands registered. Main-world polling alone did not resolve it. Source inspection of bridge.ts connect() established that core command registration is lazy on agent connection. The corrected launcher uses the real frozen SDK/server, pairing hash, source E2E helpers and scanruler_wait; it never registers commands manually. The same fixed readiness/viewport and console gates remain.
