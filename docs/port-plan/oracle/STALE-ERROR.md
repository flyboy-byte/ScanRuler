# Stale-page error construction

2026-11-02. PR #63 ports staleError and STALE_PAGE over owned UTF-16
messages and synchronous, fallible host clock/message suppliers.

The original marks the page before evaluating Error.message or String(value).
Clock failure skips message access; message access/coercion failure preserves a
successful mark. Existing page state prevents further clock sampling. The exact
uppercase-first-letter prefix, punctuation, parentheses, CommandError name and
stale_page code remain unchanged, including empty text and lone surrogates.
Host text/allocation failures are distinct and do not roll back a successful mark.

The full original modules are retained by #60, with exact source-map proof.
This adapter runs the whole staleError body. It injects only the platform Date
clock and explicit test objects for message getter/String coercion observations.
120 source-first cases compare full envelopes, page timestamps, clock/coercion
logs and thrown messages for text, Error, CommandError, custom objects/getters,
throwing conversion/getter, null, undefined and arrays. Every output matches.
Full independent inputs/source captures and lossless paired artifacts are kept.
No source answer is supplied to Rust; supplier text and faults come from inputs.

The product's general native String(value) conversion is not implemented here:
the caller supplies Error.message or converted text at the typed host boundary.
Registry asCommandError, actual clocks and browser preload wiring remain separate.
The complete session stays specified. No working user command is claimed.

Native roster on this stack:34 suites /3,587 identities. Run cargo xtask ci and
cargo xtask oracle in rust/. Wasm compilation is required; actual browser
stale-error execution remains open.
