#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Create a disposable exact candidate project; never edit application Cargo files."""
import argparse,json,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('directory',type=Path);a=p.parse_args()
    directory=a.directory.resolve()
    if directory.exists()and any(directory.iterdir()):raise SystemExit('Refuse to overwrite a nonempty directory')
    directory.mkdir(parents=True,exist_ok=True)
    spike=ROOT/'docs/port-plan/spikes/image-audit'
    quote=lambda p:json.dumps(str(p))
    manifest='''[package]
name = "scanruler-image-audit"
version = "0.0.0"
edition = "2024"
[dependencies]
image = { version = "=0.25.10", default-features = false, features = ["png", "jpeg", "bmp", "webp"] }
'''+f'\n[lib]\npath = {quote(spike/"decoder.rs")}\ncrate-type = ["cdylib", "rlib"]\n[[bin]]\nname = "scanruler-image-audit"\npath = {quote(spike/"main.rs")}\n'
    (directory/'Cargo.toml').write_text(manifest);shutil.copyfile(spike/'candidate.lock',directory/'Cargo.lock')
    print('Candidate-only project:',directory)
