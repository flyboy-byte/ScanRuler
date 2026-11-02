# Drop callback snapshots

40 independent authored inputs:24 hand cases and4 per seed1/7/99/1039;
AGPL-3.0-only. Generator encodes snapshots, never source/Rust expected answers.

Input: present byte; file-pool size; fallback ID list; items (file-kind/API flags,
entry index or u32 MAX, optional error); entries (file/directory flags, file ID,
optional file/reader errors, directory batches of optional error + child IDs).
Lengths/counts/IDs are little-endian u32; errors are optional tag plus UTF-8
name/message strings. Each repeated directory creates an independent reader.

Output DRP1 + status0 (full returned IDs) or1 (full platform name/message), then
every callback event in exact order. Source-only capture precedes Rust. These
are owned-snapshot/mock-platform comparisons, not actual browser DOM acceptance.
Success may legitimately contain zero files; no absent source output can pass.
