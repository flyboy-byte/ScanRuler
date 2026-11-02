<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Authored registry scenarios

350 deterministic input-only scenarios; all hashes/IDs and seeds1/7/99/1039 are
in manifest.json. `tools/generate_registry_cases.py` reads only original command
metadata and authored inputs (including the earlier schema input fixtures), never
Rust or source answers. All140 definitions have successful/refused admission;
fixtures echo admitted arguments, not real feature implementations.

Protocol/source graph/reproduction: [REGISTRY.md](../../oracle/REGISTRY.md).
Initial209 source observations and both extensions remain losslessly archived.
