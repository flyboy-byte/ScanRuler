<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 5. Working with AI agents on a long project

This project is built largely with AI coding agents, driven by one person, sometimes from a phone.
That works only if the project is written so that a fresh agent with no memory can pick up
mid-thread. These are the habits, with the reasons.

## 1. State lives in files, not in the chat

A conversation ends or gets compacted. The state document does not. Keep:

- **A handoff** (`docs/port-plan/HANDOFF.md`): authorisation, status, the immediate next steps.
- **One status record per work item** (`plan/units/*.json`), so there is exactly one place a status
  can be wrong.
- **Findings and decisions as separate, uniquely named files**, written once. Updates are new
  files, so history is never rewritten and nobody edits the same file at once.
- **A status line with an absolute date.** "Updated 2026-10-31" ages well. "Recently" does not.

## 2. Record why, not only what

A decision file says the old approach, the new one, the evidence, and who decided on what date.
Dead ends are kept so they are not explored again. Completed items are struck through, not
deleted.

## 3. Verify a handoff before acting on it

A handoff is a snapshot written somewhere else. On day one of this project, two of its blockers
turned out to be false on the real machine. The routine:

- Run the verification commands the handoff lists.
- Compare its environment claims with the machine.
- Treat anything it says about a past run as unproven unless an evidence file backs it.

## 4. Question recommendations; keep the ones that survive

An agent (or a document written by one) will say "install rustup" or "you need Node 22". Ask what
depends on it. The answers here were: Node is needed only for the oracle, and rustup would remove
a working toolchain and is not required. Both were checked against the packages and the plan, then
written down as decisions. The agent should say plainly when the user is right.

## 5. Tag claims by how strong the evidence is

Research notes in this workspace label claims `DOCUMENTED`, `REPORTED`, `INFERRED` or `UNKNOWN`.
Do not launder a guess into a fact. The port plan does the same thing in its own words: a static
count of 140 commands is "candidate commands, not a runtime-verified catalog".

## 6. A gate is not progress until it has failed once

Build the check first and show it can fail. The scaffold's pin check was proven by breaking the
constant. The comparison harness will be proven by feeding it a deliberately wrong Rust value.
A check that has never failed has not been shown to check anything.

## 7. Authorisation is specific

An agent may do local, reversible work freely (edit files, create a branch, run tests). It should
stop and ask for anything outward-facing or hard to undo: pushing, opening pull requests, messages
to anyone (including the original author), signing legal terms, deploying, deleting, installing
system packages that replace others. Approval for one action in one context does not carry to
the next. The plan states this in several places because it is the easiest rule to erode.

## 8. Be honest in the status line

"Builds and passes its own gate" is a different sentence from "matches the original". Use the
smaller one until the larger one is true. Skips are reported as skips. A compiling stub is not a
port.

## 9. Compaction paste-back

At the end of a long session, leave a block a fresh session can paste: which file to read first,
what to verify, what shipped, what is open, and a "do not re-raise" list of settled decisions.
Put the durable version in the state doc; the paste-back is the shortcut back in.

## 10. Feed lessons back

Generic lessons from this work (not names, not machine paths) go to the shared open-source
contribution workspace's lessons and playbook, which fold repeatable ones into a skill that every
future session loads. That is how one project makes the next one cheaper.

## Cross-checking with another model

Findings from another AI are input to verify, not fact. Keep their outputs as separate
artefacts instead of merging them into the plan.

## Reading port verification

The [verification guide](../port-plan/VERIFICATION.md) defines the explicit evidence
labels shown next to lifecycle states. Read the tested snapshots and scope before
interpreting fixed-corpus or fuzzed as a broader feature claim. Accepted requires
reviewed coverage; current acceptance remains zero.
