<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 3MF import against the original's `parse3MF` (2026-11-03)

`rust/crates/port-io/src/threemf.rs` ports `src/core/parsers/threemf.ts` body for body: the package
(zip read by `io.inflate`), `_rels/.rels` and the root model, the original's own XML scanner, units,
objects, components and build items with their transforms, mirrored-triangle correction, slicer parts
left out (Bambu/Orca `model_settings.config`, PrusaSlic3r `Slic3r_PE_model.config`), object types that
are not the part, colour groups, base materials, texture groups, and the final colour and texture note.
It takes already decoded companion images (`ObjCompanions`) exactly as the original does. No dependency
was added. `docs/port-plan/oracle/threemf.cjs` runs the original's `threemf.ts` and `objTexture.ts` from
the pinned git object (TypeScript erased with the original's own `typescript`) on the original's pinned
`fflate`; it refuses to run if the installed `fflate` differs from the lockfile.

## What is compared

For every package: success or failure; the **full error text** (including the `RangeError: Invalid code
point …` text); the **progress text** in order; **positions** (float32 bits), **indices**, **colours**,
and the **texture note** (kind and names, in order). Host-limit refusals and original timeouts are
counted apart and never as matches.

## Result

`pinned_threemf_comparison` (ignored; seeds 1, 7, 99, 1039, 4242, 31337; 4,000 generated packages per
seed plus 90 named hazard cases per seed, unmutated, so 24,540 comparisons):

| Comparisons | Identical success | Identical refusal | Host limit | Original timeout | Divergences |
| --- | --- | --- | --- | --- | --- |
| 24,540 | 10,254 | 14,267 | 19 | 0 | 0 |

Of the successes, 2,234 carry colours, 1,076 apply a texture and 1,882 note a missing one. Refusals seen,
all with identical text: unit not recognised, invalid transform, invalid vertex, a triangle corner that
names no vertex, invalid code point, could not be unzipped, no model, no triangles of a part, named
object or part not held, components naming each other in a ring, and the engine `TypeError` (below).

**Generated packages** use unit names (with case and bogus values), namespace prefixes, mesh objects
with optional types and property groups, component trees (some through a second model file by
production-extension path), build items, mirrored and rotated transforms, colour and texture groups
(with matching, mismatching and missing images), slicer metadata of both kinds, `_rels/.rels` variants,
XML decoration (comments, processing instructions, CDATA, quote styles, spacing, a BOM), duplicate and
re-cased part names, stored and deflated archives (the original's `zipSync` at levels 1, 6, 9).
**Mutation** (45% of packages) edits the XML (truncate, delete, duplicate, punctuation, bit flips,
attribute values replaced from an edge-value dictionary, tag names swapped, slashes removed, digits
replaced) and (12%) the zip bytes.

**Named cases** (always run): lone and paired surrogates through numeric entities (a lone surrogate id
matching itself, a pair equalling the literal character, reversed halves not), `&#x10FFFF;`, `&#0;`,
upper-case hex and named entities, `&#4294967296;`, the `Number()` grammar in coordinates and indices
(`0x10`, `1e1`, `Infinity`, `1_0`, Arabic-Indic digits, `0b11`, non-breaking space and BOM padding),
relationships whose first match has no `Target`, Unicode case folding of part paths (final sigma, dotted
capital I, the Kelvin sign, sharp s), a `DOCTYPE`, unquoted attributes, mis-nested `object` and `mesh`,
a second `mesh` in one object, comments and CDATA that look like elements, namespace-prefixed
attributes colliding with plain ones, transforms with the wrong count, `NaN`, hex, odd spacing and huge
values, all unit spellings including `constructor`, `__proto__`, `hasOwnProperty`, `toString`, an
inherited unit from the root, the same object placed twice, a component path naming the root, a ring,
invalid UTF-8 and a BOM, leading text before the root element, and slicer ranges (empty, inverted,
fractional, beyond the mesh).

**Real packages.** 107 real `.3mf` files on this machine (92 bundled with Cura, 13 SolidCraft slicer
exports, 2 of the owner's own designs) were read by both with `SCANRULER_3MF_REAL=<dirs>`: 107 identical,
0 divergences. Nothing was copied into the repository; a divergence would record the path only.

**The comparison can fail.** With `SCANRULER_3MF_SABOTAGE=1` (one bit of the first position flipped) the
same run reports a divergence in every successful package (88 of 200 in the first seed).

**Reach of the original.** V8 block coverage of the original's `threemf.ts` over 6,000 generated packages
(measured before the named cases were added, which only add reach): every block ran except three `??`
fallback expressions. This is reach, not data
coverage; it says the corpus drives the original through all of its branches.

## Behaviour reproduced on purpose

- JavaScript string semantics: text is UTF-16, `trim`/`\s` are the JavaScript whitespace set, `toLowerCase`
  is the Unicode one (final sigma, `İ`, Kelvin sign), numbers use the full `Number(text)` grammar.
- `String.fromCodePoint` throws `RangeError: Invalid code point N` for a numeric entity beyond U+10FFFF
  (N printed as JavaScript prints it, e.g. `1e+20`); a surrogate code point survives as a lone surrogate.
- `Object.prototype` reaches the original's plain-object lookups: `unit="constructor"` or `"__proto__"`
  is not `undefined`, so the scale is not a number and every position is NaN; a part path of
  `constructor` or `__proto__` is truthy and reaches `TextDecoder.decode` with a function, a `TypeError`.
  Other prototype names (`hasOwnProperty`, `toString`) are lower-cased first and so never match.
- `object?.components.push(…)` skips its argument, so a bad `transform` on a component outside any
  `object` is not an error; a bad `transform` on a build item always is.
- The original's `object`, `mesh` and `group` variables are followed exactly, so mis-nested or repeated
  elements append to the same objects they would there.
- Duplicate part names collapse after lower-casing (first position, last value); progress text is
  `Reading 3MF…` then `Placing the 3MF's objects…`, the second only after the root, rels and slicer
  files have been read.

## Classes reported apart

- **Host limit (19 of 24,540):** all `Archive`: a mutated zip header declares an entry over the test's
  16 MiB cap, and the original either reads the package (7), because `fflate` allocates the declared size
  lazily and returns only what was written, or fails for another reason first (12). This is the `io.inflate`
  per-entry cap, not a 3MF difference; the library default is the caller's choice. Lifting it needs an
  inflate that does not allocate the declared size (an owner decision with H1 to H3).
- **Work budget:** the original has no bound on component fan-out; a chain of 30 objects that each place
  the next twice runs for 2^30 placements. Rust stops at its work budget with `Limit(Work)` (tested in
  `a_component_fan_out_hits_the_work_budget`; the original is not run on it).
- **Engine text:** the `TypeError` message differs between Node and a browser; the comparison asserts the
  failure and its class, not Node's wording (`ThreeMfError::HostType`).
- **NaN payloads:** a positions NaN produced by `inf * 0` has a hardware-dependent sign and payload;
  the comparison treats every NaN as equal.

## Not done (open)

- Not in `ci/native-suites.json`: the hosted native-oracle job does not run it. It runs locally with
  `fnm exec --using=22 -- cargo test -p port-io --test threemf pinned -- --ignored --nocapture`
  (`SCANRULER_3MF_SEEDS`, `SCANRULER_3MF_CASES`, `SCANRULER_3MF_REAL`, `SCANRULER_3MF_SABOTAGE`,
  `SCANRULER_3MF_VERBOSE`). Registering needs a per-case artifact inventory; wiring it to
  `cargo xtask fuzz` touches `rust/xtask`, which Codex's open PRs also edit, so it waits for them.
- **Packed texture images** are not read here: the original's `threemf.ts` unzips only models, rels and
  configs; the images a 3MF packs are read by a second unzip in `src/app/scanFiles.ts` (filter
  `png|jpe?g`) and decoded in the app. They belong to `io.obj.companions.images` (own PNG decoder, then
  JPEG).
- No Chrome execution of the original's reader (Node 22 only).
- Real packages from other producers (Bambu Studio, Orca, PrusaSlicer, Microsoft 3D Builder, CAD
  exports) are not on this machine; the slicer-metadata and production-extension paths are covered by
  generated packages only.
