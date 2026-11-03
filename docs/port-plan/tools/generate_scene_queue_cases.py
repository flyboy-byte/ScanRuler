# SPDX-License-Identifier: AGPL-3.0-only
"""Independent scheduling matrix, not generated from either implementation."""
from pathlib import Path
def generate():
    return ''.join(f'case-{mask*15+pattern*3+order:03}\t{mask}\t{pattern}\t{order}\n' for mask in range(8) for pattern in range(5) for order in range(3)).encode()
if __name__=='__main__':
    folder=Path(__file__).resolve().parents[1]/'fixtures/scene-queue';folder.mkdir(exist_ok=True);(folder/'cases.tsv').write_bytes(generate())
