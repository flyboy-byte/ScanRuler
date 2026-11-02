# Candidate image audit, outside the application workspace

This code probes image 0.25.10 with only png/jpeg/bmp/webp. The exact 22-package
closure is frozen by candidate.lock; manifests are generated only in a disposable
directory. There is no Cargo.toml here and no application dependency adoption.
The library intentionally returns raw library RGBA, without a fabricated canvas
continuation. It is a dependency/compatibility experiment, not readCompanions.

Reproduce with system Rust (no rustup), Node22, original npm dependencies and
Chrome. Committed fixtures need no Pillow; Pillow 13 was authoring tooling only.

```sh
python3 docs/port-plan/tools/prepare_image_audit.py /tmp/image-audit-repro
cargo build --locked --manifest-path /tmp/image-audit-repro/Cargo.toml
cargo metadata --locked --manifest-path /tmp/image-audit-repro/Cargo.toml --format-version 1 > /tmp/image-native.json
cargo metadata --locked --manifest-path /tmp/image-audit-repro/Cargo.toml --filter-platform wasm32-unknown-unknown --format-version 1 > /tmp/image-wasm.json
cargo rustc --locked --manifest-path /tmp/image-audit-repro/Cargo.toml --target wasm32-unknown-unknown --lib -- -C link-dead-code=yes -C link-arg=--export-all -C symbol-mangling-version=v0 -C link-arg=--max-memory=134217728
fnm exec --using=22 node docs/port-plan/oracle/capture-image-audit.mjs --out /tmp/image-source-recapture.json
python3 docs/port-plan/tools/compare_image_audit.py --binary /tmp/image-audit-repro/target/debug/scanruler-image-audit --out /tmp/image-candidate-recapture.json
python3 -m unittest discover -s docs/port-plan/tools -p test_image_audit.py
```

The candidate comparison uses the immutable committed original capture, not its
own decoder outputs. Recapture writes a new file for comparison; never overwrite
committed evidence to make a candidate pass. The full pinned modules are bundled
without editing their pixel implementation. Resource limits in this small spike
are prototype safeguards and do not prove production allocation/work bounds.
Wasm evidence is compile/link/import inspection only, not decoder runtime pixels.

Verdict: **do not adopt this raw decoder as a faithful replacement**. The audit
retains 28 pixel mismatches and 2 presence mismatches. Read the measured finding
and decision before choosing a decoder/canvas continuation. No tolerance change.
