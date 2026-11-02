# Decision: 2026-11-02-pure-rust-dependency-default

- Affected units, contracts and active contributions: `io.threemf` (zip/inflate), `io.obj` (PNG/JPEG decoding), P15 (flat-scan images), P35 (purity audit), every PR that would add a crate.
- Existing plan/behavior: the workspace has no third-party dependencies; each must pass the licence policy in `CONTRIBUTING.md` and the P35 purity audit.
- Proposed change and reason: keep that rule, and set the default for formats that need real codecs. Write small, security-sensitive logic ourselves. Otherwise a **pure-Rust** crate (no C or C++, including through build scripts or bundled sources) is allowed if it passes the licence policy and the P35 audit, one crate per PR, each with its own decision file naming the crate, version (exact pin), licence, transitive tree and what we compared it against. A crate that wraps C/C++ never counts toward the endpoint.
- Evidence, reproduction and tested commits: none yet. No candidate crate has been audited, so none is named here on purpose (`UNKNOWN`).
- Alternatives and compatibility impact: write every codec ourselves (slowest, more risk in inflate and image decoders); allow any crate with a Rust API (breaks the pure-Rust endpoint). Revisable by the owner.
- Required tests/migrations and follow-up units: each codec PR needs hostile-input tests (bombs, truncation, oversized dimensions) compared with the original's outcome; P35 audits the final tree.
- Owner/reviewer decision and date: default set by the session on the owner's instruction to do what is best for the dev environment, 2026-11-02; revisable by Logan.
