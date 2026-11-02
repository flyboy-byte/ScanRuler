# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

## Rust port (in progress; `main` is the trunk)

The fork is being ported to pure Rust (kernels included). Start with `docs/port-plan/HANDOFF.md`, then `AGENT-OPERATIONS.md` (lanes, claims, merge protocol), `PURE-RUST.md` and `PLAN.md`. Library ports of STL, mesh graph and plane fitting are merged (PRs #1–#24) but no unit is accepted and there is no Rust engine, UI or project reader yet. Baseline is frozen at `2cd89c6` (app 0.9.2); the TS app stays runnable as the comparison oracle and Rust work goes under `rust/`. Per-unit status lives in `docs/port-plan/plan/units/*.json` (the backlog is generated). Check with `python docs/port-plan/tools/port_plan.py --root docs/port-plan check` and `... selftest`. The port is AGPL-derived, not clean-room. Don't message Stefan, sign the CLA, push to `main`, merge, or deploy without Logan saying so.
