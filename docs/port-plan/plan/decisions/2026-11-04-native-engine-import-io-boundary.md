<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Native engine import uses the existing owned port-io crate — 2026-11-04

The native command handler must own parsing/geometry composition. The CLI supplies
bounded filesystem bytes and formats engine results. Add the existing workspace
port-io crate as an internal dependency of port-engine (port-io depends only on
port-core, so this adds no cycle). This adds no third-party package or foreign
runtime/kernel: all three crates are already owned pure Rust under AGPL-3.0-only.
No port-io image, zip, inflate or 3MF file is edited. Initial supported command
branches are STL and PLY; other formats stay explicitly unavailable until their
whole import continuations are connected. The reference stays 2cd89c6/app0.9.2.

This existing decision now accompanies public PR #105 (M3a) after the scan
ownership unit merged. Only the import decision/handler and native info CLI are
landed in this bounded step; Logan reviews/merges. Existing parser cases/budgets and all
numerical gates remain. Native OS thread/file adapters are platform boundaries;
shared document/geometry stays in Rust and Wasm compiles independently. This does
not claim browser/platform acceptance, project save or a complete runnable M3.
