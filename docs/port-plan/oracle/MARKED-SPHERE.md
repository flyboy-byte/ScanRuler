# Complete marked-sphere continuation

2026-11-03. PR #93 depends on #89 and #92. It ports the complete
fitSphereOnSelection body plus heldRadius/heldLimit/heldTooFar/heldRefusal and
sphereOutput, with one shared requireSelection guard. Source min20 is checked
before geometry access or held settings. The existing plane caller reuses that
guard with its unchanged "surface" noun and exact previous message.

Only valid finite positive known diameters hold a radius, by dividing by two.
Invalid/nonpositive/nonfinite values are ignored exactly as before. A positive
diameter underflowing to radius0 reaches the original positive-radius refusal;
no new default/normalization is introduced. Known-size mismatch uses the source
strict threshold max(1mm,10% of held diameter). Both free/known diameter strings
use the separately compared exact fixed3 formatter, including binary ties and
the1e21 branch. Every source refusal remains FitError with its original name,
brand/constructor and complete message. Typed buffer/point quotas remain separate
host errors; arbitrary JS settings getters/boxed values are outside this boundary.

Successful outputs include sphere kind, centre/radius, optional held freeDiameter
and centre shift, sigma, clipped usedPoints, original regionSize, formError and
signed extremes. Region is the exact original shared selection owner/order,
including duplicates and outliers; clipping never replaces its recipe. MeshGraph
properties other than positions are not read by this source selection function.
No source-absent bbox/normal/noise/seed/growth heuristics are added.

2220 original-first profiles cover four seeds, exact/noisy/cap/outlier/degenerate/
nonfinite scans, min19/20/21 marking boundaries, duplicated/reversed IDs, invalid
and held/underflow diameters, exact decimal-tie refusals, and adjacent IEEE values
around both strict held-size thresholds.759 successful outputs and1461 FitError
refusals compare full actual inputs, every value/optional/count/region/identity,
exact error text/class and unchanged inputs. Scalars use the previously frozen
absolute1e-10+relative1e-10 budget; messages/discrete/nonfinite/zero signs are exact.
No candidate-generated expectations or tolerance/seed/case changes.

The reference loads the complete pinned dependency graph (including uncalled
seeded helpers) with only TypeScript syntax erased, no extracted bodies, external
dependencies or behavioral stubs. Original/transpiled source maps and whole sources,
compiler/runtime identities and complete source/paired artifacts are retained.
Native test input construction is shared with the existing sphere suite; every
previous1560 sphere input/comparison and all earlier plane cases remain mandatory.

Seeded/RANSAC/growth/guides and actual browser marked-sphere execution, engine
commands/session/history/project/CLI integration remain unported. This complete
library continuation is not accepted measurement/session/M3 functionality.
