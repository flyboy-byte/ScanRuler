<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Outside licensing review (ChatGPT) — 2026-11-04

Recorded for later, at Logan's request. **Licensing stays parked** (`LICENSING-AND-NAMING.md` section 5):
nothing here changes a licence, header, porting rule or the repository. This is another model's output,
pasted by Logan; it is input to verify, not fact, and not legal advice. Its legal citations were not
checked (`REPORTED`).

## Claude's reading (2026-11-04)

- The review's central point holds and matches our own docs: access to the original is evidence about
  copying, not proof of it, but our process (read the original, port the whole body, keep formulas,
  operation order and rounding) is a source-informed translation by design. That is the risk, not the
  differential testing.
- `LICENSING-AND-NAMING.md` section 2 states "access plus comparison means the Rust code must keep the
  AGPL" more categorically than the law requires. Keeping the AGPL is still the safe default and the
  current intent; only the reasoning is too strong.
- Provenance differs by unit, which matters for the review's second and third paths. Examples
  (`INFERRED`, not a review):
  - translated from the original's TypeScript: the STL, PLY, OBJ and 3MF readers, the engine and session work;
  - reproducing a third-party MIT library's behaviour: `io.inflate` (fflate);
  - black box, no ScanRuler code to translate: `io.obj.companions.images.png` (the original only calls the
    browser; rules were derived from Chrome's outputs and confirmed against the MIT/Apache `fdeflate`/`png`
    crates).
- A cheap step if and when Logan wants it: a provenance field per unit (translated, black-box,
  third-party), so a later code-level review does not have to reconstruct it. Not started.

## The review, verbatim

> I found your flyboy-byte/ScanRuler repository, which ports CNC Kitchen's ScanRuler from TypeScript to
> Rust.
>
> Your argument is stronger than the repository's current licensing description suggests.
>
> The README explicitly calls the project "AGPL-derived" and "not a clean-room reimplementation." But it
> also describes a separate Rust implementation whose correctness is verified against a frozen TypeScript
> reference.
>
> Those are two different legal questions: whether the repository contains AGPL-covered material, and
> whether your independently written Rust code is itself a derivative work.
>
> Keeping the original AGPL code in the repository doesn't automatically make every separate Rust
> implementation AGPL-covered. However, it does mean you are distributing the original copyrighted
> material, and its license requirements still apply to that material.
>
> The strongest evidence against treating your current port as independently licensed is in its
> development instructions.
>
> Your `docs/port-plan/HELPING.md` tells contributors to read the original source for each unit and then
> port the complete implementation. `PORTING-PROCESS.md` specifically requires preserving formulas,
> operation order, clipping rules, and rounding behavior.
>
> That is more than black-box regression testing. It is a source-informed translation process.
>
> It does not conclusively establish copyright infringement or derivative status, because mathematical
> formulas, algorithms, and functional requirements are not protected as such (uscode.house.gov). But it
> creates a meaningful risk that protected implementation details are being translated, rather than
> independently recreated.
>
> Your existing `rust/Cargo.toml` also declares `AGPL-3.0-only`. That is an intentional licensing choice
> recorded in the repository, not proof that copyright law independently requires it.
>
> I also found an important assumption in `docs/port-plan/LICENSING-AND-NAMING.md`. It concludes that
> because the Rust implementation was written with access to the source and compared against it, the
> implementation must retain the AGPL.
>
> That reasoning is too categorical. Access to source code is evidence relevant to copying, but it doesn't
> prove that protected expression was actually copied. The 2023 U.S. decision in SAS Institute v. World
> Programming reinforces the distinction between copyrightable expression and unprotected software
> functionality (law.justia.com).
>
> For RulerCraft, I see three practical paths:
>
> - Keep the existing AGPL port. Lowest legal uncertainty and no disruption to your current development
>   process.
> - Build a genuinely independent Rust implementation. Use functional specifications, documented
>   mathematical requirements, and black-box regression testing. Keep the implementation separate from the
>   original source and prohibit source-based translations. This is your strongest route toward MIT or
>   Apache licensing.
> - Obtain separate permission from CNC Kitchen. This could allow alternative licensing of your existing
>   source-informed port without restarting development.
>
> The second option doesn't necessarily mean throwing away everything you've done. But existing Rust code
> would need a provenance review, and portions produced by translating the original implementation might
> need independent replacement. Simply splitting the repository or changing SPDX headers wouldn't
> establish independence.
>
> My assessment is that your regression-testing architecture is already well suited to an independent
> implementation. The problem is the source-informed porting instructions, not the regression comparisons
> themselves.
>
> If your objective is eventually to integrate RulerCraft into Tangency's permissively licensed ecosystem,
> I would resolve this now rather than after completing the Rust geometry kernel. The more source-informed
> code accumulates, the harder an independent licensing claim becomes.
>
> The AGPL is not inherently inherited by a new implementation. But based on the actual development rules
> in your repository, I would not currently represent RulerCraft as clean-room or confidently relicense the
> existing Rust implementation without a code-level review.
