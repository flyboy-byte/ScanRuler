<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Core history borrows — 2026-11-02

Capture/apply borrows the owner's four mutable field collections. Cell payloads
are shared, while captured objects own their small metadata. This bridges the
original dynamic state contract without introducing a duplicate business store.
Actual typed family models and source remeasurement still need integration.

Stable participant hooks inspect an immutable store view while applying their
own section data. Reentrant mutation of core maps, JS prototype participant keys
and reactive subscribers are outside this admitted host boundary. Those do not
become falsely accepted behavior; actual plugin/family runtime remains required.

The source oracle exposes two private functions through an appended export in a
fresh /tmp extraction. Keep every original byte before the append, and validate
that exact content through the compiled source map. This adapter grants test
access only; no original function body or dependency is changed.
