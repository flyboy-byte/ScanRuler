<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# 1. What and why

## What ScanRuler is

ScanRuler (by CNC Kitchen) is a web app that checks a 3D scan against what the part should have
been. You load a scan in the browser and it fits spheres, cylinders and planes, measures between
them, paints the deviation from the CAD model, maps wall thickness, measures flatbed scans in
2D, and can rebuild a scan as a solid STEP body or clean up and reduce the mesh. Everything
runs locally in the browser. It also has an agent interface (an MCP server) so an AI can drive
it.

Under the hood it is TypeScript with React and three.js, and the heavy geometry runs in web
workers. It uses a few compiled kernels: a CAD kernel (Open CASCADE, compiled to WebAssembly),
a mesh boolean library, a mesh simplifier and a STEP importer.

## What "port to Rust" means here

The goal is a **faithful pure-Rust port of the whole public repository**, not a redesign:

- All six workspaces: 3D Measure, Deviation, Wall thickness, 2D Measure, Reverse engineer,
  Mesh Tools.
- File import and export, projects and crash recovery, and the agent interface.
- Runs natively and in the browser through WebAssembly.
- **Including the geometry kernels.** The end state has no C or C++ geometry code behind it, not
  even through Rust bindings and not even compiled to WebAssembly. A Rust crate that wraps a C++
  library is not a Rust kernel.

"Faithful" means the original's behaviour and mathematics are the authority. If the original
fits a plane a particular way, the port fits it that way, in the same order, with the same
defaults. Improvements are a separate piece of reviewed work, never a side effect of porting.

## What counts as done

The plan has two named checkpoints:

- **R1, application parity.** All six workflows work and match the original, but a retained
  non-Rust kernel or a Node helper may still be in use. Useful, but only a preview.
- **R2, the real endpoint.** Application, exact CAD kernel, mesh algorithms, file formats, UI and
  agent interface are all Rust, and a dependency audit of the shipped artifacts proves it.

There is no delivery date. Progress is tracked by capabilities completed and effort actually
spent. The hardest part is probably the CAD kernel and STEP handling, not the app itself, and the
plan says so openly (risk B07 in `GAPS-AND-DECISIONS.md`).

## Why do it

It is a personal side project, aimed at the direction of Tangency Labs, whose CAD stack is
written in Rust. It is also an unusually good exercise: a large, working, well-tested program to
port against, which means every step can be checked against the original instead of against
opinion.

## Licence facts (these shape everything)

- ScanRuler is **AGPL-3.0-only**, with commercial licences sold by its author. The port is a
  derivative, so it is AGPL too.
- Planning involved reading the original source. So this is **not a clean-room
  reimplementation**, and nobody should describe it as one later.
- A licensing enquiry was sent to the author; there is no reply on record. Do not assume any
  extra rights, do not contact him on the project's behalf, and do not sign the contributor
  agreement unless the owner says to.
- Putting this code into a permissive project would need rights that do not exist yet. Until they
  do, it is a standalone AGPL fork.
- The product name is the author's. Public releases will need different branding. The old file
  marker inside `.scanruler` project files stays as it is so old files keep opening.

## What this is not

It is not a way to avoid the upstream project, a certified metrology tool, or a new CAD kernel
project dressed up as an app port. New features, regulatory claims and automatic CAD editing are
outside the first complete release.
