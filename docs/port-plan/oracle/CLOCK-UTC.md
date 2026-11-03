# Own Date clipping and UTC timestamp conversion

2026-11-02. PR #68 replaces the supplied ISO formatting boundary with own Rust
Date millisecond clipping and Gregorian UTC conversion. It rejects non-finite or
outside-TimeClip inputs before truncating fractional milliseconds toward zero.
Day division floors; clock remainders are nonnegative. Astronomical year0 and
negative years use the same proleptic Gregorian calendar. Years0..9999 use four
digits; other years use a sign and six digits. Timestamp storage is bounded27
UTF-16 units and reserved fallibly. No calendar crate, OS calendar, foreign code
or lifted algorithm is used. Own400-year periods bound iteration to400 years and
12 months; there is no unbounded search over the input time magnitude.

311 source-first inputs compare direct real Date.toISOString results plus six
full original mark/get/fallback/invalid/reset states each. They cover all Date
range endpoints, negative fractions/signed zero, NaN/infinities, century/leap
boundaries, years0/1/9999/10000, wide integer times and raw float bits under
seeds1/7/99/1039. Full timestamp strings, RangeError name/message, clock counts
and page state match exactly. Error allocations/host limits remain distinct.
The source runs complete pinned types/stalePage modules via the retained #60
builder/proof. Only its clock input is injected; actual Date.toISOString converts.
Inputs are authored/generated independently before Rust and never use source
answers. Full raw input/original/Rust states and source-only capture are kept.

PageStale.mark_millis converts lazily: an already marked page ignores a later
invalid time, exactly as the original nullish assignment. Failed conversion or
host text admission leaves an unmarked page unchanged. The old generic clock
supplier API and every old clock/error/normalization case remain required.

Actual OS current-time acquisition, browser clock wiring and full registry
execution remain separate. Native Date behavior is captured with Node22;
actual Chrome-Wasm UTC execution remains required. Native roster on this stack:
36 suites /4,202 identities. Run cargo xtask ci and cargo xtask oracle in rust/.
Wasm compilation is required, not complete runtime or session acceptance.
