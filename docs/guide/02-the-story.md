<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 2. The story so far

A dated account of how this started. Dates are absolute because "last week" means nothing to the
next reader. When the story and a file disagree, the file wins; fix the story.

## Before 2026-10-31

The fork `flyboy-byte/ScanRuler` was taken from the author's repository
(`CNCKitchen/ScanRuler`). In an earlier planning session (with an AI assistant, in a sandbox
that could read GitHub through a connector but could not clone), a large plan packet was
written: 40 work units, an inventory mapping every one of the repository's 1,246 files to a
proposed disposition, a ledger of 140 candidate agent commands and 27 feature types, and long
documents on architecture, validation, CI and deployment. A few documents there were refined as
the user clarified their goals, most importantly: **pure Rust throughout, kernels included**.

That packet is a plan and an evidence inventory. It contains no Rust, no prototype, no benchmark
and no test results.

## 2026-10-31: setting up

1. **A normal first step: describe the code.** An AI agent was asked to read the repository and
   write the `CLAUDE.md` that future sessions start from: the commands, the architecture, the
   rules that bite (plugin boundary, unit handling, licence policy for dependencies, release
   steps).
2. **The plan arrives as a zip.** The packet was dropped into the folder. It was unzipped into a
   scratch folder first (an archive is untrusted until read), its integrity check and its own
   plan checker passed, and then it was placed at `docs/port-plan/`.
3. **The baseline was confirmed, not assumed.** The packet pins commit `2cd89c6`. `git rev-parse
   HEAD` in the checkout printed exactly that. A port branch `port/rust-baseline` was created
   and the author's repository was added as the `upstream` remote.
4. **Two blockers in the plan were stale.** The packet said a full checkout was impossible (the
   sandbox's proxy was down) and that Rust was missing. On the real machine, the checkout was
   there and Rust 1.99.1 with a WebAssembly target was installed. The packet described its
   author's environment, not the owner's. Lesson: check a handoff's environment claims against the
   machine before acting on them (finding `2026-10-31-b01-stale`).
5. **Two reasonable-sounding recommendations were questioned and held up.**
   - *Node 22.* The plan asks for it. The owner asked why. Honest answer: only for the original
     app, which is the reference the Rust code is compared against. Writing and building Rust
     needs no Node at all. So it waits until the first comparison. (Decision
     `2026-10-31-node-is-oracle-only`.)
   - *rustup.* The plan suggests a pinned toolchain. The owner said installing rustup would remove
     the current setup and doubted it was needed. Checked: on Arch the `rustup` package conflicts
     with the `rust` and `cargo` packages, and nothing in the plan needs it. The system toolchain
     already builds for WebAssembly. A sibling project (`solidcore`) works the same way. (Decision
     `2026-10-31-no-rustup`.)
6. **The workspace was scaffolded.** `rust/` holds four small crates and a task runner. Its first
   gate, `cargo xtask ci`, runs formatting, lints, tests, a WebAssembly build check and the plan
   checks, and it passed. A deliberate break (changing the pinned commit constant by one
   character) made it fail with a clear message, which is how you know a check can fail.
7. **Contributor scaffolding and these guides were written**, and a note was left for the shared
   open-source contribution workspace so its lessons and playbook can absorb what was learned.

## What the story teaches

- Plans age. A plan written in one environment is a hypothesis about another.
- "The tool recommends X" is not a reason to do X. Ask what depends on it.
- The first gate is built before the first feature, and it is shown to fail.
- Nothing is claimed that has not been run. The honest status line at the top of the guide index
  matters more than any milestone table.

## What happens next

Capture the original's real behaviour (install Node 22, run its tests, record what it skips), then
build the first vertical slice: read a binary STL, fit a plane to a named selection, and compare
the result with the original, natively and in the browser. See page 3.
