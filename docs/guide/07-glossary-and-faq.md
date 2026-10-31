<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 7. Glossary and FAQ

## Glossary

| Term | Meaning here |
|------|--------------|
| Port | Re-implement existing software in another language while preserving behaviour |
| Pure Rust | No C or C++ geometry code in the shipped product, even through bindings or WebAssembly |
| Oracle / reference | The original TypeScript app at commit `2cd89c6`; the judge of correct behaviour |
| Baseline / pin | The one commit being ported; recorded in `docs/port-plan/reference-pin.json` |
| Differential test | Run original and port on the same input and compare outputs |
| Vertical slice | One thin feature through every layer, end to end |
| Unit | One tracked work item (`plan/units/<id>.json`); P00 to P39 are the starting ones |
| Gate (G0 to G7) | A milestone passed by doing the thing it names |
| R1 / R2 | Application parity preview / the true pure-Rust endpoint |
| Ratchet | A rule that coverage may only grow: no passing case may vanish or weaken |
| Deferred | A case that reaches something not yet ported. Counts as neither pass nor fail |
| Kernel | The exact-geometry engine (solids, surfaces, booleans). Open CASCADE in the original |
| B-rep | Boundary representation: a solid described by its faces, edges and vertices |
| STEP | The standard file format for exchanging exact CAD models |
| Mesh | A surface made of triangles, which is what a 3D scan produces |
| Fit | Finding the sphere, plane or cylinder that best matches part of a scan |
| Deviation | How far a scan is from the reference model, signed |
| WebAssembly (Wasm) | A way to run compiled code (here Rust) inside a web page |
| MCP | Model Context Protocol; how an AI agent drives the app |
| xtask | A Rust-project convention: project chores written as a Rust program, run with `cargo xtask` |
| Clean room | Re-implementing from a specification without having seen the original's source. This port is **not** that |
| AGPL | The copyleft licence ScanRuler uses. Offering a modified version over a network requires publishing its source |

## FAQ

**Is this a rewrite?** No. It is a port: same behaviour, new language. Redesigning comes after.

**Why not just call the C++ kernel from Rust?** That would be quicker, and it is allowed as a
temporary comparison aid. The chosen endpoint is all Rust, so a wrapper never counts as done.

**Why keep the TypeScript around?** Because it is the answer key. Deleting it early would
remove the only way to know the port is right.

**How long will it take?** Unknown, and the plan refuses to invent a date. The measurement engine
is the moderate part. The exact kernel and STEP handling are the large, uncertain part.

**Can I help?** Read `CONTRIBUTING-RUST.md`. Good first contributions: harnesses, fixtures and
analysis of one bounded operation. Work is claimed through a scoped draft PR so two people do
not do the same thing.

**Can I use this code in my own project?** Under the AGPL, yes, with its conditions. For
anything permissive or closed-source, you would need rights the owner does not currently have.

**Does the author know?** A licensing enquiry was sent. There is no reply on record. Nobody
should contact him on the project's behalf.

**What can be run today?**

```bash
cd rust
cargo xtask env    # print toolchain versions
cargo xtask ci     # pin check, plan checks, fmt, clippy, tests, wasm32 check
cargo run -p port-cli
```

**Why no rustup, and why no Node yet?** See the decisions in
`docs/port-plan/plan/decisions/`. Short version: rustup would replace the working system
toolchain and nothing needs it; Node is only needed once we run the original for comparison.

**Where do I record something I learned?** A reproducible divergence goes in `plan/findings/`.
A change of approach goes in `plan/decisions/`. A handoff for the next session uses
`docs/port-plan/templates/session-handoff.md`.
