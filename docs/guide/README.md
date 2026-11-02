<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Porting ScanRuler to Rust: a guide

Updated 2026-11-01 (original guide written 2026-10-31) for someone who is smart, curious and has not been in the room. It explains
what this side project is, how it is being done, why it is being done that way, and where to
look in the repository. It is meant to be read in order, and each page stands on its own.

| # | Page | You will learn |
|---|------|----------------|
| 1 | [What and why](01-what-and-why.md) | What ScanRuler is, what "port to pure Rust" means, what counts as done, and the licence facts that shape everything |
| 2 | [The story so far](02-the-story.md) | How the project got started, the decisions made on day one, and what was checked rather than believed |
| 3 | [Porting strategy](03-porting-strategy.md) | The original as an oracle, gates G0 to G7, the first vertical slice, the work unit life cycle |
| 4 | [Testing and CI](04-testing-and-ci.md) | How "it matches the original" is proven, the gate tiers, and testing the tests |
| 5 | [Working with AI agents](05-working-with-ai-agents.md) | Handoffs, state files, evidence, and the habits that keep a long AI-assisted project honest |
| 6 | [Learning from Tangency and Storytold](06-learning-from-tangency-and-storytold.md) | Two real Rust CAD projects, what each does well, what we borrow and what we do not |
| 7 | [Glossary and FAQ](07-glossary-and-faq.md) | Terms, and the questions a newcomer asks first |

## If you only have ten minutes

1. Read the first section of page 1.
2. Read "The one idea" in page 3.
3. Skim the table of gates in page 4.

## Where things live

| Path | What it is |
|------|------------|
| `docs/port-plan/` | The plan packet: `HANDOFF.md` (start here), `PLAN.md`, `ARCHITECTURE.md`, `VALIDATION.md`, `CI-AND-REGRESSIONS.md`, `PORTING-PROCESS.md`, `plan/units/*.json` (the status of every work item) |
| `rust/` | The Cargo workspace. Small on purpose; see its [README](../../rust/README.md) |
| `src/`, `plugins/`, `mcp/`, `tests/` | The original TypeScript app, kept running as the comparison oracle |
| `AGENTS.md`, `CLAUDE.md` | Instructions for AI coding agents working here |
| `CONTRIBUTING-RUST.md` | How a human contributor takes a piece of work |

## Honest status

As of 2026-11-01, bounded geometry/format operations are implemented and compared natively. The full mesh-graph library pipeline exists; the engine is empty, the CLI is a banner and no Rust UI exists. Actual Chrome-Wasm comparison now covers the TLS/mesh-graph/marked-plane subset; wider runtime coverage and UI/session parity remain open. No numerical unit is accepted. Original browser/headless catalog metadata is captured, not complete command execution parity.

Start at the [generated status](../port-plan/STATUS.md), [audit queue](../port-plan/AUDIT-QUEUE.md) and [next implementation increments](../port-plan/IMPLEMENTATION-PLAN.md). Historical scaffold-only statements in the story refer to 2026-10-31, not current implementation status.
