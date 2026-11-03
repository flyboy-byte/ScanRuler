# Use matching standard Unicode data for command labels

2026-11-03. Before choosing new tables/dependencies, inspect the real toolchains:
Node22.23.4 uses Unicode17.0/ICU78.3; system Rust uses Unicode17.0.0. Existing IO
helpers already use full standard lowercase on valid text. Extend this to owned
UTF-16 by preserving unpaired units as context boundaries. Verify the complete
code-point/context domain against the original, and gate profile changes instead
of silently accepting a new Unicode version or adopting ASCII/lossy behavior.

The license policy already permits ordinary Rust standard-library use. No new
dependency/table adoption is needed. Runtime/browser version coverage and actual
command/history integration remain separate; none of this promotes acceptance.
