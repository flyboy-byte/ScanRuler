# Exact three-decimal source number text

2026-11-03. PR #92 implements owned Number.toFixed(3), the exact intrinsic used
by both free/known diameter expressions in sphere heldRefusal. Those expressions
are verified against the pinned complete source before original evaluation. This
is a primitive number conversion, not a claim to execute the entire sphere
selection function or arbitrary boxed/prototype JS semantics.

For finite abs(x)<1e21, decompose the exact IEEE number as mantissa times a power
of two. Exact scaling by1000 is mantissa*125 times2^(exponent+3). Below the source
threshold the integer needs at most80 bits, so u128 carries the full rational
without floating multiplication. Divide by a power of two and choose the larger
integer on a positive tie, then apply the original sign. Standard Rust floating
formatting's midpoint choice is not used. For example 0.0625 becomes0.063 and
-0.0625 becomes-0.063. Actual negative zero becomes0.000; a negative nonzero that
rounds to zero keeps its sign. Subnormals are handled directly. Nonfinite values
and abs>=1e21 use the existing source-compatible number_string helper unchanged.
Output is bounded to26 ASCII bytes; ordinary small String allocations are not
process-wide OOM isolation. No new dependency or change to old number formatting.

8492 source-first cases comprise authored signs/subnormals/special payloads,
exact ties and adjacent IEEE values, decimal/exponent thresholds and8192 random
bit patterns under seeds1/7/99/1039. Original and Rust consume the same authored
primitive bits; input bytes and primitive categories (NaN/inf/-inf/-0/finite) are
checked, as are complete output bytes and unchanged input. NaN payloads remain
input data, not distinguishable formatted values. No numerical tolerance or
candidate-generated expected strings is involved. Generator, complete source
transcript, pinned call-site source/provenance and full paired artifacts remain.

Native/Wasm compilation and all existing source suites remain required. Actual
browser fixed3 execution, marked-sphere selection and full measurement/command/
session/CLI integration remain open. This helper is not an accepted feature or M3.
