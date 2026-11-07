<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Rust port: archived (2026-11-06)

This fork's `main` tracks the original ScanRuler by CNC Kitchen unchanged.

The AGPL-3.0 Rust port that lived here (native STL/PLY/OBJ/3MF import, session engine,
plane/sphere/cylinder fitting, undo/redo, project save/read, CLI, and its original-versus-Rust
comparison harness, through milestone M3) is preserved unchanged:

- branch [`rust-port`](../../tree/rust-port) and tag `port-archive-2026-11-06` (commit `51d100e`)

It is a derivative of ScanRuler and stays AGPL-3.0-only. It is no longer developed.

New work happens in [flyboy-byte/rulercraft](https://github.com/flyboy-byte/rulercraft): an
independent, clean-room mesh engine licensed MIT OR Apache-2.0. RulerCraft contains no code
from ScanRuler or from this port; it runs the original ScanRuler only as an external program
to compare results.
