# Command history label expression

2026-11-03. PR #85 ports the complete expression shared by guarded and interrupt:
`Agent: ${command.label?.(input) ?? command.title.toLowerCase()}`. The original
builder verifies exactly both pinned occurrences and adds only a reference wrapper
containing that verbatim expression. It does not replace either command body.

Custom labels receive the exact command/input owners and run once. Empty text is
retained; only absent/nullish results fall back. Default title access is lazy, so
custom success or failure does not read it. Provider faults retain their original
value/identity and are not normalized here. Title/custom text and prefix preserve
all UTF-16 units, including unpaired surrogates.

Rust's full contextual Unicode lowercase operates on valid runs between unpaired
units. Those units are preserved and break casing context, as in the original.
No replacement character or ASCII substitute is used. Node22.23.4 reports Unicode
17.0/ICU78.3 and system Rust reports Unicode17.0.0; explicit gates reject an
unreviewed source/toolchain Unicode change. No new dependency or Unicode table is
vendored. The checked input bounds the standard lowercase temporary; this is not
process-wide OOM isolation. Fallible span/output copies and complete output limits
return separate host errors, with no partially transformed value published.

640 source-first comparison blocks cover every code point through U+10FFFF,
including isolated surrogate units, and a complete four-position sigma-context
census. Authored cases add combining marks, expanding lowercase, supplementary
letters, malformed surrogate runs, empty/control titles, all custom-label modes,
receiver/input identity, lazy access and exact callback/title faults. Original and
Rust construct inputs independently and compare the full actual input bytes first,
then complete output bytes, status/identity and every provider effect. Reference
results never enter the native computation. Appended title-fault and mixed-case custom probes preserve
all bytes of the initial616-block source profile. Full compressed source transcript,
source map/provenance and paired raw artifacts remain available.

The native gate streams the large reference transcript rather than holding it all
in memory; each case retains actual UTF-16 input/output and status/effect files.
The duplicate temporary transcript is removed only after all comparisons pass.
Native controls check Unicode version, context boundaries, lazy title access and
input/output refusals. All existing formatter/parser/session suites remain required.

Scope is owned title strings and typed custom/title providers. General JS host
objects/property access outside that boundary, full guarded/interrupt/history
execution, actual runtime wiring and browser-Wasm label execution remain open.
Wasm compilation is required; it does not grant runtime acceptance. No command,
M3 or accepted-unit claim. Run cargo xtask ci/oracle from rust/.
