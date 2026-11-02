# AGENTS.md

This file provides guidance to Codex (claude.ai/code) when working with code in this repository.

ScanRuler (by CNC Kitchen) is a browser-only tool for checking 3D scans against CAD: element fitting, deviation maps, wall thickness, 2D flatbed measuring, plus the Reverse engineer and Mesh Tools workspaces. Static Vite + React 19 + three.js + zustand build, heavy geometry in Web Workers, deployed as static assets on a Cloudflare Worker. AGPL-3.0-only with a commercial-exception model, so licensing rules below matter. User docs: `README.md`, `docs/manual.md`; the full dev/release/deploy runbook is `docs/development.md` (read it rather than re-deriving).

## Commands

```bash
npm install
npm run dev                      # vite dev server
npm test                         # vitest run (tests/ and plugins/*/tests/)
npx vitest run tests/foo.test.ts # single file; add -t "name" for a single test
npm run build                    # tsc --noEmit && vite build -> dist/
npm run e2e:ci                   # headless-Chrome suites on generated fixtures; run AFTER build
node scripts/e2e-<name>.mjs      # one browser check against a running dev server (see package.json e2e:* scripts)
npm --prefix mcp ci && npm --prefix mcp test   # the agent server, a separate package (node --test)
npm run build:engine             # headless engine -> dist-engine/engine.mjs (for scanruler-mcp --headless)
```

CI (`.github/workflows/ci.yml`) additionally runs `node scripts/check-plugin-boundary.mjs` and `node scripts/check-units.mjs`; run both after touching imports or anything that prints a length. Node >= 22. Browser checks need Chrome (`CHROME` env overrides the path); logs go to `e2e-out/ci/`.

Acceptance tests that need `ballbar.stl`, `side bracket left.stl` / `block-marius.stl` skip when those uncommitted scans are absent.

## Architecture

- `src/core/` — pure geometry/measurement logic (fit, deviation, field, section, flat, thickness, parsers, STEP/STL/3MF export, project file format). Heavy work runs in `meshWorker.ts`, reached through `workerClient.ts` / `workerRpc.ts` / `workerProtocol.ts`.
- `src/state/` — zustand stores (`store.ts`, per-workspace stores, `historyStore.ts` for undo). `src/app/` — import, project load/save, scene sync, workspace glue. `src/viewer/` — three.js viewport. `src/ui/` — panels and widgets.
- **Agent command layer (`src/commands/`)** is the central cross-cutting piece: every user-level action is also a registered command (`registry.ts`, `schema.ts`, `core/`, `guides/`), run by the panel's own code path as one undo step. The `mcp/` package (`scanruler-mcp`, its own `package.json`, tests, license) exposes these to AI agents over MCP and a local bridge to the open tab; `mcp/src/commands.json` is generated from the app's command list and carries the app version. `src/headless/engine.ts` runs the same commands without a browser. `skills/scan-measuring` is the agent-facing skill.
- **Plugins** (`plugins/reverse`, `plugins/mesh-tools`) add whole workspaces through `ScanRulerPlugin` in `src/plugins/api.ts`. Hard rule: nothing outside `plugins/` may import from it (the app finds plugins via only two globs: `src/plugins/registry.ts` and `src/core/workerPlugins.ts`); `?plugins=none` runs without them. Each plugin keeps its own `tests/`, `scripts/`, `ci.json` (browser checks CI runs). Extension points are tabulated in `plugins/README.md`; a new command, undo section, project-file key or state section goes through the matching `register*` call there.

## Conventions that bite

- Every file under `plugins/` starts with `// SPDX-License-Identifier: AGPL-3.0-only` (Markdown: HTML comment).
- Lengths shown on screen must go through the display-unit helpers (`check-units.mjs` enforces it).
- Dependencies: new npm packages or lifted algorithms must pass the license policy in `CONTRIBUTING.md` (no GPL/LGPL/AGPL code we don't own; never port from CloudCompare/MeshLab/OpenSCAD). `meshstep` and `occt-wasm` are pinned exactly (no caret); bumping `occt-wasm` has a checklist (fuzz goldens, bench, changelog) in `docs/development.md`.
- Version lives only in `package.json`. A release bumps it, regenerates `mcp/src/commands.json` (`UPDATE_MCP_COMMANDS=1 npx vitest run tests/mcpCommands.test.ts`), and adds a top `CHANGELOG.md` section; the tag push deploys. Changes (including plugins') are logged in the top-level `CHANGELOG.md`.
- Projects open by their own `schemaVersion`, not the app version; changing the project format is a major-version matter.
- Contributions need the CLA (`CONTRIBUTING.md`).

## Rust port contributor rules

This is a faithful pure-Rust port of the pinned baseline (`2cd89c6`, app 0.9.2); the original's behaviour and mathematics are authoritative. Read `docs/port-plan/HANDOFF.md`, `PURE-RUST.md`, `PORTING-PROCESS.md` and `CI-AND-REGRESSIONS.md`, then the relevant `docs/port-plan/plan/units/<id>.json`. The human-readable guide is `docs/guide/`; `CONTRIBUTING-RUST.md` is the contributor process.

- Check active claims by symbols and paths before editing shared contracts; keep each PR to one bounded unit.
- Preserve formulas, operation order, defaults, input guards and output meaning. Document missing continuations explicitly; an early return is not a ported body.
- Every numerical or kernel port compares original against Rust on stable inputs with case IDs and seeds, on values and state, not only error status. Missing original results and zero cases cannot pass.
- Never feed original outputs into the Rust run. Never loosen a tolerance, drop a passing comparison, trim seeds, or regenerate expected values from Rust to get green.
- User-visible operations are engine commands; the UI is thin. Native and Wasm both, bounded work, safe errors, undo and project compatibility are part of a feature.
- Rust only for application, numerics and geometry; foreign kernels are reference tools. A Rust wrapper around C/C++ is not a Rust kernel.
- Edit only the affected unit files; add unique files under `plan/findings/` and `plan/decisions/`. `backlog.json` is generated (`python docs/port-plan/tools/port_plan.py --root docs/port-plan render`).
- Public actions (messages, pushes, deployments, legal signatures, contacting the upstream author) need the owner's explicit say-so.

Gates: `cd rust && cargo xtask ci` (fast gate, exists now); `cargo xtask diff` / `diff-ci` (parity gate, not yet built; do not claim parity without it). Toolchain: system Rust, no rustup (`plan/decisions/2026-10-31-no-rustup.md`); Node is only for the original oracle.

Several agents at once: `docs/port-plan/AGENT-OPERATIONS.md` (branch from `main`, claim with a draft PR, lanes, merge protocol). Branches, the frozen oracle worktree and how upstream updates are merged: `docs/port-plan/UPSTREAM-SYNC.md`. `main` is the port trunk; `upstream-main` is a fast-forward-only mirror.
