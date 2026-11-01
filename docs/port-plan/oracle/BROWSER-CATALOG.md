# Original browser registry capture

Bounded P02 original-oracle tooling only. Based on the independent headless-catalog PR #13, not the numerical port stack. It starts the frozen original Vite app using its own launchApp/finish Chrome helpers in fresh core/all processes and browser profiles. It never calls registerCoreCommands/installPlugins itself or substitutes static/Rust descriptors.

Before writing a capture, the app must have an installed live registry, expected original plugins, a real session.state result with viewport=true and scan=null, and a passing original console/page-error policy. Metadata is read from actual registry exports inside the live page. Every browser-registered command is listed; headlessFlagNames/pageOnlyFlagNames describe the original capability flag, not whether a command was successfully executed against a loaded scan.

From the fork root, Node 22 and Chrome required (CHROME override supported):

```bash
node --test docs/port-plan/oracle/browser-catalog-contract.test.mjs
(cd .oracle && npm ci)
node docs/port-plan/oracle/capture-browser-catalog.mjs .oracle core rust/target/original-browser-catalog
node docs/port-plan/oracle/capture-browser-catalog.mjs .oracle all rust/target/original-browser-catalog
```

Prepare the exact .oracle checkout per UPSTREAM-SYNC.md first. Tracked source/locks are verified clean before and after capture; the launcher is copied untracked only to resolve the original Vite/puppeteer dependencies. Browser setup uses the original helper's standard synthetic-profile preferences, no agent pairing or user files. Source failures remain errors, with screenshots/HTML/console diagnostics retained by the original helper. Each workflow capture retains raw JSON and diagnostics 30 days.

Eight validator cases pass locally: valid context, rejection of headless viewport/loaded scan/console failure/dropped offered name/incorrect capability partition/empty registry, and schema-loss detection. These synthetic validator tests are not actual browser results. Hosted Chrome capture is pending until an actual successful run is recorded; do not claim a browser catalog before that. Node 20 can test the pure validator but cannot run capture.

This observes the original development-server page, not the production bundle/MCP transport or a Rust browser. Nested feature variants, wider G0/P01 evidence and Rust coordinator/purity work remain open. Parent P02 cannot be marked accepted from this harness alone. Canonical units own status; regenerate backlog.json during integration rather than competing with the stacked port index.
