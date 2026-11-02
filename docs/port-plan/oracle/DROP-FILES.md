# Dropped-folder port contract — 2026-11-02

io.obj.companions.drop. Source: complete pinned droppedFiles/fileOf/childrenOf
bodies, no replacement traversal. Authored platform snapshots/callbacks supply
file identities, optional getters, entry flags, directory batches and opaque
errors. Source and Rust receive independent copies, never source answers.

Compare every output identity, full error name/message and callback event order
exactly. Read all file-item entries first; if none is a directory, return the
browser fallback list. Otherwise skip absent entries, read top files in order,
collect every directory batch until empty **before** reading its file children,
and ignore nested directories. Keep duplicates, both-flag precedence, independent
readers for repeated directory entries and the exact point/error of each failure.

Generic Rust platform callbacks return opaque file/error types. Async traversal
owns only bounded temporary lists, not another file/session/history store. Source
platform failures remain distinct from host resource/allocation errors. No decoder,
filesystem recursion, sorting or eager streaming rewrite. Four fixed seeds.
Actual DOM/browser adapter execution and mutable live-list timing remain open;
Wasm compilation does not establish that runtime acceptance.
