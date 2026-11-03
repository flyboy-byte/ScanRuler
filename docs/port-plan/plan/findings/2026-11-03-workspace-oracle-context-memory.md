# Workspace oracle context memory

2026-11-03. Initial reference imported the full 3.4MB state/plugin bundle once for
each of 306 string/context rows. Node22 hit its default 2GB heap after 229 complete
records; that attempt is incomplete and cannot establish parity.

Use one isolated original module per 17 installed configurations and restore the
only mutated metadata field between the 18 strings. The original mapping/list
bodies read only this controlled metadata; every case compares the full installed
metadata witness. All prior captured prefix bytes are identical, recorded by hash
and byte count; all 306 cases pass. Do not increase heap/time limits or trim the
profile to address reference-process context accumulation. Complete source modules
remain unchanged; probes contain only explicit input metadata.
