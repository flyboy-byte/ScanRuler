<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 4. Testing and CI

## What exists today — 2026-11-01

| Gate | Command / check | Current scope |
| --- | --- | --- |
| Plan/status | `cargo xtask plan` or plan workflow | Canonical metadata, generated freshness and negative tooling tests; not parity |
| Rust fast | `cargo xtask ci` | fmt/lints, ordinary native tests, pin/plan/status and Wasm compilation |
| Native source | `cargo xtask oracle` | All registered required suites; see the generated status inventory for case identities including refusals |
| Raw evidence | `ci_report.py evidence --candidate <sha>` | Full case/file inventory at exact checkout; not an independent numerical comparator |
| Original catalogs | validator tests and real capture workflows | Immutable core/all browser/headless schemas and context; not Rust commands |
| Original app | source-tests, source-mcp, source-browser | All original tests/build/browser checks; independent lanes and strict final gate |
| Browser-Wasm / generic diff / exact-base ratchet | P36/P37 | Not implemented; no green substitute |

Fast checks and actual native comparisons now have distinct results. The final Rust `gate` requires every Rust lane; original `test-and-build` requires all original lanes. Failed or skipped required lanes remain red. Source timing failures no longer block collecting independent MCP/browser results; no threshold is changed. Main validations are never cancelled by later commits. SHA-named raw evidence/log artifacts are retained thirty days. See [the executable CI contract](../port-plan/CI-AND-REGRESSIONS.md) for exact commands and limitations.

A green native gate is evidence for its fixed native corpus, not browser runtime or whole-product parity. [STATUS.md](../port-plan/STATUS.md) counts canonical inventory, while the [audit queue](../port-plan/AUDIT-QUEUE.md) records observed run outcomes. Neither can stand in for the other.

## How "matches the original" is proven

Each test case is a record: an immutable id, the unit and operation, an input (or the hash of an
input file), a random seed, the sequence of operations, and the list of outputs to compare. A
coordinator hands independent copies to two adapters, one driving the original and one driving
the Rust. Each emits structured data, never a summary someone guessed.

Every run records: the original's commit, the candidate commit, lockfiles, toolchains, the
fixture manifest digest, the seed schedule, the comparator version and the environment.

Outcomes are distinct states, so nothing hides: pass, mismatch, deferred or unported, unsupported,
skipped, reference failure, harness error, timeout, crash. Raw outputs from both sides are kept.

What gets compared depends on the operation: fits and their residuals, geometry plus topology and
volume, mutations plus resulting state and history, file formats plus what reopening them means,
UI plus the identities picked and numbers shown. Discrete things compare exactly. Numbers use a
per-field tolerance chosen up front from the original's tests and representation. Any
normalisation is versioned and limited to differences that are only representation.

## Fixtures

Tests need inputs with answers known independently of both programs: exact and noisy planes,
spheres, cylinders, cones and tori; boxes and brackets with known raised and sunken faces; scans
with a known rigid pose and a deliberate scale error (alignment must not absorb scale); broken,
inverted and duplicated meshes; thin walls. A fixture manifest records its licence, generator and
seed, hashes, units, frame, exact recipe, expected values and how they were derived, tolerances,
platforms and known limits. Generators produce redistributable data. The author's private
ball-bar and bracket scans remain optional; tests that need them report **skipped, never passed**.

## The ratchet

For a pull request, the candidate's reports are compared against validated reports for the
**exact target base commit**, with the same reference, inputs, seeds and comparator. Not an
arbitrary old green run. The comparison fails if any previously passing case disappears, stops
passing, or loses its compared values; if a unit, input or output key is dropped; if seeds
shrink; or if a pass becomes skip, deferred or "known difference". Error-only agreement never
substitutes for a successful value comparison.

Golden values are never "whatever the Rust produced". They come from the pinned original. A
change to a comparator, tolerance or generator is reviewed like a change to the test
infrastructure, because it can make broken code look correct.

## Gate tiers

| Tier | When | What |
|------|------|------|
| Planning bootstrap | Every PR, before code exists | Unit, dependency and schema checks, plus the checker's self-test; named as plan-only |
| Fast product gate | Every implementation PR | Format, lints, unit and regression tests, dependency checks, native tests, WebAssembly build, targeted comparison, callers of changed code |
| Merge gate | Before merging | Whole accepted-unit corpus, exact-base ratchet, browser project, history and agent-interface checks, no filtered comparison |
| Main validation | Every main commit | Reports keyed by commit; never cancelled by a later merge |
| Scheduled depth | On a budget | Bigger seeded corpus, fuzzing, mutation testing, large meshes, performance, wider platforms |
| Release | Explicit | Full native and web matrix, dependency purity audit, installers, legacy file acceptance |

Superseded PR runs may be cancelled. Cache dependencies, not unvalidated golden results.

## Testing the tests

A comparison harness can be wrong in a way that makes everything green. So it is tested too:

- It must reject a deliberately altered Rust value, a missing key, an empty suite, a skipped
  reference, a stale input hash and a changed case id. Change one thing at a time and show the
  comparator fails.
- Mutation testing in two layers: ordinary Rust tests first, then the differential tests. A
  mutation both layers miss points to a concrete gap; minimise the input and add an assertion.
  Equivalent or non-compiling mutations are classified, not counted as catches.
- The plan checker itself has a self-test that must reject nine kinds of invalid state, and the
  scaffold's pin check was shown to fail by editing the constant.

## Other acceptance areas

Hostile input (huge sizes, overflow, non-finite numbers, decompression bombs: the original caps
project archives at 16 members, 64 MiB of manifest and 1 GiB expanded); project files from old
versions; undo and recovery; the agent interface's pairing, jobs and file handling, tested old
helper against new page and new helper against old page; UI picks and displayed numbers; and
performance and memory budgets, including the peaks during undo and rebuild. `VALIDATION.md` has
the detail.
