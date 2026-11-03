# Decision: 2026-11-03-own-inflate-zip-png-jpeg

- Affected units, contracts and active contributions: `io.threemf`, new `io.inflate`, `io.obj.companions.images`, P15 (flat-scan images), P35 (purity audit).
- Existing plan/behavior: `2026-11-02-pure-rust-dependency-default` allows our own code or an audited pure-Rust crate. The original's 3MF reader unzips with `fflate.unzipSync` (filtered to `.model`, `.rels`, `.config` names) and fails with one message when unzipping throws (`src/core/parsers/threemf.ts`, lines 392 to 396). Image companions are decoded by the browser (Chrome), so the reference for pixels is Chrome's decoder.
- Proposed change and reason: write our own code, no new dependency.
  1. `io.inflate`: a DEFLATE decoder and a ZIP central-directory reader with strict size and ratio limits. It is compared against `fflate`'s results on a generated and adversarial corpus (bombs, bad offsets, truncation, overlapping entries, odd flags). Where `fflate` is lenient or strict in a way a real file could hit, we match `fflate`, not the ZIP specification. 3MF and PNG both use it.
  2. PNG: our own decoder on top of that inflate, compared pixel for pixel with Chrome.
  3. JPEG: our own decoder, ported from the libjpeg algorithms Chrome uses (IDCT and upsampling variants chosen to match Chrome's output), last in order because it is the largest. The only off-the-shelf candidate audited so far (`image` 0.25.10) matched Chrome on 8 of 38 inputs, so matching pixels needs a faithful port, not adoption.
  - Order: `io.inflate`, then `io.threemf` and PNG, then JPEG.
- Evidence, reproduction and tested commits: `plan/findings` image-codec audit (28 pixel mismatches and 2 presence mismatches on 56 original inputs for the candidate); the original's use of `fflate` at the lines above. No code exists yet.
- Alternatives and compatibility impact: adopt an audited pure-Rust crate per codec (rejected for now: the one tried failed parity, and inflate bugs are the classic hostile-input risk, so we want to own and fuzz it); write nothing and drop 3MF and textures (rejected: part of the original's behaviour). No project-format impact.
- Required tests/migrations and follow-up units: hostile-input corpus per codec compared with the original's outcome; `cargo xtask fuzz` extended to the new parsers; P35 audits the final tree. New unit `io.inflate`; `io.threemf` depends on it.
- Owner/reviewer decision and date: proposed by Claude and approved by Logan's instruction to finish the port, 2026-11-03; revisable by Logan.
