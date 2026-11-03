# Upstream drift ledger: 0.9.2 (pin) to 0.9.4

2026-11-03. First entry in the drift ledger that [UPSTREAM-SYNC.md](../../UPSTREAM-SYNC.md)
asks for. **Owner decision, 2026-11-03: no rebaseline for now.** The reference stays
`2cd89c6769a10c27ee7754b477ada521c9ef5cb7` (app 0.9.2, mcp 0.5.0). This note records what
the pin is missing so a later rebaseline, or a decision to keep waiting, starts from facts.
Guidance for when the pin is much older is in the last section of UPSTREAM-SYNC.md.

## Window

Upstream `CNCKitchen/ScanRuler` `main` at `5c1f2f9` (2026-11-01), compared with the pin
(the pin is an ancestor of it). Upstream publishes one bot commit of the whole tree per
release, so these three commits are the whole window:

| Commit | UTC | What | Size |
| --- | --- | --- | --- |
| `a15bd37` | 2026-11-01 10:58 | ScanRuler 0.9.3 | 54 files, +994/-137 |
| `f183e9d` | 2026-11-01 11:48 | ScanRuler 0.9.4 | 28 files, +1111/-152 |
| `5c1f2f9` | 2026-11-01 12:59 | "Update ScanRuler": contributor-credit publishing | 4 files, +221 |

Together: 76 files, +2,319/-282. Method: full clone of upstream, `git diff` against the pin,
the maintainer's CHANGELOG entries, and a read of every diff under `src/core`, `src/commands`,
`mcp`, `plugins/reverse/{core,state,app}` and the tests. The viewer and UI diffs
(`src/viewer/*`, `src/ui/*`, `src/styles.css`) were taken from the file list and changelog
only, not read line by line.

## What did not change

Checked against the diff, not assumed.

- **Root dependencies.** `package.json` changed only its version and three new `e2e:*`
  script entries; `package-lock.json` changed only its two version fields. The pinned
  `fflate` 0.8.3 (the DEFLATE/ZIP port reproduces its `unzipSync`), `meshstep` and
  `occt-wasm` are unchanged.
- **Project format.** No file with `project` in its path changed, and `PROJECT_SCHEMA`
  (`src/app/project.ts`) is untouched.
- **Command list.** `mcp/src/commands.json` changed only its `app` and `saved` stamp lines.
  This does not cover result shapes: the notes change below adds fields to results.

## Ledger

Kind: **security** (hostile input, prompt injection, data written outside intent), **fix**
(wrong or hung behaviour), **limit** (a browser limit), **feature**, **process**.
Disposition is a proposal; none is scheduled.

| # | Release | Change | Kind | Relevance to the port | Proposed disposition |
| --- | --- | --- | --- | --- | --- |
| D1 | 0.9.4 | ASCII STL, ASCII PLY and OBJ are read in 16 MiB pieces (`parsers/text.ts`, `textChunks.ts`), so files past about 512 MB open. Chrome's `TextDecoder` silently returns `""` past its string limit, which showed as "no triangles". | limit | The browser string limit does not exist for native Rust reading bytes. No difference was found on valid inputs (see the fuzz section). Malformed ASCII PLY behaves differently (D1a). | Defer. Re-run the PLY comparison on malformed lists at rebaseline. |
| D1a | 0.9.4 | ASCII PLY list handling: for a negative or non-numeric list count the pinned parser does `ti += n`, which moves its token cursor backwards (or to NaN); the new one skips forward. | fix | The Rust PLY port was built and fuzzed against the pin, so it is expected to carry the pinned behaviour; that was not re-checked here. | Defer. Record as a known quirk. Do not change the Rust to match 0.9.4 without a rebaseline. |
| D2 | 0.9.3 | 3MF whose components multiply without end is refused instead of exhausting memory: past 1,000,000 placed objects or 30,000,000 placed vertices (`MAX_PLACED` in `threemf.ts`). | security | `io.threemf` (#58). The Rust port has caller-set budgets for vertices, triangles and work, so it can absorb this, but its refusal is a different error from upstream's. A different memory problem (aggregate part budget during decompression) is open on #58 and is not what upstream fixed. | Backport as a deliberate divergence before anything opens untrusted 3MF. |
| D3 | 0.9.4 | 3MF and STEP files too large for one string say so ("too large", with what to do) instead of a false message. | limit | Browser-specific. | Map into the host-limit class; no behaviour to port. |
| D4 | 0.9.3 | A mesh-worker request that throws while the queue drains is now answered with an error (`answer()` in `meshWorker.ts`). Before, the throw reached no one and the request and everything queued behind it waited forever. | fix | Requirement for the Rust executor/worker integration still to be built: every queued request must get a reply, including on failure. | Carry into the executor design. |
| D5 | 0.9.3 | An agent command's answer goes only on the connection it arrived on (`bridge.ts`); a server restarted since counts ids from 1 again and could take another request's answer. | security | MCP server (M5). | Carry into the Rust MCP design. |
| D6 | 0.9.3 | Agent server (scanruler-mcp 0.5.0 to 0.6.1): a connection may send 16 MB before its hello (was 1 GB), at most eight waiting; an oversized or broken frame no longer ends the server; malformed URL path gets 400 and a failed request 500 instead of crashing `--serve`; binary paths such as `__proto__`, `constructor`, `prototype` are refused in page and server; the lockfile takes patched versions of five reported dependency vulnerabilities. | security | MCP server (M5). Also affects anyone running the TypeScript MCP server from this fork today. | Carry into the Rust MCP design. Consider pulling the TS fixes if the TS server is in daily use. |
| D7 | 0.9.3 | `scanruler_write` refuses hidden folders and agent/tool folders (`.claude`, `.git`, `.github`, `.cursor`, `.ssh`, `node_modules` and others) and files agents or tools act on (`CLAUDE.md`, `AGENTS.md`, `package.json`, `mcp.json`, `settings.json` and others); it judges a path by where links lead and handles Windows trailing dots and colons. | security | MCP server (M5). Relevant here because this repo's own agents read `CLAUDE.md` and `AGENTS.md`. | Carry into the Rust MCP design. |
| D8 | 0.9.3 | Reverse engineer: notes that come from a project file are marked `fromFile`, listed apart, and the agent is told to treat their text as content, not instructions. The mark is session-only, never saved, and is cleared when the person keeps or answers the note. | security | Reverse workspace (M7), project round-trip. No project-format change. Result shape changes: `fromFile`, `treat`, and the open count excludes file notes. | Carry into the Reverse workspace unit. |
| D9 | 0.9.3, 0.9.4 | Notes key on the view bar; Command as the additive modifier on a Mac; small features stay a fixed size on screen on large scans; a sample scan on the start page. | feature | UI. None until the UI is ported. | Defer. The sample file (`public/samples/fridgeBracket_01-reduced.3mf`, 4 MB, 288,582 triangles) is not adopted as a fixture: no licence or provenance was found for it, and the plan requires both for every fixture. |
| D10 | 0.9.4 | `reNetworkCreases.test.ts`: the fixed 1500 ms assertion became relative: best of three box reads must be under the whole-scan read divided by 2.5, with a 5 s ceiling and a 60 s test timeout. The upstream comment says the read takes 0.25 s on a desktop and 1.7 s on a CI runner. Upstream's `test-and-build` job budget went from 20 to 30 minutes. | fix | This is the flake in [2026-11-03-original-crease-timing-pr70.md](2026-11-03-original-crease-timing-pr70.md) and the red check on #61 and #58. The one-worker scheduling in #69 is a workaround; upstream fixed the cause. | Not adopted. The fork keeps the original unmodified and does not raise the limit. Adopting the test change is an owner decision, since it edits the oracle's own test. |
| D11 | credits commit | Contributor credit on publish (`scripts/oss-credits.mjs`, `tests/ossCredits.test.ts`, CONTRIBUTING.md "How you are credited"): a pull request is applied in the development repository and closed here, and the release commit names the contributor as co-author. | process | None for the code. Relevant to the licensing and contribution questions parked in LICENSING-AND-NAMING.md. | None. |
| D12 | 0.9.3, 0.9.4 | New e2e scripts (`e2e:sample`, `e2e:small-feature`, `e2e:large-text`); the plugin-boundary check also reads `src/plugins/`; setup steps install the MCP package's dependencies. | process | Oracle tooling only. | None until a rebaseline. |

## Differential check of the three ASCII parsers

Claim under test: the 0.9.4 rewrites of `parseSTL`, `parsePLY` and `parseOBJ` return what
0.9.2 returns. Both source trees were extracted with `git archive` into separate
directories and run side by side on generated inputs. The harness, its configuration and
the raw log are in [evidence/upstream-drift-2026-11-03/](../../evidence/upstream-drift-2026-11-03/).
Seeds are fixed, so the runs repeat exactly.

| Run | Format | Inputs | Same result, ok | Same result, refused | Different |
| --- | --- | ---: | ---: | ---: | ---: |
| A: hostile mix | STL | 20,000 | 8,242 | 11,758 | **0** |
| A | PLY | 20,000 | 4,914 | 14,803 | **283** |
| A | OBJ | 20,000 | 1,204 | 18,796 | **0** |
| B: mostly well-formed | STL | 40,000 | 17,639 | 22,361 | **0** |
| B | PLY | 40,000 | 31,041 | 8,721 | **238** |
| B | OBJ | 40,000 | 16,159 | 23,841 | **0** |

Every PLY difference is a case where both versions succeed with different triangles.
In the shortest differing input of each run the divergence starts at a negative or
non-numeric face list count, which matches the cursor difference in D1a. **That is not
proven to be the only cause**: the generator also produces garbled token streams
(for example whitespace that is not JavaScript `\s`, such as U+200B), and the 238
remaining differences in run B were not each traced.

What this does not show:

- The chunk-boundary logic (words, lines and multi-byte characters split across 16 MiB
  pieces) was **not exercised**: every input is far below one piece. A run with the piece
  size forced down to 37 bytes was prepared and not executed. It needs a modified copy
  of upstream's parser source, which the session's permission check blocked. Re-run it
  before relying on the new reader for large files.
- No binary STL or PLY, no real-world files, no 3MF, and OBJ run A is 94% refusals.
- The harness found no divergence in STL or OBJ, and in PLY only on malformed list data in
  the cases inspected. Absence of a difference in generated inputs does not establish
  equivalence.

## Not verified

- Which source tree the Node side of `cargo xtask oracle` reads. See "Do not merge
  upstream until the oracle is decoupled" in UPSTREAM-SYNC.md.
- Viewer and UI behaviour (D9), read from the changelog only.
- Whether the Rust 3MF port needs the object-count cap of D2 or is already covered by its
  `Work` budget.
