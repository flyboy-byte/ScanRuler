<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Licensing, trademarks and the rename

> **PARKED (owner decision, 2026-11-02).** No rename, brand check, asset work or release preparation until the
> project is more mature **and** Stefan has replied to Logan's email about the SolidCraft integration. This file
> is a reference for that day, not a work queue. Do not create a unit for it, do not rename the remote, and do
> not contact Stefan; only Logan does. Until then the only standing rules are: keep every copyright/SPDX
> notice, add no new owner branding (logo, imprint, shop or donation links) to Rust code, and distribute
> nothing (no binaries, packages, deploys).

Written 2026-11-02 from the repository's own text. **This is an engineering reading, not legal advice.**
Claims carry an evidence tag: `DOCUMENTED` (quoted from a file in this repo), `INFERRED` (our reading),
`UNKNOWN` (needs the owner, the licensor or a lawyer). Nobody here contacts the licensor, signs the CLA or
publishes anything on the strength of this file; that stays with Logan.

## 1. What the repository says (all `DOCUMENTED`)

| Fact | Where |
| --- | --- |
| The code is **AGPL-3.0-only**. | `LICENSE`, SPDX headers |
| It is also sold under commercial exceptions; one owner (Stefan Hermann, CNC Kitchen) holds the right to license all of it, enforced through a CLA. | `COMMERCIAL.md`, `CONTRIBUTING.md` section 1, `CLA.md` |
| "The project name and the CNC Kitchen name and logo are trademarks and are **not** licensed under the AGPL. **Forks must use a different name.**" | `COMMERCIAL.md` Notes |
| Third-party parts keep their licences: fonts (SIL OFL 1.1), Open CASCADE (LGPL-2.1 + exception). | `COMMERCIAL.md`, `public/fonts/`, `CONTRIBUTING.md` |
| The sample scans (`ballbar.stl`, `block-marius.stl`, `side bracket left.stl`) are local fixtures, not committed and not covered by any licence here. | `COMMERCIAL.md` |
| Dependency policy: no copyleft code we do not own, and never port from CloudCompare, MeshLab or OpenSCAD. | `CONTRIBUTING.md` section 2 |
| A licensing enquiry from Logan to Stefan is pending; no reply is known. | `HANDOFF.md` |

## 2. What follows for the port

1. **The port is a derivative work under the AGPL (`INFERRED`).** It is read from and compared against
   Stefan's source, so it stays AGPL-3.0-only. We keep every existing copyright and SPDX notice, mark our
   changes, and offer complete source to anyone we give the program to or let use it over a network
   (AGPL section 13 matters for a hosted Wasm or web build).
2. **We cannot relicense it (`INFERRED`).** Permissive or proprietary use, including inside Tangency's
   permissive projects, needs a grant from the copyright holder. Until a reply exists, assume none.
3. **We hold no commercial-exception rights (`INFERRED`).** Those belong to the owner. Nothing in the port
   may be offered as "commercial licence available".
4. **Nothing in the licence text lets us use the marks (`DOCUMENTED`).** The AGPL permits a licensor to
   decline trademark rights, and `COMMERCIAL.md` does so. Source-code freedom and name use are separate.
5. **Not clean-room (`DOCUMENTED` in `HANDOFF.md`).** Never describe the port or any test derived from the
   source as clean-room.
6. **The CLA does not bind us.** It applies to contributions sent to the licensor. There is no upstream PR
   plan, so nothing to sign. If that ever changes it is Logan's decision alone.
7. **Third-party obligations carry over:** fonts keep OFL text if we ship them; Open CASCADE stays LGPL
   only while we use it. The pure-Rust endpoint removes it, and the dependency audit (P35) proves that.

## 3. The rename: what "different name" has to cover

"Forks must use a different name" and the owner's marks (the project name, "CNC Kitchen", the logo)
leave three groups. The measured counts are from `git grep` on 2026-11-02.

| Group | What | Rule |
| --- | --- | --- |
| A. Factual references to the original | Docs, evidence, test names, the frozen tag `reference/2cd89c6`, fixture names, `.oracle`, the oracle comparisons, the `mcp/` and TypeScript trees we keep as the reference | **Keep.** Accurate, descriptive mention of the original program is not using the name as ours. Say "the original, ScanRuler by CNC Kitchen" where it needs naming. Do not stylise it. |
| B. Our product's identity | Binary `scanruler-rs`, banners and CLI text, the README title and front page, window and page titles, MCP server and tool-set naming, package and crate metadata, the repository URL and name, wasm export prefix `scanruler_wasm_*`, any release artefact or domain | **Must change before anything is distributed, hosted or released.** Public repo and README are the exposure today (see section 4). |
| C. Owner's branding and personal material | `docs/brand/*` and `public/scanruler-*.svg`, favicon, the "by CNC Kitchen" title, Imprint page (Stefan's legal address), the support card with his shop, PayPal and Ko-fi links, `StatusStrip` link, the "CNC Kitchen blue" labelling | **Do not ship in the port.** Replace with our own. The brand colour is a design token, not a mark, but it is better not to match the look exactly (`UNKNOWN` whether it matters). |

Reference counts: the name appears in about 300 tracked files (docs 74, plugins 68, src 38, mcp 37, scripts 36,
tests 16, rust 7, .github 7). Most are group A. Group B in Rust is about seven files today and grows with every
crate that prints or exports the name, so build a single seam now (section 5).

### Compatibility identifiers (decision needed, `UNKNOWN` legally)

Some identifiers are functional, not branding: the **`.scanruler` project extension**, the MCP package name
`scanruler-mcp` and the legacy wire format, `schemaVersion` keys. P16 and P18 require reading and (for the
wire) speaking the legacy forms. Proposed rule: **read and accept the legacy identifiers forever; write our own
name for anything new**, and let the owner or a lawyer say whether even that is acceptable. Record the answer
in a decision file before P16 or P18 is accepted.

## 4. Where we stand today (honest status)

- `flyboy-byte/scanruler` is a GitHub fork, so it keeps the name by construction, and its README now carries a
  "ScanRuler in pure Rust" front page. A public repository with that title is the part most likely to read as
  using the name as ours. **Rename the repository and retitle the README before the repo is advertised or
  anything is released.** Renaming a repo is an outward-facing action, so it waits for Logan.
- Nothing is distributed: no binary, deploy, crates.io or npm package, domain or installer exists. Do not
  create any until the rename work (parked, section 5) is done.
- Rust identifiers use neutral `port-*` crate names already; only the CLI and wasm export carry the old name.
- No donation, imprint or shop link has been copied into Rust code.

## 5. Plan for later (parked; no unit exists)

1. **Name seam first.** One constant for the product name, binary name and wasm export prefix in `port-core`,
   used everywhere product-facing. A rename then is one edit plus a re-run of the brand check.
2. **Working codename now, final name later.** Use a neutral internal codename in new Rust surfaces; choose the
   public name when distribution is near. Pick something with no overlap with CNC Kitchen or "ScanRuler"
   (do a trademark-search step before committing to it; `UNKNOWN`).
3. **Brand check in the gate (`cargo xtask brand`).** Fails if product-facing Rust, UI, asset or metadata
   files contain the owner's marks outside an allowlist of group-A documents. Prove it by letting it fail once.
4. **Assets.** Our own logo, favicon and colours; delete nothing from the original tree (it is the oracle) and
   never copy `docs/brand` or `public/scanruler-*` into the port.
5. **Replace the owner-specific screens** (imprint, support card, links) with ours or remove them.
6. **Notices.** A top-level `NOTICE`/credits file stating that the work is derived from ScanRuler by Stefan
   Hermann (CNC Kitchen), AGPL-3.0-only, with the licence text and a source offer.
7. **Fonts and kernels.** Keep OFL text if fonts ship; otherwise choose different fonts. The P35 audit proves
   no LGPL kernel remains in the pure-Rust build.

## 5a. Name choice (owner decision, 2026-11-03)

**RulerCraft** is the chosen working name for the eventual rebrand. **Nothing changes until the project is
more mature**: no rename of the repository, crates, binary, wasm exports, README title or any file, and the
section 5 parking rules above stay in force. This records the choice only.

Why: a web and GitHub search on 2026-11-03 found no product or repository using it. `MeshCraft` was set
aside because an AI mesh-generation research project and several repositories already use it; `GaugeCraft`
(one empty repository) is the fallback. That search is not trademark clearance: before use, search the
USPTO database for the exact mark in classes 9 and 42 and consider a lawyer's read (section 6, question 5).
The `*craft` pattern also echoes SolidCraft, so check with its owners before presenting it as part of that
family.

## 6. Open questions only an owner or a lawyer can close

1. Will Stefan grant anything beyond the AGPL, or accept the port, and under what name terms? Logan's
   enquiry is the channel; no one else contacts him.
2. May the port read the `.scanruler` format and call itself compatible, and may it write that extension?
3. Is a descriptive "port of ScanRuler" in the README or credits acceptable after the rename?
4. Does Tangency need a different arrangement than the standalone fork? (`HANDOFF.md`: policy not imposed on
   Logan's own fork.)
5. Before the first public release, is a lawyer's read worth the cost? Recommended for the name and the
   compatibility questions, not for the AGPL basics.
6. Keep the AGPL port, build an independent implementation for a permissive licence, or seek permission?
   An outside review (ChatGPT, 2026-11-04, recorded with Claude's notes in
   `plan/findings/2026-11-04-licensing-review-chatgpt.md`) argues section 2's reasoning is too categorical
   and that the timing matters if Tangency integration is the goal. Recorded only; still parked.

Nothing in this file authorises a rename of the remote, a release, or any contact. It is the checklist for when
Logan decides to, after Stefan's reply.
