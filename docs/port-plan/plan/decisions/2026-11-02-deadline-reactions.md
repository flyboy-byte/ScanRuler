<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Deadline reaction ownership — 2026-11-02

Use a small owned event state machine for limited(), not a select wrapper that
always polls one future first. Source Promise.race distinguishes reactions after
registration from already-settled input-array ties. Abort dispatch occurs before
the timer's rejection and can itself settle the body. Separate those phases with
a domain-bound capability, then clear the timer before publishing the answer.

The caller result and body work have separate lifetimes. Timeout requests abort
but source body work may continue and change the session; retain that work and
consume its eventual result/error. Do not drop the import job or add a new epoch
filter to make this easier. A runtime must assemble this component with its owned
body tasks, clock and error normalization before any full command claim.

Move the existing owned ECMAScript number formatter unchanged to port-core, keep
the PLY wrapper/symbol and all old tests, and rerun every caller. The deadline
message still divides ms/1000 at timer fire and preserves lossless UTF16. Host
metadata/clock failure classes remain separate from source errors.
